





#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {

    Primary,

    Secondary,

    Middle,

    Other(u8),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAxis {

    Time,

    Price,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}


#[derive(Debug, Clone, Copy)]
pub struct PointerMoveEvent {

    pub x: f64,
    pub y: f64,
}


#[derive(Debug, Clone, Copy)]
pub struct PointerDownEvent {
    pub button: PointerButton,
    pub x: f64,
    pub y: f64,
    pub modifiers: Modifiers,
}


#[derive(Debug, Clone, Copy)]
pub struct AxisPointerDownEvent {
    pub axis: ScaleAxis,

    pub x: f64,

    pub y: f64,
}


#[derive(Debug, Clone, Copy)]
pub struct AxisDoubleClickEvent {
    pub axis: ScaleAxis,
}


#[derive(Debug, Clone, Copy)]
pub struct PointerUpEvent {
    pub button: PointerButton,
}


#[derive(Debug, Clone, Copy)]
pub struct PointerDragEvent {

    pub x: f64,

    pub y: f64,
}


#[derive(Debug, Clone, Copy)]
pub struct ScrollEvent {

    pub delta_x: f64,

    pub delta_y: f64,

    pub x: f64,
    pub y: f64,
    pub modifiers: Modifiers,
}


#[derive(Debug, Clone, Copy)]
pub enum InteractionEvent {

    PointerMove(PointerMoveEvent),

    PointerDown(PointerDownEvent),

    AxisPointerDown(AxisPointerDownEvent),

    AxisDoubleClick(AxisDoubleClickEvent),

    PointerUp(PointerUpEvent),

    PointerDrag(PointerDragEvent),

    Scroll(ScrollEvent),
}


#[derive(Debug, Clone, Copy, Default)]
pub struct InteractionResult {

    pub viewport_changed: bool,

    pub crosshair_changed: bool,

    pub crosshair_pos: Option<(f64, f64)>,
}
