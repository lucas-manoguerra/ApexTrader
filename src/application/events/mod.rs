//! Interpretación de entrada de usuario.
//!
//! Traduce eventos de egui (scroll, arrastre, posición del mouse) en acciones
//! de dominio (zoom, pan, actualización del crosshair) sobre el `ChartState`.

use crate::chart::layout::ChartLayout;
use crate::domain::chart_state::ChartState;
use crate::domain::transforms::ScreenPoint;
use eframe::egui;

/// Manejador de entrada del chart.
///
/// Encapsula la lógica de zoom (scroll), pan (arrastre) y crosshair (posición
/// del cursor). El dominio no conoce este manejador: solo expone el estado
/// resultante (`crosshair_pos`).
pub struct InputHandler {
    crosshair_pos: Option<ScreenPoint>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            crosshair_pos: None,
        }
    }

    /// Devuelve la última posición del crosshair en `ScreenPoint`.
    pub fn crosshair_pos(&self) -> Option<ScreenPoint> {
        self.crosshair_pos
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Procesa los eventos de entrada de un frame y actualiza el estado del
/// gráfico (zoom, pan, crosshair).
///
/// `input_handler` es el manejador mutable de entrada.
/// `ui` es el contexto de egui para leer input.
/// `response` es la respuesta del área interactiva (`ui.interact`).
/// `interaction_rect` es el área interactiva del chart.
/// `chart` es el estado mutable del gráfico.
pub fn handle_input(
    input_handler: &mut InputHandler,
    ui: &egui::Ui,
    response: &egui::Response,
    interaction_rect: egui::Rect,
    chart: &mut ChartState,
) {
    // --- Manejo de input: Zoom con scroll ---
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
    if scroll_delta.y != 0.0
        && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
    {
        // Coordenadas normalizadas relativas al área de plot (no al rect
        // completo), usando el mismo ChartLayout que el renderer (AT-002).
        let layout = ChartLayout::compute(
            interaction_rect.width() as f64,
            interaction_rect.height() as f64,
        );
        let center_x = if layout.plot.width() > 0.0 {
            ((pos.x as f64 - interaction_rect.left() as f64 - layout.plot.left)
                / layout.plot.width())
            .clamp(0.0, 1.0)
        } else {
            0.5
        };
        let center_y = if layout.plot.height() > 0.0 {
            ((pos.y as f64 - interaction_rect.top() as f64 - layout.plot.top)
                / layout.plot.height())
            .clamp(0.0, 1.0)
        } else {
            0.5
        };

        let candle_count = chart.len();
        let screen_width = layout.plot_width();
        let screen_height = layout.plot_height();

        let vp = chart.viewport_mut();
        // El scroll vertical afecta al eje Y (precio); el eje X no se mueve.
        vp.zoom_at(
            0.0,
            scroll_delta.y as f64,
            center_x,
            center_y,
            screen_width,
            screen_height,
        );
        vp.clamp_zoom_to_candles(candle_count, screen_width);
        vp.clamp_pan_to_candles(candle_count, screen_width);
    }

    // --- Manejo de input: Pan con arrastre ---
    if response.dragged() {
        let delta = response.drag_delta();

        let candle_count = chart.len();
        let screen_width = interaction_rect.width() as f64;

        let vp = chart.viewport_mut();
        vp.pan(
            delta.x as f64,
            delta.y as f64,
            interaction_rect.height() as f64,
        );
        vp.clamp_pan_to_candles(candle_count, screen_width);
    }

    // --- Manejo de input: Crosshair sigue el cursor ---
    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
        let rel_x = pos.x - interaction_rect.left();
        let rel_y = pos.y - interaction_rect.top();
        input_handler.crosshair_pos = Some(ScreenPoint {
            x: rel_x as f64,
            y: rel_y as f64,
        });
    }
}
