use crate::errors::RendererError;
use eframe::egui;

/// Trait que abstrae el renderizado de un chart de velas.
///
/// Cualquier implementación (glow, wgpu, futuro software renderer, etc.)
/// debe cumplir este contrato. El dominio (`ChartState`) no conoce a ninguna
/// implementación concreta: solo recibe un `&mut dyn ChartRenderer` y le pide
/// que dibuje la escena que él posee.
pub trait ChartRenderer {
    /// Dibuja la escena en el área dada.
    ///
    /// `painter` es el contexto de dibujo de egui (disponible en `ui.painter()`).
    /// `rect` es el área disponible en pantalla.
    /// `scene` es la escena de renderizado completa: qué dibujar, sin decir cómo.
    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &ChartScene<'_>,
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
mod ir;
mod wgpu;
mod wgpu_pipeline;

pub use glow::GlowChartRenderer;
pub use ir::{
    CandleRender, ChartScene, CrosshairRender, InvalidationMask, InvalidationReason,
    PriceAxisLabel, RenderFrame, RenderGeometry, RenderLayer, TimeAxisLabel, VolumeBarRender,
};
pub use wgpu::WgpuChartRenderer;
