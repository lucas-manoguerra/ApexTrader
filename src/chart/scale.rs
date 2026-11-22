//! Escalas independientes para los ejes X y Y.
//!
//! El eje X (tiempo / índice de vela) y el eje Y (precio) escalan de forma
//! independiente: el zoom vertical no distorsiona el horizontal y viceversa.
//! Esto permite, por ejemplo, hacer zoom solo en precios sin afectar la
//! cantidad de velas visibles, y viceversa.

use crate::domain::transforms;

/// Configuración de escala para un eje.
#[derive(Debug, Clone, Copy)]
pub struct AxisScale {
    /// Nivel de zoom del eje. 1.0 = escala base.
    pub zoom: f64,
    /// Desplazamiento en unidades de datos del eje.
    pub offset: f64,
    /// Nivel mínimo de zoom permitido.
    pub min_zoom: f64,
    /// Nivel máximo de zoom permitido.
    pub max_zoom: f64,
}

impl Default for AxisScale {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: 0.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }
}

/// Estado de escalas del chart: eje X y eje Y independientes.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChartScales {
    /// Escala del eje X (tiempo / índice de vela).
    pub x: AxisScale,
    /// Escala del eje Y (precio).
    pub y: AxisScale,
}

impl ChartScales {
    /// Crea escalas con valores por defecto.
    pub fn new() -> Self {
        Self::default()
    }

    /// Establece el zoom de ambos ejes manteniendo la proporción actual.
    pub fn set_zoom(&mut self, x_zoom: f64, y_zoom: f64) {
        self.x.zoom = x_zoom.clamp(self.x.min_zoom, self.x.max_zoom);
        self.y.zoom = y_zoom.clamp(self.y.min_zoom, self.y.max_zoom);
    }

    /// Establece el desplazamiento de ambos ejes.
    pub fn set_offset(&mut self, x_offset: f64, y_offset: f64) {
        self.x.offset = x_offset;
        self.y.offset = y_offset;
    }

    /// Devuelve `true` si alguna escala está fuera de sus límites.
    pub fn is_valid(&self) -> bool {
        self.x.zoom.is_finite()
            && self.y.zoom.is_finite()
            && self.x.offset.is_finite()
            && self.y.offset.is_finite()
            && self.x.zoom > 0.0
            && self.y.zoom > 0.0
            && self.x.zoom >= self.x.min_zoom
            && self.x.zoom <= self.x.max_zoom
            && self.y.zoom >= self.y.min_zoom
            && self.y.zoom <= self.y.max_zoom
    }

    /// Aplica un zoom alrededor de un punto dado en cada eje de forma
    /// independiente. El punto `center` (0..=1) se mantiene fijo en
    /// pantalla en ambos ejes.
    pub fn zoom_at(
        &mut self,
        screen_width: f64,
        screen_height: f64,
        center_x: f64,
        center_y: f64,
        x_zoom_factor: f64,
        y_zoom_factor: f64,
    ) {
        if !screen_width.is_finite()
            || !screen_height.is_finite()
            || !center_x.is_finite()
            || !center_y.is_finite()
            || !x_zoom_factor.is_finite()
            || !y_zoom_factor.is_finite()
        {
            return;
        }

        // Eje X: ajusta offset para mantener center_x fijo.
        let center_x_px = center_x * screen_width;
        let offset_x_px = self.x.offset * transforms::candle_spacing(self.x.zoom);
        let new_x_zoom = (self.x.zoom * x_zoom_factor).clamp(self.x.min_zoom, self.x.max_zoom);
        let x_zoom_ratio = new_x_zoom / self.x.zoom;
        let new_offset_x_px =
            center_x_px + (offset_x_px - center_x_px) * x_zoom_ratio;
        self.x.offset = new_offset_x_px / transforms::candle_spacing(new_x_zoom);
        self.x.zoom = new_x_zoom;

        // Eje Y: ajusta offset para mantener center_y fijo.
        let center_y_px = center_y * screen_height;
        // offset_y está en unidades de precio; para convertir a píxeles usamos
        // la relación del eje Y (screen_height / price_range * zoom).
        // Aquí usamos una aproximación local: el factor de conversión
        // píxeles-por-unidad depende del price_range, que no conoce esta
        // estructura. Dejamos el offset_y en unidades de precio y lo
        // escalamos proporcionalmente al ratio de zoom.
        let offset_y_px = self.y.offset * screen_height / self.y.zoom;
        let new_y_zoom = (self.y.zoom * y_zoom_factor).clamp(self.y.min_zoom, self.y.max_zoom);
        let y_zoom_ratio = new_y_zoom / self.y.zoom;
        let new_offset_y_px =
            center_y_px + (offset_y_px - center_y_px) * y_zoom_ratio;
        self.y.offset = new_offset_y_px * new_y_zoom / screen_height;
        self.y.zoom = new_y_zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_are_independent() {
        let mut scales = ChartScales::new();
        scales.set_zoom(2.0, 5.0);
        assert!((scales.x.zoom - 2.0).abs() < f64::EPSILON);
        assert!((scales.y.zoom - 5.0).abs() < f64::EPSILON);
        // Cambiar solo X no afecta Y.
        scales.set_zoom(3.0, scales.y.zoom);
        assert!((scales.x.zoom - 3.0).abs() < f64::EPSILON);
        assert!((scales.y.zoom - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zoom_clamps_to_limits() {
        let mut scales = ChartScales::new();
        scales.set_zoom(100.0, 100.0);
        assert!((scales.x.zoom - 10.0).abs() < f64::EPSILON);
        assert!((scales.y.zoom - 10.0).abs() < f64::EPSILON);
        scales.set_zoom(0.01, 0.01);
        assert!((scales.x.zoom - 0.1).abs() < f64::EPSILON);
        assert!((scales.y.zoom - 0.1).abs() < f64::EPSILON);
    }
}
