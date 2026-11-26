use crate::errors::RendererError;
use crate::renderer::{ChartRenderer, ChartScene};
use eframe::egui;
use tracing::{info, warn};

pub use crate::renderer::wgpu_pipeline::WgpuPipeline;







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






    pub fn init_surface(
        &mut self,
        _raw_handle: raw_window_handle::RawWindowHandle,
        _width: u32,
        _height: u32,
    ) -> Result<(), RendererError> {
        warn!("init_surface stub: surface no creada todavía. Se usará glow como fallback.");
        Ok(())
    }





    pub fn has_surface(&self) -> bool {
        self.surface.is_some() && self.pipeline.is_some()
    }
    pub fn resize(&mut self, width: u32, height: u32) {

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





    pub fn present(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl ChartRenderer for WgpuChartRenderer {
    fn draw(
        &mut self,
        _painter: &egui::Painter,
        _rect: egui::Rect,
        _scene: &ChartScene<'_>,
    ) -> Result<(), RendererError> {


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
