//! Adaptador de eventos de egui al sistema normalizado de interacción.
//!
//! Este módulo es la única frontera que conoce egui. Las coordenadas se
//! convierten una sola vez del rect completo al área útil de plot; después,
//! el controlador opera sobre el Viewport propiedad de ChartState.

use crate::chart::interaction::controller::InteractionController;
use crate::chart::interaction::events::{
    InteractionEvent, Modifiers, PointerButton, PointerDownEvent, PointerDragEvent,
    PointerMoveEvent, PointerUpEvent, ScrollEvent,
};
use crate::chart::layout::ChartLayout;
use crate::domain::chart_state::ChartState;
use crate::domain::transforms::{ChartPoint, ScreenPoint, screen_to_chart_point};
use eframe::egui;

/// Manejador persistente de entrada del chart.
///
/// Conserva el estado del gesto entre frames. No posee un Viewport ni duplica
/// zoom/pan: cada evento modifica el Viewport compartido del ChartState.
pub struct InputHandler {
    controller: InteractionController,
    crosshair_pos: Option<ScreenPoint>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            controller: InteractionController::new(0.0, 0.0),
            crosshair_pos: None,
        }
    }

    /// Devuelve la posición del crosshair relativa al rect completo del chart.
    pub fn crosshair_pos(&self) -> Option<ScreenPoint> {
        self.crosshair_pos
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Procesa la entrada del frame y actualiza el Viewport compartido.
///
/// Los gestos de arrastre usan la posición absoluta actual del puntero; el
/// controlador calcula siempre el cambio desde el punto de inicio. Esto evita
/// sumar varias veces un delta acumulado de egui.
pub fn handle_input(
    input_handler: &mut InputHandler,
    ui: &egui::Ui,
    response: &egui::Response,
    interaction_rect: egui::Rect,
    chart: &mut ChartState,
) {
    let layout = ChartLayout::compute(
        interaction_rect.width() as f64,
        interaction_rect.height() as f64,
    );
    if !layout.is_valid() {
        input_handler.crosshair_pos = None;
        input_handler.controller.set_screen_size(0.0, 0.0);
        return;
    }

    let plot_width = layout.plot_width();
    let plot_height = layout.plot_height();
    input_handler
        .controller
        .set_screen_size(plot_width, plot_height);

    let candle_count = chart.len();

    // Zoom por rueda, anclado al punto real del cursor dentro del plot.
    let scroll_delta = ui.input(|input| input.smooth_scroll_delta);
    if scroll_delta.y != 0.0 {
        if let Some(pos) = ui.input(|input| input.pointer.hover_pos()) {
            if let Some((screen_point, chart_point)) =
                pointer_coordinates(pos, interaction_rect, &layout)
            {
                let event = InteractionEvent::Scroll(ScrollEvent {
                    delta_y: scroll_delta.y as f64,
                    x: chart_point.x,
                    y: chart_point.y,
                    modifiers: Modifiers::default(),
                });
                input_handler.controller.on_event(
                    chart.viewport_mut(),
                    candle_count,
                    &event,
                );

                // Mantener el crosshair en la ubicación del puntero.
                input_handler.crosshair_pos = Some(screen_point);
            }
        }
    }

    // Capturar el inicio de pan antes de procesar el primer movimiento.
    if response.drag_started_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some((_, point)) = pointer_coordinates(pos, interaction_rect, &layout) {
                let event = InteractionEvent::PointerDown(PointerDownEvent {
                    button: PointerButton::Primary,
                    x: point.x,
                    y: point.y,
                    modifiers: Modifiers::default(),
                });
                input_handler.controller.on_event(
                    chart.viewport_mut(),
                    candle_count,
                    &event,
                );
            }
        }
    }

    // Pan con posición absoluta: no se suma drag_delta frame tras frame.
    if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            let (_, point) = pointer_coordinates_unbounded(pos, interaction_rect, &layout);
            let event = InteractionEvent::PointerDrag(PointerDragEvent {
                x: point.x,
                y: point.y,
            });
            input_handler.controller.on_event(chart.viewport_mut(), candle_count, &event);
        }
    }

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        let event = InteractionEvent::PointerUp(PointerUpEvent {
            button: PointerButton::Primary,
        });
        input_handler.controller.on_event(chart.viewport_mut(), candle_count, &event);
    }

    // El crosshair usa ScreenPoint relativo al rect completo, mientras que
    // las coordenadas de interacción del controlador son relativas al plot.
    input_handler.crosshair_pos = ui
        .input(|input| input.pointer.hover_pos())
        .and_then(|pos| pointer_coordinates(pos, interaction_rect, &layout))
        .map(|(screen_point, _)| screen_point);
}

fn pointer_coordinates(
    pos: egui::Pos2,
    interaction_rect: egui::Rect,
    layout: &ChartLayout,
) -> Option<(ScreenPoint, ChartPoint)> {
    let (screen_point, chart_point) =
        pointer_coordinates_unbounded(pos, interaction_rect, layout);
    if chart_point.x < 0.0
        || chart_point.y < 0.0
        || chart_point.x > layout.plot_width()
        || chart_point.y > layout.plot_height()
    {
        return None;
    }
    Some((screen_point, chart_point))
}

fn pointer_coordinates_unbounded(
    pos: egui::Pos2,
    interaction_rect: egui::Rect,
    layout: &ChartLayout,
) -> (ScreenPoint, ChartPoint) {
    let screen_point = ScreenPoint {
        x: (pos.x - interaction_rect.left()) as f64,
        y: (pos.y - interaction_rect.top()) as f64,
    };
    let chart_point = screen_to_chart_point(
        screen_point,
        layout.plot.left,
        layout.plot.top,
    );
    (screen_point, chart_point)
}
