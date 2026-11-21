use crate::domain::candle::Candle;
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
    /// Dibuja las velas en el área dada.
    ///
    /// `painter` es el contexto de dibujo de egui (disponible en `ui.painter()`).
    /// `rect` es el área disponible en pantalla.
    /// `candles` son los datos prestados por `ChartState` (`&[Candle]`).
    /// `viewport` es el estado de zoom y pan prestado por `ChartState` (`&Viewport`).
    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        candles: &[Candle],
        viewport: &Viewport,
        crosshair_pos: Option<(f64, f64)>,
    ) -> Result<(), RendererError>;

    /// Nombre legible del renderer (para logs y depuración).
    fn name(&self) -> &str;
}

/// Factory que intenta crear el mejor renderer disponible.
///
/// Intenta wgpu primero; si falla (por ejemplo, GPU no compatible) o si no
/// hay surface para dibujar, cae a glow.
pub fn create_renderer() -> Box<dyn ChartRenderer> {
    match WgpuChartRenderer::new() {
        Ok(renderer) => {
            // wgpu se inicializó, pero sin surface no puede dibujar.
            // Verificamos si tiene surface disponible.
            if renderer.has_surface() {
                tracing::info!("Usando renderer: wgpu");
                Box::new(renderer)
            } else {
                tracing::warn!("wgpu inicializado sin surface, usando glow como fallback");
                Box::new(GlowChartRenderer::new())
            }
        }
        Err(e) => {
            tracing::warn!("wgpu no disponible ({}), usando glow como fallback", e);
            Box::new(GlowChartRenderer::new())
        }
    }
}

mod glow;
mod wgpu;
mod wgpu_pipeline;

pub use glow::GlowChartRenderer;
pub use wgpu::WgpuChartRenderer;
