//! Controlador de interacción del chart.
//!
//! Gestiona el estado de gestos de pan y zoom con anclas capturadas al
//! inicio del gesto, evitando el drift acumulado de aplicar deltas
//! frame tras frame (inspirado en Lightweight Charts).

use crate::chart::interaction::events::{
    InteractionEvent, InteractionResult, PointerButton, PointerDownEvent, PointerDragEvent,
    PointerMoveEvent, PointerUpEvent, ScrollEvent,
};
use crate::domain::viewport::Viewport;

/// Estado de un gesto en curso.
#[derive(Debug, Clone, Copy)]
enum GestureState {
    /// Sin gesto activo.
    Idle,
    /// Pan horizontal/vertical en curso.
    Pan {
        /// Offset X del viewport al inicio del gesto.
        anchor_offset_x: f64,
        /// Offset Y del viewport al inicio del gesto.
        anchor_offset_y: f64,
        /// Posición X de pantalla donde inició el pan.
        anchor_screen_x: f64,
        /// Posición Y de pantalla donde inició el pan.
        anchor_screen_y: f64,
    },
    /// Zoom en curso (anchura de vela y desplazamiento centrado en el cursor).
    Zoom {
        /// Zoom X del viewport al inicio del gesto.
        anchor_zoom_x: f64,
        /// Zoom Y del viewport al inicio del gesto.
        anchor_zoom_y: f64,
        /// Offset X del viewport al inicio del gesto.
        anchor_offset_x: f64,
        /// Offset Y del viewport al inicio del gesto.
        anchor_offset_y: f64,
        /// Posición X de pantalla donde inició el zoom (ancla).
        anchor_screen_x: f64,
        /// Posición Y de pantalla donde inició el zoom (ancla).
        anchor_screen_y: f64,
    },
}

/// Controlador de interacción del chart.
///
/// Mantiene el estado de gestos y aplica operaciones al `Viewport`.
/// Los ejes X e Y se manejan de forma independiente.
pub struct InteractionController {
    state: GestureState,
    viewport: Viewport,
    /// Ancho del área de plot en píxeles (para conversiones pantalla→datos).
    screen_width: f64,
    /// Alto del área de plot en píxeles (para conversiones pantalla→datos).
    screen_height: f64,
}

impl InteractionController {
    /// Crea un controlador con el viewport y dimensiones de pantalla dadas.
    pub fn new(viewport: Viewport, screen_width: f64, screen_height: f64) -> Self {
        Self {
            state: GestureState::Idle,
            viewport,
            screen_width,
            screen_height,
        }
    }

    /// Actualiza las dimensiones del área de plot (llamado en resize).
    pub fn set_screen_size(&mut self, width: f64, height: f64) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Devuelve una referencia al viewport.
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Devuelve una referencia mutable al viewport.
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Devuelve el estado actual del gesto (para tests).
    fn gesture_state(&self) -> GestureState {
        self.state
    }

    /// Procesa un evento y devuelve si hubo cambios.
    pub fn on_event(&mut self, event: &InteractionEvent) -> InteractionResult {
        match event {
            InteractionEvent::PointerMove(ev) => self.on_pointer_move(ev),
            InteractionEvent::PointerDown(ev) => self.on_pointer_down(ev),
            InteractionEvent::PointerUp(ev) => self.on_pointer_up(ev),
            InteractionEvent::PointerDrag(ev) => self.on_pointer_drag(ev),
            InteractionEvent::Scroll(ev) => self.on_scroll(ev),
        }
    }

    fn on_pointer_move(&mut self, ev: &PointerMoveEvent) -> InteractionResult {
        InteractionResult {
            viewport_changed: false,
            crosshair_changed: true,
            crosshair_pos: Some((ev.x, ev.y)),
        }
    }

