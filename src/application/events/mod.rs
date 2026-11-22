//! Interpretación de entrada de usuario.
//!
//! Traduce eventos de egui (scroll, arrastre, posición del mouse) en acciones
//! de dominio (zoom, pan, actualización del crosshair) sobre el `ChartState`.

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

    /// Procesa los eventos de entrada de un frame y actualiza el estado del
    /// gráfico (zoom, pan, crosshair).
    ///
    /// `ui` es el contexto de egui para leer input.
    /// `response` es la respuesta del área interactiva (`ui.interact`).
    /// `interaction_rect` es el área interactiva del chart.
    /// `chart` es el estado mutable del gráfico.
    pub fn handle_input(
        &mut self,
        ui: &mut egui::Ui,
        response: &egui::Response,
        interaction_rect: egui::Rect,
        chart: &mut ChartState,
    ) {
        // --- Manejo de input: Zoom con scroll ---
        let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
        if scroll_delta.y != 0.0 {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let center_x = (pos.x - interaction_rect.left()) / interaction_rect.width();
                let center_y = (pos.y - interaction_rect.top()) / interaction_rect.height();

                let candle_count = chart.len();
                let screen_width = interaction_rect.width() as f64;
                let screen_height = interaction_rect.height() as f64;

                let vp = chart.viewport_mut();
                vp.zoom_at(
                    scroll_delta.y as f64,
                    center_x as f64,
                    center_y as f64,
                    screen_width,
                    screen_height,
                );
                vp.clamp_zoom_to_candles(candle_count, screen_width);
                vp.clamp_pan_to_candles(candle_count, screen_width);
            }
        }

        // --- Manejo de input: Pan con arrastre ---
        if response.dragged() {
            let delta = response.drag_delta();

            let candle_count = chart.len();
            let screen_width = interaction_rect.width() as f64;

            let vp = chart.viewport_mut();
            vp.pan(delta.x as f64, delta.y as f64, interaction_rect.height() as f64);
            vp.clamp_pan_to_candles(candle_count, screen_width);
        }

        // --- Manejo de input: Crosshair sigue el cursor ---
        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
            let rel_x = pos.x - interaction_rect.left();
            let rel_y = pos.y - interaction_rect.top();
            self.crosshair_pos = Some(ScreenPoint {
                x: rel_x as f64,
                y: rel_y as f64,
            });
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
