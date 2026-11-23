//! Eventos de interacción normalizados.
//!
//! El dominio no conoce `egui`: la capa de aplicación traduce eventos de
//! egui a estos tipos antes de pasarlos al `InteractionController`.

/// Botón del puntero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// Botón principal (izquierdo).
    Primary,
    /// Botón secundario (derecho).
    Secondary,
    /// Botón medio / rueda.
    Middle,
    /// Otro botón.
    Other(u8),
}

/// Eje que recibe una interacción directa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAxis {
    /// Eje temporal horizontal.
    Time,
    /// Eje de precios vertical.
    Price,
}

/// Teclas modificadoras activas durante un evento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Evento de movimiento del puntero.
#[derive(Debug, Clone, Copy)]
pub struct PointerMoveEvent {
    /// Posición en coordenadas del área de plot (píxeles, 0..=width).
    pub x: f64,
    pub y: f64,
}

/// Evento de pulsación de botón del puntero.
#[derive(Debug, Clone, Copy)]
pub struct PointerDownEvent {
    pub button: PointerButton,
    pub x: f64,
    pub y: f64,
    pub modifiers: Modifiers,
}

/// Inicio de arrastre sobre un eje.
#[derive(Debug, Clone, Copy)]
pub struct AxisPointerDownEvent {
    pub axis: ScaleAxis,
    /// Posición X relativa al área de plot.
    pub x: f64,
    /// Posición Y relativa al área de plot.
    pub y: f64,
}

/// Doble clic sobre un eje para restablecer su escala.
#[derive(Debug, Clone, Copy)]
pub struct AxisDoubleClickEvent {
    pub axis: ScaleAxis,
}

/// Evento de liberación de botón del puntero.
#[derive(Debug, Clone, Copy)]
pub struct PointerUpEvent {
    pub button: PointerButton,
}

/// Evento de arrastre con posición absoluta dentro del área de plot.
#[derive(Debug, Clone, Copy)]
pub struct PointerDragEvent {
    /// Posición X actual en píxeles del área de plot.
    pub x: f64,
    /// Posición Y actual en píxeles del área de plot.
    pub y: f64,
}

/// Evento de scroll (rueda del mouse).
#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent {
    /// Delta horizontal de scroll en píxeles; positivo desplaza el historial hacia la izquierda.
    pub delta_x: f64,
    /// Delta vertical de scroll en píxeles; positivo acerca el eje temporal.
    pub delta_y: f64,
    /// Posición del cursor en el área de plot.
    pub x: f64,
    pub y: f64,
    pub modifiers: Modifiers,
}

/// Evento de interacción unificado.
#[derive(Debug, Clone, Copy)]
pub enum InteractionEvent {
    /// El puntero se movió a una nueva posición.
    PointerMove(PointerMoveEvent),
    /// Se inició un arrastre sobre el área de plot.
    PointerDown(PointerDownEvent),
    /// Se inició un arrastre de escala desde uno de los ejes.
    AxisPointerDown(AxisPointerDownEvent),
    /// Doble clic en un eje para restablecer solo esa escala.
    AxisDoubleClick(AxisDoubleClickEvent),
    /// Se soltó un botón del puntero.
    PointerUp(PointerUpEvent),
    /// El puntero se arrastró (delta desde el frame anterior).
    PointerDrag(PointerDragEvent),
    /// Evento de scroll (rueda del mouse).
    Scroll(ScrollEvent),
}

/// Resultado de procesar un evento de interacción.
#[derive(Debug, Clone, Copy, Default)]
pub struct InteractionResult {
    /// `true` si el viewport cambió y requiere redibujado.
    pub viewport_changed: bool,
    /// `true` si el crosshair cambió de posición.
    pub crosshair_changed: bool,
    /// Nueva posición del crosshair en coordenadas del área de plot.
    pub crosshair_pos: Option<(f64, f64)>,
}
