//! Controlador de interacción del chart.
//!
//! El controlador conserva únicamente el estado transitorio de los gestos.
//! El Viewport es prestado por ChartState en cada evento y sigue siendo la
//! única fuente de verdad para zoom y pan.

use crate::chart::interaction::events::{
    InteractionEvent, InteractionResult, PointerButton, PointerDownEvent, PointerDragEvent,
    PointerMoveEvent, PointerUpEvent, ScrollEvent,
};
use crate::domain::viewport::Viewport;

/// Estado de un gesto en curso. Las posiciones del puntero son absolutas
/// dentro del área de plot, no deltas acumulados entre frames.
#[derive(Debug, Clone, Copy)]
enum GestureState {
    Idle,
    Pan {
        button: PointerButton,
        anchor_offset_x: f64,
        anchor_offset_y: f64,
        anchor_zoom_y: f64,
        anchor_pointer_x: f64,
        anchor_pointer_y: f64,
    },
    Zoom {
        button: PointerButton,
        anchor_zoom_x: f64,
        anchor_zoom_y: f64,
        anchor_offset_x: f64,
        anchor_offset_y: f64,
        anchor_pointer_x: f64,
        anchor_pointer_y: f64,
    },
}

/// Controlador de interacción sin propiedad del estado del gráfico.
///
/// Conserva el estado de los gestos y las dimensiones del plot. Cada evento
/// modifica el Viewport recibido por referencia, que pertenece a ChartState.
pub struct InteractionController {
    state: GestureState,
    screen_width: f64,
    screen_height: f64,
}

