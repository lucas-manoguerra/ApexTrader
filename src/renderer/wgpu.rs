use crate::domain::candle::Candle;
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use crate::renderer::ChartRenderer;
use eframe::egui;
use tracing::{info, warn};

pub use crate::renderer::wgpu_pipeline::WgpuPipeline;

/// Renderizador de chart usando wgpu (GPU acelerado con DirectX 12 en Windows).
///
/// Inicializa instance, adapter, device y queue. Cuando se disponga del
/// window handle de winit, crea la Surface y renderiza las velas directamente
/// en la GPU.
///
/// Mientras no haya surface, el dibujo visual sigue a cargo de GlowChartRenderer.
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
    /// Este método debe llamarse una vez que se disponga del `RawWindowHandle`
    /// de la ventana (por ejemplo, el `HWND` de Windows).
    pub fn init_surface(
        &mut self,
        _raw_handle: raw_window_handle::RawWindowHandle,
        _width: u32,
        _height: u32,
    ) -> Result<(), RendererError> {
        // Stub: surface no creada todavía. Cuando dispongamos del window handle
        // de winit, se implementará con la API correcta de wgpu 30.
        warn!("init_surface stub: surface no creada todavía. Se usará glow como fallback.");
        Ok(())
    }

    /// Indica si el renderer wgpu tiene una surface lista para dibujar.
    pub fn has_surface(&self) -> bool {
        self.surface.is_some() && self.pipeline.is_some()
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        if let (Some(ref mut config), Some(surface)) = (self.config.as_mut(), self.surface.as_ref())
            && width > 0 && height > 0 {
                config.width = width;
                config.height = height;
                surface.configure(&self.device, config);
                info!("Surface redimensionada: {}x{}", width, height);
            }
    }

    /// Presenta el frame actual (swap chain).
    /// En wgpu 30, SurfaceTexture::Drop hace el present automáticamente.
    pub fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // wgpu 30: SurfaceTexture se presenta automáticamente al ser dropeada.
        // No hay método .present() explícito.
        Ok(())
    }
}

impl ChartRenderer for WgpuChartRenderer {
    fn draw(
        &mut self,
        _painter: &egui::Painter,
        _rect: egui::Rect,
        _candles: &[Candle],
        _viewport: &Viewport,
    ) -> Result<(), RendererError> {
        // Si no hay surface, no podemos dibujar con wgpu.
        // Devolvemos Err para que el caller sepa que wgpu no está listo
        // y se use glow como fallback.
        if self.surface.is_none() || self.pipeline.is_none() {
            return Err(RendererError::BackendInitFailed(
                "wgpu surface no inicializada".to_string(),
            ));
        }

        // Cuando la surface esté inicializada, aquí se renderizará con wgpu.
        // Por ahora, retorna Ok como placeholder.
        Ok(())
    }

    fn name(&self) -> &str {
        "wgpu (DirectX 12)"
    }
}