    fn on_pointer_down(&mut self, ev: &PointerDownEvent) -> InteractionResult {
        match ev.button {
            PointerButton::Primary => {
                self.state = GestureState::Pan {
                    anchor_offset_x: self.viewport.offset_x(),
                    anchor_offset_y: self.viewport.offset_y(),
                    anchor_screen_x: ev.x,
                    anchor_screen_y: ev.y,
                };
                InteractionResult {
                    viewport_changed: false,
                    crosshair_changed: false,
                    crosshair_pos: None,
                }
            }
            PointerButton::Secondary => {
                self.state = GestureState::Zoom {
                    anchor_zoom_x: self.viewport.zoom_x(),
                    anchor_zoom_y: self.viewport.zoom_y(),
                    anchor_offset_x: self.viewport.offset_x(),
                    anchor_offset_y: self.viewport.offset_y(),
                    anchor_screen_x: ev.x,
                    anchor_screen_y: ev.y,
                };
                InteractionResult {
                    viewport_changed: false,
                    crosshair_changed: false,
                    crosshair_pos: None,
                }
            }
            _ => InteractionResult::default(),
        }
    }

    fn on_pointer_up(&mut self, _ev: &PointerUpEvent) -> InteractionResult {
        let was_gesture = matches!(
            self.state,
            GestureState::Pan { .. } | GestureState::Zoom { .. }
        );
        self.state = GestureState::Idle;
        if was_gesture {
            InteractionResult {
                viewport_changed: true,
                crosshair_changed: false,
                crosshair_pos: None,
            }
        } else {
            InteractionResult::default()
        }
    }

    fn on_pointer_drag(&mut self, ev: &PointerDragEvent) -> InteractionResult {
        match self.state {
            GestureState::Pan {
                anchor_offset_x,
                anchor_offset_y,
                anchor_screen_x: _,
                anchor_screen_y: _,
            } => {
                let spacing = self.viewport.candle_spacing();
                if spacing > 0.0 && self.screen_height > 0.0 {
                    let dy_offset = ev.dy / (self.screen_height * self.viewport.zoom_y());
                    self.viewport
                        .set_offset_x(anchor_offset_x + ev.dx / spacing);
                    self.viewport.set_offset_y(anchor_offset_y - dy_offset);
                }
                InteractionResult {
                    viewport_changed: true,
                    crosshair_changed: true,
                    crosshair_pos: None,
                }
            }
            GestureState::Zoom {
                anchor_zoom_x,
                anchor_zoom_y,
                anchor_offset_x,
                anchor_offset_y,
                anchor_screen_x,
                anchor_screen_y,
            } => {
                let factor = if ev.dy < 0.0 {
                    1.0 + 0.01
                } else if ev.dy > 0.0 {
                    1.0 - 0.01
                } else {
                    1.0
                };

                if factor != 1.0 && self.screen_width > 0.0 && self.screen_height > 0.0 {
                    let new_zoom_x = (anchor_zoom_x * factor)
                        .clamp(self.viewport.min_zoom(), self.viewport.max_zoom());
                    let new_zoom_y = (anchor_zoom_y * factor)
                        .clamp(self.viewport.min_zoom(), self.viewport.max_zoom());

                    let center_x = anchor_screen_x / self.screen_width;
                    let center_y = anchor_screen_y / self.screen_height;

                    let spacing_old = 20.0 * anchor_zoom_x;
                    let center_x_px = center_x * self.screen_width;
                    let offset_x_px = anchor_offset_x * spacing_old;
                    let new_offset_x_px =
                        center_x_px + (offset_x_px - center_x_px) * (new_zoom_x / anchor_zoom_x);
                    let new_offset_x = new_offset_x_px / (20.0 * new_zoom_x);

                    let offset_y_px = anchor_offset_y * self.screen_height * anchor_zoom_y;
                    let center_y_px = center_y * self.screen_height;
                    let zoom_ratio_y = new_zoom_y / anchor_zoom_y;
                    let new_offset_y_px = center_y_px + (offset_y_px - center_y_px) * zoom_ratio_y;
                    let new_offset_y = new_offset_y_px / (self.screen_height * new_zoom_y);

                    self.viewport.set_zoom(new_zoom_x, new_zoom_y);
                    self.viewport.set_offset_x(new_offset_x);
                    self.viewport.set_offset_y(new_offset_y);
                }

                InteractionResult {
                    viewport_changed: true,
                    crosshair_changed: true,
                    crosshair_pos: None,
                }
            }
            GestureState::Idle => InteractionResult::default(),
        }
    }

