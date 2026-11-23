//! Opciones configurables para los gestos del chart.
//!
//! Los grupos separan navegación (scroll) de modificación de escala (scale),
//! siguiendo el modelo de configuración de Lightweight Charts. Sus nombres
//! describen los gestos que soporta actualmente ApexTrader en escritorio.

/// Configuración de todas las interacciones del chart.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InteractionOptions {
    /// Gestos que desplazan el historial sin cambiar el zoom.
    pub scroll: ScrollOptions,
    /// Gestos que modifican el nivel de zoom.
    pub scale: ScaleOptions,
}

/// Opciones de navegación temporal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollOptions {
    /// Habilita el desplazamiento del historial con la rueda horizontal.
    pub horizontal_mouse_wheel: bool,
    /// Habilita el desplazamiento del historial arrastrando con el botón izquierdo.
    pub primary_mouse_drag: bool,
}

impl Default for ScrollOptions {
    fn default() -> Self {
        Self {
            horizontal_mouse_wheel: true,
            primary_mouse_drag: true,
        }
    }
}

/// Opciones de zoom temporal y de precios.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleOptions {
    /// Habilita el zoom de ambos ejes arrastrando con el botón derecho.
    pub secondary_mouse_drag: bool,
    /// Habilita el zoom temporal con la rueda vertical, anclado al cursor.
    pub vertical_mouse_wheel: bool,
    /// Gestos directos de arrastre sobre los ejes.
    pub axis_pressed_mouse_move: AxisPressedMouseMoveOptions,
    /// Restablecimiento independiente de escalas mediante doble clic.
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

/// Configura el arrastre directo de cada eje.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisPressedMouseMoveOptions {
    /// Habilita el zoom temporal arrastrando verticalmente el eje inferior.
    pub time: bool,
    /// Habilita el zoom de precios arrastrando horizontalmente el eje lateral.
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

/// Configura el restablecimiento de escala mediante doble clic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisDoubleClickResetOptions {
    /// Ajusta el historial completo al área visible.
    pub time: bool,
    /// Restablece la escala vertical al nivel base y centra el rango visible.
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
