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
    match target_format {
        Some(_) => tracing::info!(
            "Usando pipeline WGPU para la serie del chart y egui para los elementos de presentación."
        ),
        None => tracing::warn!(
            "No hay target WGPU disponible; la serie del chart usará el renderer Painter de compatibilidad."
        ),
    }
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