impl InteractionController {
    /// Crea un controlador para las dimensiones iniciales del área de plot.
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        Self {
            state: GestureState::Idle,
            screen_width: valid_dimension(screen_width),
            screen_height: valid_dimension(screen_height),
        }
    }

    /// Actualiza las dimensiones del plot.
    ///
    /// Un resize cancela cualquier gesto activo porque sus anclas se
    /// expresaron en el sistema de coordenadas anterior.
    pub fn set_screen_size(&mut self, width: f64, height: f64) {
        let width = valid_dimension(width);
        let height = valid_dimension(height);
        if width != self.screen_width || height != self.screen_height {
            self.state = GestureState::Idle;
        }
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Indica si hay un gesto de pan o zoom activo.
    pub fn is_gesture_active(&self) -> bool {
        !matches!(self.state, GestureState::Idle)
    }

    /// Procesa un evento sobre el Viewport propietario del ChartState.
    pub fn on_event(
        &mut self,
        viewport: &mut Viewport,
        candle_count: usize,
        event: &InteractionEvent,
    ) -> InteractionResult {
        match event {
            InteractionEvent::PointerMove(ev) => self.on_pointer_move(ev),
            InteractionEvent::PointerDown(ev) => self.on_pointer_down(viewport, ev),
            InteractionEvent::PointerUp(ev) => self.on_pointer_up(ev),
            InteractionEvent::PointerDrag(ev) => {
                self.on_pointer_drag(viewport, candle_count, ev)
            }
            InteractionEvent::Scroll(ev) => self.on_scroll(viewport, candle_count, ev),
        }
    }

    fn on_pointer_move(&self, ev: &PointerMoveEvent) -> InteractionResult {
        InteractionResult {
            viewport_changed: false,
            crosshair_changed: true,
            crosshair_pos: Some((ev.x, ev.y)),
        }
    }

    fn on_pointer_down(
        &mut self,
        viewport: &Viewport,
        ev: &PointerDownEvent,
    ) -> InteractionResult {
        if !ev.x.is_finite() || !ev.y.is_finite() {
            return InteractionResult::default();
        }

        match ev.button {
            PointerButton::Primary => {
                self.state = GestureState::Pan {
                    button: ev.button,
                    anchor_offset_x: viewport.offset_x(),
                    anchor_offset_y: viewport.offset_y(),
                    anchor_zoom_y: viewport.zoom_y(),
                    anchor_pointer_x: ev.x,
                    anchor_pointer_y: ev.y,
                };
            }
            PointerButton::Secondary => {
                self.state = GestureState::Zoom {
                    button: ev.button,
                    anchor_zoom_x: viewport.zoom_x(),
                    anchor_zoom_y: viewport.zoom_y(),
                    anchor_offset_x: viewport.offset_x(),
                    anchor_offset_y: viewport.offset_y(),
                    anchor_pointer_x: ev.x,
                    anchor_pointer_y: ev.y,
                };
            }
            PointerButton::Middle | PointerButton::Other(_) => {
                return InteractionResult::default();
            }
        }

        InteractionResult::default()
    }

    fn on_pointer_up(&mut self, ev: &PointerUpEvent) -> InteractionResult {
        let active_button = match self.state {
            GestureState::Idle => None,
            GestureState::Pan { button, .. } | GestureState::Zoom { button, .. } => Some(button),
        };

        if active_button == Some(ev.button) {
            self.state = GestureState::Idle;
        }
        InteractionResult::default()
    }

    fn on_pointer_drag(
        &mut self,
        viewport: &mut Viewport,
        candle_count: usize,
        ev: &PointerDragEvent,
    ) -> InteractionResult {
        if !ev.x.is_finite() || !ev.y.is_finite() {
            return InteractionResult::default();
        }

        match self.state {
            GestureState::Idle => InteractionResult::default(),
            GestureState::Pan {
                anchor_offset_x,
                anchor_offset_y,
                anchor_zoom_y,
                anchor_pointer_x,
                anchor_pointer_y,
                ..
            } => {
                if self.screen_width <= 0.0 || self.screen_height <= 0.0 {
                    return InteractionResult::default();
                }

                let mut next = *viewport;
                let spacing = next.candle_spacing();
                if spacing <= 0.0 || !spacing.is_finite() || anchor_zoom_y <= 0.0 {
                    return InteractionResult::default();
                }

                let delta_x = ev.x - anchor_pointer_x;
                let delta_y = ev.y - anchor_pointer_y;
                next.set_offset_x(anchor_offset_x + delta_x / spacing);
                next.set_offset_y(
                    anchor_offset_y - delta_y / (self.screen_height * anchor_zoom_y),
                );
                next.clamp_pan_to_candles(candle_count, self.screen_width);

                let changed = next != *viewport;
                if changed {
                    *viewport = next;
                }

                InteractionResult {
                    viewport_changed: changed,
                    crosshair_changed: true,
                    crosshair_pos: Some((ev.x, ev.y)),
                }
            }
            GestureState::Zoom {
                anchor_zoom_x,
                anchor_zoom_y,
                anchor_offset_x,
                anchor_offset_y,
                anchor_pointer_x,
                anchor_pointer_y,
                ..
            } => {
                if self.screen_width <= 0.0
                    || self.screen_height <= 0.0
                    || anchor_zoom_x <= 0.0
                    || anchor_zoom_y <= 0.0
                {
                    return InteractionResult::default();
                }

                let step = viewport.zoom_step();
                let base = 1.0 + step;
                if !base.is_finite() || base <= 0.0 {
                    return InteractionResult::default();
                }

                // 40 px de arrastre vertical equivalen a un paso de zoom.
                // La magnitud es continua y se calcula siempre desde el ancla.
                let delta_y = ev.y - anchor_pointer_y;
                let factor = base.powf(-delta_y / 40.0);
                if !factor.is_finite() || factor <= 0.0 {
                    return InteractionResult::default();
                }

                let mut next = *viewport;
                next.set_zoom(anchor_zoom_x, anchor_zoom_y);
                next.set_offset_x(anchor_offset_x);
                next.set_offset_y(anchor_offset_y);

                let changed = next.zoom_to_at(
                    anchor_zoom_x * factor,
                    anchor_zoom_y * factor,
                    anchor_pointer_x / self.screen_width,
                    anchor_pointer_y / self.screen_height,
                    self.screen_width,
                    self.screen_height,
                );

                if changed {
                    next.clamp_zoom_to_candles(candle_count, self.screen_width);
                    next.clamp_pan_to_candles(candle_count, self.screen_width);
                    *viewport = next;
                }

                InteractionResult {
                    viewport_changed: changed,
                    crosshair_changed: true,
                    crosshair_pos: Some((ev.x, ev.y)),
                }
            }
        }
    }

    fn on_scroll(
        &mut self,
        viewport: &mut Viewport,
        candle_count: usize,
        ev: &ScrollEvent,
    ) -> InteractionResult {
        if !ev.delta_y.is_finite()
            || ev.delta_y == 0.0
            || !ev.x.is_finite()
            || !ev.y.is_finite()
            || self.screen_width <= 0.0
            || self.screen_height <= 0.0
        {
            return InteractionResult::default();
        }

        let base = 1.0 + viewport.zoom_step();
        if !base.is_finite() || base <= 0.0 {
            return InteractionResult::default();
        }

        // El scroll de egui es una magnitud de píxeles. Escalar por 100 px
        // mantiene el zoom suave en trackpads y ruedas con distintos pasos.
        let factor = base.powf(ev.delta_y / 100.0);
        if !factor.is_finite() || factor <= 0.0 {
            return InteractionResult::default();
        }

        let mut next = *viewport;
        let changed = next.zoom_to_at(
            next.zoom_x(),
            next.zoom_y() * factor,
            ev.x / self.screen_width,
            ev.y / self.screen_height,
            self.screen_width,
            self.screen_height,
        );

        if changed {
            next.clamp_zoom_to_candles(candle_count, self.screen_width);
            next.clamp_pan_to_candles(candle_count, self.screen_width);
            *viewport = next;
        }

        InteractionResult {
            viewport_changed: changed,
            crosshair_changed: true,
            crosshair_pos: Some((ev.x, ev.y)),
        }
    }

    #[cfg(test)]
    fn gesture_state(&self) -> GestureState {
        self.state
    }
}

