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

pub fn create_renderer(target_format: Option<eframe::wgpu::TextureFormat>) -> Box<dyn ChartRenderer> {
    tracing::info!(
        "Usando renderer basado en egui::Painter; la presentación GPU corresponde al backend seleccionado por eframe."
    );
    Box::new(EguiPainterChartRenderer::with_target_format(target_format))
}

mod egui_painter;
mod geometry;
mod ir;
mod wgpu_callback;

pub use egui_painter::EguiPainterChartRenderer;
pub use ir::{
    CandleRender, ChartScene, CrosshairRender, InvalidationMask, InvalidationReason,
    PriceAxisLabel, RenderColor, RenderFrame, RenderGeometry, RenderLayer, RenderVertex,
    TimeAxisLabel, VolumeBarRender,
};

pub use geometry::build_geometry;
pub use wgpu_callback::{ChartGpuCallback, GpuVertex};
