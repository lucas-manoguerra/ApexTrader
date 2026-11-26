use crate::errors::RendererError;
use eframe::egui;

pub trait ChartRenderer {

    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &ChartScene<'_>,
    ) -> Result<(), RendererError>;

    fn name(&self) -> &str;
}

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
    PriceAxisLabel, RenderColor, RenderFrame, RenderGeometry, RenderLayer, TimeAxisLabel,
    VolumeBarRender,
};
pub use wgpu::WgpuChartRenderer;
