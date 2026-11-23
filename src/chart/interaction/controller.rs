//! Controlador de interacción del chart.
//!
//! El controlador conserva únicamente el estado transitorio de los gestos.
//! El Viewport es prestado por ChartState en cada evento y sigue siendo la
//! única fuente de verdad para zoom y pan.

use crate::chart::interaction::events::{
    AxisDoubleClickEvent, AxisPointerDownEvent, InteractionEvent, InteractionResult, PointerButton,
    PointerDownEvent, PointerDragEvent, PointerMoveEvent, PointerUpEvent, ScaleAxis, ScrollEvent,
};
use crate::chart::interaction::options::InteractionOptions;
use crate::domain::viewport::Viewport;

/// Estado de un gesto en curso. Las posiciones del puntero son absolutas
/// en coordenadas relativas al plot, incluso cuando el inicio está sobre un eje.
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
    AxisScale {
        button: PointerButton,
        axis: ScaleAxis,
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
    options: InteractionOptions,
}

impl InteractionController {
    /// Crea un controlador para las dimensiones iniciales del área de plot.
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        Self::with_options(screen_width, screen_height, InteractionOptions::default())
    }

    /// Crea un controlador con una política explícita de interacción.
    pub fn with_options(
        screen_width: f64,
        screen_height: f64,
        options: InteractionOptions,
    ) -> Self {
        Self {
            state: GestureState::Idle,
            screen_width: valid_dimension(screen_width),
            screen_height: valid_dimension(screen_height),
            options,
        }
    }

    /// Devuelve una copia de las opciones activas.
    pub fn options(&self) -> InteractionOptions {
        self.options
    }

    /// Reemplaza las opciones activas y cancela cualquier gesto en curso.
    ///
    /// Cancelar el gesto evita que un ancla creada con una política anterior
    /// siga modificando el viewport después de cambiar la configuración.
    pub fn set_options(&mut self, options: InteractionOptions) {
        if self.options != options {
            self.options = options;
            self.state = GestureState::Idle;
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

    /// Indica si hay un gesto de pan o zoom activo, incluido el arrastre de ejes.
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
            InteractionEvent::AxisPointerDown(ev) => self.on_axis_pointer_down(viewport, ev),
            InteractionEvent::AxisDoubleClick(ev) => {
                self.on_axis_double_click(viewport, candle_count, ev)
            }
            InteractionEvent::PointerUp(ev) => self.on_pointer_up(ev),
            InteractionEvent::PointerDrag(ev) => self.on_pointer_drag(viewport, candle_count, ev),
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

    fn on_pointer_down(&mut self, viewport: &Viewport, ev: &PointerDownEvent) -> InteractionResult {
        if !ev.x.is_finite() || !ev.y.is_finite() {
            return InteractionResult::default();
        }

        match ev.button {
            PointerButton::Primary if self.options.scroll.primary_mouse_drag => {
                self.state = GestureState::Pan {
                    button: ev.button,
                    anchor_offset_x: viewport.offset_x(),
                    anchor_offset_y: viewport.offset_y(),
                    anchor_zoom_y: viewport.zoom_y(),
                    anchor_pointer_x: ev.x,
                    anchor_pointer_y: ev.y,
                };
            }
            PointerButton::Secondary if self.options.scale.secondary_mouse_drag => {
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
            PointerButton::Primary
            | PointerButton::Secondary
            | PointerButton::Middle
            | PointerButton::Other(_) => {
                return InteractionResult::default();
            }
        }

        InteractionResult::default()
    }

    fn on_axis_pointer_down(
        &mut self,
        viewport: &Viewport,
        ev: &AxisPointerDownEvent,
    ) -> InteractionResult {
        if !ev.x.is_finite() || !ev.y.is_finite() {
            return InteractionResult::default();
        }

        let enabled = match ev.axis {
            ScaleAxis::Time => self.options.scale.axis_pressed_mouse_move.time,
            ScaleAxis::Price => self.options.scale.axis_pressed_mouse_move.price,
        };
        if !enabled {
            return InteractionResult::default();
        }

        self.state = GestureState::AxisScale {
            button: PointerButton::Primary,
            axis: ev.axis,
            anchor_zoom_x: viewport.zoom_x(),
            anchor_zoom_y: viewport.zoom_y(),
            anchor_offset_x: viewport.offset_x(),
            anchor_offset_y: viewport.offset_y(),
            anchor_pointer_x: ev.x,
            anchor_pointer_y: ev.y,
        };
        InteractionResult::default()
    }

    fn on_axis_double_click(
        &self,
        viewport: &mut Viewport,
        candle_count: usize,
        ev: &AxisDoubleClickEvent,
    ) -> InteractionResult {
        let enabled = match ev.axis {
            ScaleAxis::Time => self.options.scale.axis_double_click_reset.time,
            ScaleAxis::Price => self.options.scale.axis_double_click_reset.price,
        };
        if !enabled || self.screen_width <= 0.0 || self.screen_height <= 0.0 {
            return InteractionResult::default();
        }

        let mut next = *viewport;
        match ev.axis {
            ScaleAxis::Time => {
                next.fit_content(candle_count, self.screen_width);
            }
            ScaleAxis::Price => {
                next.set_zoom_y(1.0);
                next.set_offset_y(0.0);
            }
        }

        let changed = next != *viewport;
        if changed {
            *viewport = next;
        }
        InteractionResult {
            viewport_changed: changed,
            crosshair_changed: false,
            crosshair_pos: None,
        }
    }

    fn on_pointer_up(&mut self, ev: &PointerUpEvent) -> InteractionResult {
        let active_button = match self.state {
            GestureState::Idle => None,
            GestureState::Pan { button, .. }
            | GestureState::Zoom { button, .. }
            | GestureState::AxisScale { button, .. } => Some(button),
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
                let vertical_direction = if self.options.scroll.invert_vertical_pan {
                    1.0
                } else {
                    -1.0
                };
                next.set_offset_x(anchor_offset_x + delta_x / spacing);
                next.set_offset_y(
                    anchor_offset_y
                        + vertical_direction * delta_y / (self.screen_height * anchor_zoom_y),
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

                next.zoom_to_at(
                    anchor_zoom_x * factor,
                    anchor_zoom_y * factor,
                    anchor_pointer_x / self.screen_width,
                    anchor_pointer_y / self.screen_height,
                    self.screen_width,
                    self.screen_height,
                );

                // Aunque el nivel calculado vuelva al zoom inicial, next puede
                // diferir del viewport actual por un frame anterior del gesto.
                next.clamp_zoom_to_candles(candle_count, self.screen_width);
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
            GestureState::AxisScale {
                axis,
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

                let (delta, center_x, center_y) = match axis {
                    ScaleAxis::Time => (
                        ev.y - anchor_pointer_y,
                        anchor_pointer_x / self.screen_width,
                        0.5,
                    ),
                    ScaleAxis::Price => {
                        // Arrastrar hacia el interior del gráfico acerca la escala
                        // tanto desde el eje izquierdo como desde el derecho.
                        let direction = if anchor_pointer_x < 0.0 { -1.0 } else { 1.0 };
                        (
                            (ev.x - anchor_pointer_x) * direction,
                            0.5,
                            anchor_pointer_y / self.screen_height,
                        )
                    }
                };
                let base = 1.0 + viewport.zoom_step();
                let factor = base.powf(-delta / 40.0);
                if !base.is_finite() || base <= 0.0 || !factor.is_finite() || factor <= 0.0 {
                    return InteractionResult::default();
                }

                let (zoom_x, zoom_y) = match axis {
                    ScaleAxis::Time => (anchor_zoom_x * factor, anchor_zoom_y),
                    ScaleAxis::Price => (anchor_zoom_x, anchor_zoom_y * factor),
                };

                let mut next = *viewport;
                next.set_zoom(anchor_zoom_x, anchor_zoom_y);
                next.set_offset_x(anchor_offset_x);
                next.set_offset_y(anchor_offset_y);
                next.zoom_to_at(
                    zoom_x,
                    zoom_y,
                    center_x,
                    center_y,
                    self.screen_width,
                    self.screen_height,
                );

                if matches!(axis, ScaleAxis::Time) {
                    next.clamp_zoom_to_candles(candle_count, self.screen_width);
                    next.clamp_pan_to_candles(candle_count, self.screen_width);
                }

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
        }
    }

    fn on_scroll(
        &mut self,
        viewport: &mut Viewport,
        candle_count: usize,
        ev: &ScrollEvent,
    ) -> InteractionResult {
        if !ev.delta_x.is_finite()
            || !ev.delta_y.is_finite()
            || !ev.x.is_finite()
            || !ev.y.is_finite()
            || self.screen_width <= 0.0
            || self.screen_height <= 0.0
        {
            return InteractionResult::default();
        }

        // Cada eje de la rueda se filtra de forma independiente para que
        // scroll y scale puedan configurarse sin acoplar sus comportamientos.
        let horizontal_delta = if self.options.scroll.horizontal_mouse_wheel {
            ev.delta_x
        } else {
            0.0
        };
        let vertical_delta = if self.options.scale.vertical_mouse_wheel {
            ev.delta_y
        } else {
            0.0
        };
        if horizontal_delta == 0.0 && vertical_delta == 0.0 {
            return InteractionResult::default();
        }

        let mut next = *viewport;

        // La rueda vertical modifica el zoom temporal y mantiene anclada
        // la coordenada lógica situada bajo el cursor.
        if vertical_delta != 0.0 {
            let base = 1.0 + viewport.zoom_step();
            if base.is_finite() && base > 0.0 {
                // El delta de egui se expresa en píxeles; 100 px equivalen
                // aproximadamente a un paso completo de zoom.
                let factor = base.powf(vertical_delta / 100.0);
                if factor.is_finite() && factor > 0.0 {
                    next.zoom_to_at(
                        next.zoom_x() * factor,
                        next.zoom_y(),
                        ev.x / self.screen_width,
                        ev.y / self.screen_height,
                        self.screen_width,
                        self.screen_height,
                    );
                }
            }
        }

        // La rueda horizontal desplaza el historial sin alterar las escalas.
        if horizontal_delta != 0.0 {
            next.pan(-horizontal_delta, 0.0, self.screen_height);
        }

        next.clamp_zoom_to_candles(candle_count, self.screen_width);
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
        AxisDoubleClickEvent, AxisPointerDownEvent, InteractionEvent, Modifiers, PointerDownEvent,
        PointerDragEvent, PointerUpEvent, ScaleAxis, ScrollEvent,
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
    fn vertical_scroll_zooms_time_axis_around_the_pointer() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let zoom_x_before = viewport.zoom_x();
        let zoom_y_before = viewport.zoom_y();
        let anchor_x = 700.0;
        let logical_under_cursor = viewport.screen_to_logical(anchor_x, 1920.0);

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 0.0,
                delta_y: 100.0,
                x: anchor_x,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert!(viewport.zoom_x() > zoom_x_before);
        assert_eq!(viewport.zoom_y(), zoom_y_before);
        assert!(
            (viewport.screen_to_logical(anchor_x, 1920.0) - logical_under_cursor).abs() < 1e-9,
            "el zoom temporal debe conservar la coordenada lógica bajo el cursor"
        );
    }

    #[test]
    fn horizontal_scroll_pans_history_without_changing_zoom() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let initial_offset = viewport.offset_x();
        let initial_zoom_x = viewport.zoom_x();
        let initial_zoom_y = viewport.zoom_y();

        let result = controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 120.0,
                delta_y: 0.0,
                x: 960.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert!(result.viewport_changed);
        assert!(viewport.offset_x() < initial_offset);
        assert_eq!(viewport.zoom_x(), initial_zoom_x);
        assert_eq!(viewport.zoom_y(), initial_zoom_y);
    }

    #[test]
    fn diagonal_scroll_combines_time_zoom_and_horizontal_pan() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let initial_offset = viewport.offset_x();
        let initial_zoom_x = viewport.zoom_x();
        let initial_zoom_y = viewport.zoom_y();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 80.0,
                delta_y: 100.0,
                x: 700.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert!(viewport.zoom_x() > initial_zoom_x);
        assert_eq!(viewport.zoom_y(), initial_zoom_y);
        assert!(viewport.offset_x() < initial_offset);
    }

    #[test]
    fn scroll_and_scale_wheel_options_are_independent() {
        let mut viewport = make_viewport();
        let initial = viewport;
        let mut options = InteractionOptions::default();
        options.scroll.horizontal_mouse_wheel = false;
        let mut controller = InteractionController::with_options(1920.0, 1080.0, options);

        let mut zoom_only = initial;
        let mut zoom_only_controller = InteractionController::with_options(
            1920.0,
            1080.0,
            InteractionOptions {
                scroll: crate::chart::interaction::options::ScrollOptions {
                    horizontal_mouse_wheel: false,
                    primary_mouse_drag: true,
                    ..crate::chart::interaction::options::ScrollOptions::default()
                },
                scale: crate::chart::interaction::options::ScaleOptions {
                    vertical_mouse_wheel: true,
                    secondary_mouse_drag: true,
                    ..crate::chart::interaction::options::ScaleOptions::default()
                },
            },
        );
        zoom_only_controller.on_event(
            &mut zoom_only,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 0.0,
                delta_y: 100.0,
                x: 700.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );
        let zoom_before = initial.zoom_x();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 120.0,
                delta_y: 100.0,
                x: 700.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(
            viewport, zoom_only,
            "la opción desactivada no debe añadir pan"
        );
        assert!(viewport.zoom_x() > zoom_before);

        options.scroll.horizontal_mouse_wheel = true;
        options.scale.vertical_mouse_wheel = false;
        controller.set_options(options);
        let offset_before = viewport.offset_x();
        let zoom_before = viewport.zoom_x();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::Scroll(ScrollEvent {
                delta_x: 120.0,
                delta_y: 100.0,
                x: 700.0,
                y: 324.0,
                modifiers: Modifiers::default(),
            }),
        );

        assert!(viewport.offset_x() < offset_before);
        assert_eq!(viewport.zoom_x(), zoom_before);
    }

    #[test]
    fn primary_pan_and_secondary_scale_drags_can_be_disabled_independently() {
        let mut viewport = make_viewport();
        let initial = viewport;
        let mut options = InteractionOptions::default();
        options.scroll.primary_mouse_drag = false;
        let mut controller = InteractionController::with_options(1920.0, 1080.0, options);

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 100.0, 100.0),
        );
        assert!(!controller.is_gesture_active());
        controller.on_event(&mut viewport, 100, &pointer_drag(150.0, 100.0));
        assert_eq!(viewport, initial);

        options.scroll.primary_mouse_drag = true;
        options.scale.secondary_mouse_drag = false;
        controller.set_options(options);
        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Secondary, 800.0, 420.0),
        );
        assert!(!controller.is_gesture_active());
        controller.on_event(&mut viewport, 100, &pointer_drag(800.0, 380.0));
        assert_eq!(viewport, initial);
    }

    #[test]
    fn changing_options_cancels_an_active_gesture() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 100.0, 100.0),
        );
        assert!(controller.is_gesture_active());

        let mut options = controller.options();
        options.scroll.primary_mouse_drag = false;
        controller.set_options(options);

        assert!(!controller.is_gesture_active());
        controller.on_event(&mut viewport, 100, &pointer_drag(150.0, 100.0));
        assert_eq!(viewport.offset_x(), 0.0);
    }

    #[test]
    fn time_axis_drag_zooms_horizontally_around_the_anchor() {
        let mut viewport = make_viewport();
        viewport.fit_content(100, 1_000.0);
        viewport.set_zoom_y(3.0);
        viewport.set_offset_y(-0.25);
        let mut controller = InteractionController::new(1_000.0, 600.0);
        let anchor_x = 320.0;
        let logical_before = viewport.screen_to_logical(anchor_x, 1_000.0);
        let zoom_x_before = viewport.zoom_x();
        let zoom_y_before = viewport.zoom_y();
        let offset_y_before = viewport.offset_y();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::AxisPointerDown(AxisPointerDownEvent {
                axis: ScaleAxis::Time,
                x: anchor_x,
                y: 610.0,
            }),
        );
        let result = controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, 570.0));

        assert!(result.viewport_changed);
        assert!(viewport.zoom_x() > zoom_x_before);
        assert_eq!(viewport.zoom_y(), zoom_y_before);
        assert_eq!(viewport.offset_y(), offset_y_before);
        assert!(
            (viewport.screen_to_logical(anchor_x, 1_000.0) - logical_before).abs() < 1e-9,
            "el arrastre temporal debe conservar la coordenada lógica anclada"
        );
    }

    #[test]
    fn price_axis_drag_zooms_vertically_without_changing_time_scale() {
        let mut viewport = make_viewport();
        viewport.set_zoom_y(2.0);
        viewport.set_offset_y(-0.35);
        let mut controller = InteractionController::new(1_000.0, 600.0);
        let anchor_x = -30.0;
        let anchor_y = 210.0;
        let zoom_x_before = viewport.zoom_x();
        let offset_x_before = viewport.offset_x();
        let scaled_price_before =
            (300.0 - anchor_y) / (600.0 * viewport.zoom_y()) + viewport.offset_y();

        controller.on_event(
            &mut viewport,
            100,
            &InteractionEvent::AxisPointerDown(AxisPointerDownEvent {
                axis: ScaleAxis::Price,
                x: anchor_x,
                y: anchor_y,
            }),
        );
        let result =
            controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x + 40.0, anchor_y));
        let scaled_price_after =
            (300.0 - anchor_y) / (600.0 * viewport.zoom_y()) + viewport.offset_y();

        assert!(result.viewport_changed);
        assert!(viewport.zoom_y() > 2.0);
        assert_eq!(viewport.zoom_x(), zoom_x_before);
        assert_eq!(viewport.offset_x(), offset_x_before);
        assert!((scaled_price_after - scaled_price_before).abs() < 1e-9);
    }

    #[test]
    fn time_axis_double_click_fits_history_and_preserves_price_scale() {
        let mut viewport = make_viewport();
        viewport.set_zoom(2.0, 3.0);
        viewport.set_offset_x(-8.0);
        viewport.set_offset_y(0.4);
        let mut controller = InteractionController::new(1_000.0, 600.0);
        let zoom_y_before = viewport.zoom_y();
        let offset_y_before = viewport.offset_y();

        let result = controller.on_event(
            &mut viewport,
            50,
            &InteractionEvent::AxisDoubleClick(AxisDoubleClickEvent {
                axis: ScaleAxis::Time,
            }),
        );

        assert!(result.viewport_changed);
        assert_eq!(viewport.zoom_y(), zoom_y_before);
        assert_eq!(viewport.offset_y(), offset_y_before);
        assert!((viewport.logical_to_screen(0.0, 1_000.0) - 50.0).abs() < 1e-8);
        assert!((viewport.logical_to_screen(49.0, 1_000.0) - 950.0).abs() < 1e-8);
    }

    #[test]
    fn price_axis_double_click_resets_only_vertical_scale() {
        let mut viewport = make_viewport();
        viewport.set_zoom(2.5, 3.5);
        viewport.set_offset_x(-12.0);
        viewport.set_offset_y(0.75);
        let mut controller = InteractionController::new(1_000.0, 600.0);
        let zoom_x_before = viewport.zoom_x();
        let offset_x_before = viewport.offset_x();

        let result = controller.on_event(
            &mut viewport,
            50,
            &InteractionEvent::AxisDoubleClick(AxisDoubleClickEvent {
                axis: ScaleAxis::Price,
            }),
        );

        assert!(result.viewport_changed);
        assert_eq!(viewport.zoom_x(), zoom_x_before);
        assert_eq!(viewport.offset_x(), offset_x_before);
        assert_eq!(viewport.zoom_y(), 1.0);
        assert_eq!(viewport.offset_y(), 0.0);
    }

    #[test]
    fn axis_interaction_options_are_independent() {
        let mut viewport = make_viewport();
        let initial = viewport;
        let mut options = InteractionOptions::default();
        options.scale.axis_pressed_mouse_move.time = false;
        options.scale.axis_double_click_reset.price = false;
        let mut controller = InteractionController::with_options(1_000.0, 600.0, options);

        controller.on_event(
            &mut viewport,
            50,
            &InteractionEvent::AxisPointerDown(AxisPointerDownEvent {
                axis: ScaleAxis::Time,
                x: 400.0,
                y: 610.0,
            }),
        );
        assert!(!controller.is_gesture_active());
        controller.on_event(&mut viewport, 50, &pointer_drag(400.0, 570.0));
        assert_eq!(viewport, initial);

        viewport.set_zoom_y(3.0);
        viewport.set_offset_y(0.5);
        let before_disabled_reset = viewport;
        controller.on_event(
            &mut viewport,
            50,
            &InteractionEvent::AxisDoubleClick(AxisDoubleClickEvent {
                axis: ScaleAxis::Price,
            }),
        );
        assert_eq!(viewport, before_disabled_reset);
    }

    #[test]
    fn vertical_pan_follows_pointer_direction_by_default() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1_000.0, 600.0);

        assert!(controller.options().scroll.invert_vertical_pan);
        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 500.0, 300.0),
        );
        controller.on_event(&mut viewport, 100, &pointer_drag(500.0, 350.0));

        assert!(viewport.offset_y() > 0.0);
        assert_eq!(viewport.offset_x(), 0.0);
    }

    #[test]
    fn vertical_pan_direction_can_be_restored_to_previous_behavior() {
        let mut viewport = make_viewport();
        let mut options = InteractionOptions::default();
        options.scroll.invert_vertical_pan = false;
        let mut controller = InteractionController::with_options(1_000.0, 600.0, options);

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 500.0, 300.0),
        );
        controller.on_event(&mut viewport, 100, &pointer_drag(500.0, 350.0));

        assert!(viewport.offset_y() < 0.0);
        assert_eq!(viewport.offset_x(), 0.0);
    }

    #[test]
    fn pan_uses_absolute_positions_without_accumulating_frame_deltas() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let initial_offset = viewport.offset_x();

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 100.0, 100.0),
        );
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

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 100.0, 100.0),
        );
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
        let data_x_after = (anchor_x - 960.0) / viewport.candle_spacing() - viewport.offset_x();
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
    fn zoom_drag_returns_to_its_initial_state_when_pointer_returns_to_anchor() {
        let mut viewport = make_viewport();
        let initial = viewport;
        let mut controller = InteractionController::new(1920.0, 1080.0);
        let anchor_x = 800.0;
        let anchor_y = 420.0;

        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Secondary, anchor_x, anchor_y),
        );
        controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, anchor_y - 40.0));
        assert!(viewport.zoom_y() > initial.zoom_y());

        controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, anchor_y - 80.0));
        controller.on_event(&mut viewport, 100, &pointer_drag(anchor_x, anchor_y));

        assert_eq!(viewport.zoom_x(), initial.zoom_x());
        assert_eq!(viewport.zoom_y(), initial.zoom_y());
        assert!((viewport.offset_x() - initial.offset_x()).abs() < 1e-9);
        assert!((viewport.offset_y() - initial.offset_y()).abs() < 1e-9);
    }

    #[test]
    fn resize_cancels_an_active_gesture() {
        let mut viewport = make_viewport();
        let mut controller = InteractionController::new(1920.0, 1080.0);
        controller.on_event(
            &mut viewport,
            100,
            &pointer_down(PointerButton::Primary, 100.0, 100.0),
        );
        assert!(controller.is_gesture_active());

        controller.set_screen_size(1280.0, 720.0);

        assert!(!controller.is_gesture_active());
    }
}
