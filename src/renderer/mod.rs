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
    /// Dibuja las velas en el área dada.
    ///
    /// `painter` es el contexto de dibujo de egui (disponible en `ui.painter()`).
    /// `rect` es el área disponible en pantalla.
    /// `points` son los datos de chart prestados por `ChartState` (`&[ChartDataPoint]`).
    /// `viewport` es el estado de zoom y pan prestado por `ChartState` (`&Viewport`).
    /// `crosshair_pos` es la posición del cursor en `ScreenPoint` (coordenadas
    /// relativas al rect de dibujo recibido por el renderer).
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
/// wgpu requiere una `Surface` real vinculada a un window handle (winit / HWND)
/// antes de poder renderizar. Hoy `init_surface()` es un stub y no hay
/// integración de wgpu con la ventana administrada por eframe, así que
/// inicializar wgpu solo pagaría el costo de `Instance`/`Adapter`/`Device`/
/// `Queue` sin obtener renderizado GPU.
///
/// Seleccionamos el backend por capacidad efectiva, no por mera creación de
/// `Device`. Mientras no exista una surface real vinculada a la ventana, se
/// usa Glow (egui Painter), que es el único backend que puede dibujar hoy.
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
