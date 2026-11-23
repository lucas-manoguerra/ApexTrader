use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use eframe::egui;

/// Trait que abstrae el renderizado de un chart de velas.
///
/// Cualquier implementación (glow, wgpu, futuro software renderer, etc.)
/// debe cumplir este contrato. El dominio (`ChartState`) no conoce a ninguna
/// implementación concreta: solo recibe un `&mut dyn ChartRenderer` y le pide
/// que dibuje las velas que él posee.
pub trait ChartRenderer {
    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        points: &[ChartDataPoint],
        viewport: &Viewport,
        crosshair_pos: Option<ScreenPoint>,
    ) -> Result<(), RendererError>;

    /// Nombre legible del renderer (para logs y depuración).
    fn name(&self) -> &str;
}

/// Factory que crea el renderer con capacidad efectiva para dibujar.
///
/// Hoy `glow` (egui Painter) es el único backend funcional. `wgpu` se
/// inicializa (instance, adapter, device, queue) pero no puede renderizar
/// sin una `Surface` real vinculada a la ventana. Hasta que se implemente
/// esa integración, wgpu está aislado como backend experimental y no se
/// presenta como funcional.
pub fn create_renderer() -> Box<dyn ChartRenderer> {
    tracing::info!(
        "Usando renderer: glow (egui Painter). wgpu requiere integración real de surface con la ventana antes de poder renderizar."
    );
    Box::new(GlowChartRenderer::new())
}

mod glow;
mod wgpu;
mod wgpu_pipeline;

pub use glow::GlowChartRenderer;
pub use wgpu::WgpuChartRenderer;
