//! Adaptador de eventos de egui al sistema normalizado de interacción.
//!
//! Este módulo es la única frontera que conoce egui. Las coordenadas se
//! convierten una sola vez del rect completo al área útil de plot; después,
//! el controlador opera sobre el Viewport propiedad de ChartState.

use crate::chart::interaction::controller::InteractionController;
use crate::chart::interaction::events::{
    AxisDoubleClickEvent, AxisPointerDownEvent, InteractionEvent, Modifiers, PointerButton,
    PointerDownEvent, PointerDragEvent, PointerUpEvent, ScaleAxis, ScrollEvent,
};
use crate::chart::interaction::options::InteractionOptions;
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
        Self::with_options(InteractionOptions::default())
    }

    /// Crea el adaptador de entrada con opciones de interacción explícitas.
    pub fn with_options(options: InteractionOptions) -> Self {
        Self {
            controller: InteractionController::with_options(0.0, 0.0, options),
            crosshair_pos: None,
        }
    }

    /// Devuelve las opciones de interacción activas.
    pub fn options(&self) -> InteractionOptions {
        self.controller.options()
    }

    /// Actualiza las opciones de interacción.
    ///
    /// Si cambia la configuración, el controlador cancela cualquier gesto
    /// activo para evitar continuar con anclas obsoletas.
    pub fn set_options(&mut self, options: InteractionOptions) {
        self.controller.set_options(options);
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

    // Adaptar el scroll vertical (zoom temporal) y horizontal (pan) al punto bajo el cursor.
    let scroll_delta = ui.input(|input| input.smooth_scroll_delta);
    if (scroll_delta.x != 0.0 || scroll_delta.y != 0.0)
        && let Some(pos) = ui.input(|input| input.pointer.hover_pos())
        && let Some((screen_point, chart_point)) =
            pointer_coordinates(pos, interaction_rect, &layout)
    {
        let event = InteractionEvent::Scroll(ScrollEvent {
            delta_x: scroll_delta.x as f64,
            delta_y: scroll_delta.y as f64,
            x: chart_point.x,
            y: chart_point.y,
            modifiers: Modifiers::default(),
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);

        input_handler.crosshair_pos = Some(screen_point);
    }

    // El doble clic sobre un eje restablece exclusivamente esa escala.
    if response.double_clicked_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((axis, _)) = axis_pointer_coordinates(pos, interaction_rect, &layout)
    {
        let event = InteractionEvent::AxisDoubleClick(AxisDoubleClickEvent { axis });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }

    // Un arrastre iniciado sobre un eje escala ese eje; en el plot conserva
    // el comportamiento anterior de pan con el botón principal.
    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        if let Some((axis, point)) = axis_pointer_coordinates(pos, interaction_rect, &layout) {
            let event = InteractionEvent::AxisPointerDown(AxisPointerDownEvent {
                axis,
                x: point.x,
                y: point.y,
            });
            input_handler
                .controller
                .on_event(chart.viewport_mut(), candle_count, &event);
        } else if let Some((_, point)) = pointer_coordinates(pos, interaction_rect, &layout) {
            let event = InteractionEvent::PointerDown(PointerDownEvent {
                button: PointerButton::Primary,
                x: point.x,
                y: point.y,
                modifiers: Modifiers::default(),
            });
            input_handler
                .controller
                .on_event(chart.viewport_mut(), candle_count, &event);
        }
    }

    if response.drag_started_by(egui::PointerButton::Secondary)
        && let Some(pos) = response.interact_pointer_pos()
        && let Some((_, point)) = pointer_coordinates(pos, interaction_rect, &layout)
    {
        let event = InteractionEvent::PointerDown(PointerDownEvent {
            button: PointerButton::Secondary,
            x: point.x,
            y: point.y,
            modifiers: Modifiers::default(),
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }

    // Los eventos de arrastre contienen posiciones absolutas: el controlador
    // calcula siempre el desplazamiento desde el punto de inicio del gesto.
    if response.dragged_by(egui::PointerButton::Primary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let (_, point) = pointer_coordinates_unbounded(pos, interaction_rect, &layout);
        let event = InteractionEvent::PointerDrag(PointerDragEvent {
            x: point.x,
            y: point.y,
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }

    if response.dragged_by(egui::PointerButton::Secondary)
        && let Some(pos) = response.interact_pointer_pos()
    {
        let (_, point) = pointer_coordinates_unbounded(pos, interaction_rect, &layout);
        let event = InteractionEvent::PointerDrag(PointerDragEvent {
            x: point.x,
            y: point.y,
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }

    if response.drag_stopped_by(egui::PointerButton::Primary) {
        let event = InteractionEvent::PointerUp(PointerUpEvent {
            button: PointerButton::Primary,
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }
    if response.drag_stopped_by(egui::PointerButton::Secondary) {
        let event = InteractionEvent::PointerUp(PointerUpEvent {
            button: PointerButton::Secondary,
        });
        input_handler
            .controller
            .on_event(chart.viewport_mut(), candle_count, &event);
    }

    // El crosshair se guarda relativo al rect completo del chart; las
    // coordenadas del controlador, en cambio, son relativas al plot.
    input_handler.crosshair_pos = ui
        .input(|input| input.pointer.hover_pos())
        .and_then(|pos| pointer_coordinates(pos, interaction_rect, &layout))
        .map(|(screen_point, _)| screen_point);
}

fn axis_pointer_coordinates(
    pos: egui::Pos2,
    interaction_rect: egui::Rect,
    layout: &ChartLayout,
) -> Option<(ScaleAxis, ChartPoint)> {
    let x = (pos.x - interaction_rect.left()) as f64;
    let y = (pos.y - interaction_rect.top()) as f64;
    let inside = |rect: crate::chart::layout::Rect| {
        x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom
    };

    let axis = if inside(layout.price_axis_right) {
        ScaleAxis::Price
    } else if inside(layout.time_axis) {
        ScaleAxis::Time
    } else {
        return None;
    };

    let screen_point = ScreenPoint { x, y };
    let chart_point = screen_to_chart_point(screen_point, layout.plot.left, layout.plot.top);
    Some((axis, chart_point))
}

fn pointer_coordinates(
    pos: egui::Pos2,
    interaction_rect: egui::Rect,
    layout: &ChartLayout,
) -> Option<(ScreenPoint, ChartPoint)> {
    let (screen_point, chart_point) = pointer_coordinates_unbounded(pos, interaction_rect, layout);
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
    let chart_point = screen_to_chart_point(screen_point, layout.plot.left, layout.plot.top);
    (screen_point, chart_point)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn interaction_rect() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(100.0, 60.0), egui::vec2(800.0, 600.0))
    }

    #[test]
    fn input_handler_exposes_and_updates_interaction_options() {
        let mut input_handler = InputHandler::new();
        let mut options = InteractionOptions::default();
        options.scale.vertical_mouse_wheel = false;

        input_handler.set_options(options);

        assert_eq!(input_handler.options(), options);
    }

    #[test]
    fn pointer_coordinates_convert_global_pointer_to_local_plot_space() {
        let rect = interaction_rect();
        let layout = ChartLayout::compute(rect.width() as f64, rect.height() as f64);
        let expected_x = layout.plot_width() * 0.35;
        let expected_y = layout.plot_height() * 0.70;
        let pos = egui::pos2(
            rect.left() + (layout.plot.left + expected_x) as f32,
            rect.top() + (layout.plot.top + expected_y) as f32,
        );

        let (screen, chart) =
            pointer_coordinates(pos, rect, &layout).expect("el puntero está dentro del plot");

        assert!((screen.x - (layout.plot.left + expected_x)).abs() < 1e-3);
        assert!((screen.y - (layout.plot.top + expected_y)).abs() < 1e-3);
        assert!((chart.x - expected_x).abs() < 1e-3);
        assert!((chart.y - expected_y).abs() < 1e-3);
    }

    #[test]
    fn axis_pointer_coordinates_identify_right_price_and_time_scales() {
        let rect = interaction_rect();
        let layout = ChartLayout::compute(rect.width() as f64, rect.height() as f64);
        let price_pos = egui::pos2(
            rect.left()
                + (layout.price_axis_right.left + layout.price_axis_right.width() / 2.0) as f32,
            rect.top()
                + (layout.price_axis_right.top + layout.price_axis_right.height() / 2.0) as f32,
        );
        let time_pos = egui::pos2(
            rect.left() + (layout.time_axis.left + layout.time_axis.width() / 2.0) as f32,
            rect.top() + (layout.time_axis.top + layout.time_axis.height() / 2.0) as f32,
        );

        let (price_axis, _) = axis_pointer_coordinates(price_pos, rect, &layout)
            .expect("la escala derecha debe detectarse");
        let (time_axis, _) = axis_pointer_coordinates(time_pos, rect, &layout)
            .expect("el eje temporal debe detectarse");

        assert_eq!(price_axis, ScaleAxis::Price);
        assert_eq!(time_axis, ScaleAxis::Time);
    }

    #[test]
    fn left_gutter_is_not_a_visible_price_scale() {
        let rect = interaction_rect();
        let layout = ChartLayout::compute(rect.width() as f64, rect.height() as f64);
        let pos = egui::pos2(
            rect.left() + (layout.price_axis_left.left + layout.price_axis_left.width() / 2.0) as f32,
            rect.top() + (layout.price_axis_left.top + layout.price_axis_left.height() / 2.0) as f32,
        );
        assert!(axis_pointer_coordinates(pos, rect, &layout).is_none());
    }

    #[test]
    fn pointer_in_axis_margin_is_not_a_plot_interaction() {
        let rect = interaction_rect();
        let layout = ChartLayout::compute(rect.width() as f64, rect.height() as f64);
        let pos = egui::pos2(
            rect.left() + (layout.plot.left / 2.0) as f32,
            rect.top() + (layout.plot.top + layout.plot.height() / 2.0) as f32,
        );

        assert!(pointer_coordinates(pos, rect, &layout).is_none());
    }
}