    fn on_scroll(&mut self, ev: &ScrollEvent) -> InteractionResult {
        if ev.delta_y == 0.0 {
            return InteractionResult::default();
        }

        let factor = if ev.delta_y > 0.0 {
            1.0 + self.viewport.zoom_step()
        } else {
            1.0 - self.viewport.zoom_step()
        };

        let new_zoom_y = (self.viewport.zoom_y() * factor)
            .clamp(self.viewport.min_zoom(), self.viewport.max_zoom());

        if new_zoom_y != self.viewport.zoom_y() && self.screen_height > 0.0 {
            let center_y_frac = ev.y / self.screen_height;
            let center_y_px = center_y_frac * self.screen_height;

            let offset_y_px =
                self.viewport.offset_y() * self.screen_height * self.viewport.zoom_y();
            let zoom_ratio = new_zoom_y / self.viewport.zoom_y();
            let new_offset_y_px = center_y_px + (offset_y_px - center_y_px) * zoom_ratio;
            let new_offset_y = new_offset_y_px / (self.screen_height * new_zoom_y);

            self.viewport.set_zoom_y(new_zoom_y);
            self.viewport.set_offset_y(new_offset_y);
        }

        InteractionResult {
            viewport_changed: true,
            crosshair_changed: true,
            crosshair_pos: Some((ev.x, ev.y)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::interaction::events::{
        InteractionEvent, Modifiers, PointerDownEvent, PointerDragEvent, PointerUpEvent,
        ScrollEvent,
    };

    fn make_viewport() -> Viewport {
        let mut v = Viewport::new();
        v.set_zoom(2.0, 2.0);
        v
    }

    #[test]
    fn scroll_zoom_increases_zoom_y() {
        let viewport = make_viewport();
        let mut controller = InteractionController::new(viewport, 1920.0, 1080.0);

        let zoom_before = controller.viewport().zoom_y();

        controller.on_event(&InteractionEvent::Scroll(ScrollEvent {
            delta_y: 1.0,
            x: 960.0,
            y: 324.0,
            modifiers: Modifiers::default(),
        }));

        let zoom_after = controller.viewport().zoom_y();
        assert!(
            zoom_after > zoom_before,
            "scroll hacia adelante debe hacer zoom in: {} -> {}",
            zoom_before,
            zoom_after
        );
    }

    #[test]
    fn pan_changes_offset_x() {
        let viewport = make_viewport();
        let mut controller = InteractionController::new(viewport, 1920.0, 1080.0);

        let offset_before = controller.viewport().offset_x();

        controller.on_event(&InteractionEvent::PointerDown(PointerDownEvent {
            button: PointerButton::Primary,
            x: 100.0,
            y: 100.0,
            modifiers: Modifiers::default(),
        }));
        controller.on_event(&InteractionEvent::PointerDrag(PointerDragEvent {
            dx: 50.0,
            dy: 0.0,
        }));

        let offset_after = controller.viewport().offset_x();
        assert!(
            offset_after != offset_before,
            "el pan debe cambiar el offset_x: {} -> {}",
            offset_before,
            offset_after
        );
    }

    #[test]
    fn pointer_up_returns_to_idle() {
        let viewport = make_viewport();
        let mut controller = InteractionController::new(viewport, 1920.0, 1080.0);

        controller.on_event(&InteractionEvent::PointerDown(PointerDownEvent {
            button: PointerButton::Primary,
            x: 100.0,
            y: 100.0,
            modifiers: Modifiers::default(),
        }));
        assert!(matches!(
            controller.gesture_state(),
            GestureState::Pan { .. }
        ));

        controller.on_event(&InteractionEvent::PointerUp(PointerUpEvent {
            button: PointerButton::Primary,
        }));

        assert!(matches!(controller.gesture_state(), GestureState::Idle));
    }

    #[test]
    fn zoom_gesture_changes_zoom() {
        let viewport = make_viewport();
        let mut controller = InteractionController::new(viewport, 1920.0, 1080.0);

        let zoom_before = controller.viewport().zoom_y();

        controller.on_event(&InteractionEvent::PointerDown(PointerDownEvent {
            button: PointerButton::Secondary,
            x: 960.0,
            y: 540.0,
            modifiers: Modifiers::default(),
        }));
        controller.on_event(&InteractionEvent::PointerDrag(PointerDragEvent {
            dx: 0.0,
            dy: -50.0,
        }));

        let zoom_after = controller.viewport().zoom_y();
        assert!(
            zoom_after > zoom_before,
            "arrastre hacia arriba con botón secundario debe hacer zoom in"
        );
    }
}
