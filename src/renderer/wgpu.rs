use crate::errors::RendererError;
use crate::renderer::{ChartRenderer, ChartScene};
use eframe::egui;
use tracing::{info, warn};

pub use crate::renderer::wgpu_pipeline::WgpuPipeline;

/// Renderizador de chart usando wgpu (GPU acelerado con DirectX 12 en Windows).
///
/// **Estado: experimental / unfinished.** Inicializa instance, adapter, device
/// y queue, pero no puede renderizar sin una `Surface` real vinculada a la
/// ventana. Hasta que se implemente esa integración, wgpu no se presenta como
/// un backend funcional; `glow` sigue siendo el único renderer usable.
pub struct WgpuChartRenderer {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    pipeline: Option<WgpuPipeline>,
    available: bool,
}

impl WgpuChartRenderer {
    /// Intenta inicializar wgpu sin surface (solo instance, adapter, device, queue).
    ///
    /// Este método solo crea los objetos de contexto GPU. No puede renderizar
    /// hasta que se llame a `init_surface` con un window handle real. Hasta
    /// entonces, `draw()` devuelve `Err`.
    pub fn new() -> Result<Self, RendererError> {
        info!("Intentando inicializar wgpu...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            flags: wgpu::InstanceFlags::empty(),
            backend_options: Default::default(),
            display: Default::default(),
            memory_budget_thresholds: Default::default(),
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .map_err(|_| RendererError::BackendInitFailed("No se encontró adapter wgpu".to_string()))?;

        info!("Adapter wgpu encontrado: {:?}", adapter.get_info().name);

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: Some("apex-trader-device"),
            trace: wgpu::Trace::Off,
            memory_hints: wgpu::MemoryHints::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
        }))
        .map_err(|e| RendererError::BackendInitFailed(format!("No se pudo crear device: {}", e)))?;

        info!("Device y queue de wgpu creados correctamente");

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            config: None,
            pipeline: None,
            available: true,
        })
    }

    /// Inicializa la surface de wgpu a partir de un window handle nativo.
    ///
    /// **Stub:** hoy no crea una surface real. Cuando se disponga del
    /// `RawWindowHandle` de la ventana, se implementará con la API correcta
    /// de wgpu 30. Hasta entonces, `draw()` seguirá devolviendo `Err`.
    pub fn init_surface(
        &mut self,
        _raw_handle: raw_window_handle::RawWindowHandle,
        _width: u32,
        _height: u32,
    ) -> Result<(), RendererError> {
        warn!("init_surface stub: surface no creada todavía. Se usará glow como fallback.");
        Ok(())
    }

    /// Indica si el renderer wgpu tiene una surface lista para dibujar.
    ///
    /// Retorna `true` solo si hay surface y pipeline inicializados. Hoy,
    /// como `init_surface` es un stub, siempre retorna `false`.
    pub fn has_surface(&self) -> bool {
        self.surface.is_some() && self.pipeline.is_some()
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        // No-op hasta que haya surface y pipeline configurados.
        if let (Some(ref mut config), Some(surface)) = (self.config.as_mut(), self.surface.as_ref())
            && width > 0
            && height > 0
        {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
            info!("Surface redimensionada: {}x{}", width, height);
        }
    }

    /// Presenta el frame actual (swap chain).
    ///
    /// **Placeholder:** en wgpu 30, SurfaceTexture::Drop hace el present
    /// automáticamente. Hasta que haya surface real, este método no hace nada.
    pub fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl ChartRenderer for WgpuChartRenderer {
    fn draw(
        &mut self,
        _painter: &egui::Painter,
        _rect: egui::Rect,
        _scene: &ChartScene,
    ) -> Result<(), RendererError> {
        // wgpu no puede dibujar sin surface ni pipeline. Retorna Err para que
        // el caller use glow como fallback.
        if self.surface.is_none() || self.pipeline.is_none() {
            return Err(RendererError::BackendInitFailed(
                "wgpu surface no inicializada".to_string(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "wgpu (DirectX 12)"
    }
}
