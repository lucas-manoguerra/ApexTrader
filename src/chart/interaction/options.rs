#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InteractionOptions {

    pub scroll: ScrollOptions,

    pub scale: ScaleOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollOptions {

    pub horizontal_mouse_wheel: bool,

    pub primary_mouse_drag: bool,

    pub invert_vertical_pan: bool,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            horizontal_mouse_wheel: true,
            primary_mouse_drag: true,
            invert_vertical_pan: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleOptions {

    pub secondary_mouse_drag: bool,

    pub vertical_mouse_wheel: bool,

    pub axis_pressed_mouse_move: AxisPressedMouseMoveOptions,

    pub axis_double_click_reset: AxisDoubleClickResetOptions,
}

impl Default for ScaleOptions {
    fn default() -> Self {
        Self {
            secondary_mouse_drag: true,
            vertical_mouse_wheel: true,
            axis_pressed_mouse_move: AxisPressedMouseMoveOptions::default(),
            axis_double_click_reset: AxisDoubleClickResetOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisPressedMouseMoveOptions {

    pub time: bool,

    pub price: bool,
}

impl Default for AxisPressedMouseMoveOptions {
    fn default() -> Self {
        Self {
            time: true,
            price: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisDoubleClickResetOptions {

    pub time: bool,

    pub price: bool,
}

impl Default for AxisDoubleClickResetOptions {
    fn default() -> Self {
        Self {
            time: true,
            price: true,
        }
    }
}