fn valid_dimension(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::interaction::events::{
        InteractionEvent, Modifiers, PointerDownEvent, PointerDragEvent, PointerUpEvent,
        ScrollEvent,
    };
    use crate::domain::transforms;

    fn make_viewport() -> Viewport {
        let mut viewport = Viewport::new();
        viewport.set_zoom(2.0, 2.0);
        viewport
    }

    fn pointer_down(button: PointerButton, x: f64, y: f64) -> InteractionEvent {
        InteractionEvent::PointerDown(PointerDownEvent {
            button,
            x,
            y,
            modifiers: Modifiers::default(),
        })
    }

    fn pointer_drag(x: f64, y: f64) -> InteractionEvent {
        InteractionEvent::PointerDrag(PointerDragEvent { x, y })
    }

    #[test]
    fn scroll_mutates_the_shared_viewport() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let zoom_before = viewport.zoom_y();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_y: 100.0,
                x: 960.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert!(viewport.zoom_y() > zoom_before);
    }

    #[test]
    fn pan_uses_absolute_positions_without_accumulating_frame_deltas() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let initial_offset = viewport.offset_x();

        controller.on_event(&mut viewport, 100, &pointer_down(PointerButton::Primary, 100.0, 100.0));
        controller.on_event(&mut viewport, 100, &pointer_drag(150.0, 100.0));
        let offset_at_150 = viewport.offset_x();

        controller.on_event(&mut viewport, 100, &pointer_drag(175.0, 100.0));
        controller.on_event(&mut viewport, 100, &pointer_drag(150.0, 100.0));

        assert!((viewport.offset_x() - offset_at_150).abs() < f64::EPSILON);
        assert!(offset_at_150 > initial_offset);
    }

    #[test]
    fn pointer_up_only_finishes_the_matching_gesture() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);

        controller.on_event(&mut viewport, 100, &pointer_down(PointerButton::Primary, 100.0, 100.0));
        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::PointerUp(PointerUpEvent {
                button: PointerButton::Secondary,
            }),
        );
        assert!(controller.is_gesture_active());

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::PointerUp(PointerUpEvent {
                button: PointerButton::Primary,
            }),
        );
        assert!(!controller.is_gesture_active());
        assert!(matches!(controller.gesture_state(), GestureState::Idle));
    }

    #[test]
    fn zoom_drag_is_continuous_and_anchored_to_the_start_position() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let anchor_x = 800.0;
        let anchor_y = 420.0;
        let data_x_before = (anchor_x - 960.0) / viewport.candle_spacing() - viewport.offset_x();
        let price_before = transforms::screen_y_to_price(
            anchor_y,
            1080.0,
            1000.0,
            viewport.zoom_y(),
            viewport.offset_y(),
            50000.0,
        );

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Secondary, anchor_x, anchor_y),
        );
        controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, anchor_y - 40.0));
        let zoom_one_step = viewport.zoom_y();
        controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, anchor_y - 80.0));
        let zoom_two_steps = viewport.zoom_y();

        assert!(zoom_two_steps > zoom_one_step);
        let data_x_after =
            (anchor_x - 960.0) / viewport.candle_spacing() - viewport.offset_x();
        let price_after = transforms::screen_y_to_price(
            anchor_y,
            1080.0,
            1000.0,
            viewport.zoom_y(),
            viewport.offset_y(),
            50000.0,
        );
        assert!((data_x_before - data_x_after).abs() < 1e-9);
        assert!((price_before - price_after).abs() < 1e-9);
    }

    #[test]
    fn resize_cancels_an_active_gesture() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        controller.on_event(&mut viewport, 100, &pointer_down(PointerButton::Primary, 100.0, 100.0));
        assert!(controller.is_gesture_active());

        controller.set_screen_size(1280.0, 720.0);

        assert!(!controller.is_gesture_active());
    }
}
