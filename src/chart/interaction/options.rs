//! Opciones configurables para los gestos del chart.
//!
//! Los grupos separan navegación (scroll) de modificación de escala (scale),
//! siguiendo el modelo de configuración de Lightweight Charts. Sus nombres
//! describen los gestos que soporta actualmente ApexTrader en escritorio.

/// Configuración de todas las interacciones del chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractionOptions {
    /// Gestos que desplazan el historial sin cambiar el zoom.
    pub scroll: ScrollOptions,
    /// Gestos que modifican el nivel de zoom.
    pub scale: ScaleOptions,
}

impl Default for InteractionOptions {
    fn default() -> Self {
        Self {
            scroll: ScrollOptions::default(),
            scale: ScaleOptions::default(),
        }
    }
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
    /// Habilita el zoom temporal con la rueda vertical, anclado al cursor.
    pub vertical_mouse_wheel: bool,
    /// Habilita el zoom de ambos ejes arrastrando con el botón derecho.
    pub secondary_mouse_drag: bool,
}

impl Default for ScaleOptions {
    fn default() -> Self {
        Self {
            vertical_mouse_wheel: true,
            secondary_mouse_drag: true,
        }
    }
}
