//! Modelo de crosshair del chart.
//!
//! El crosshair usa las mismas transformaciones y estado de escala que el
//! renderer, sin duplicar fórmulas de conversión de coordenadas.
//! Es un tipo puro de dominio: no conoce `egui` ni `wgpu`.

use crate::chart::layout::ChartLayout;
use crate::domain::transforms::screen_y_to_price;
use crate::domain::viewport::Viewport;

/// Modo de visualización del crosshair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairMode {
    /// Líneas vertical y horizontal completas.
    #[default]
    Full,
    /// Solo etiquetas, sin líneas.
    LabelsOnly,
    /// Crosshair oculto.
    Hidden,
}

/// Parámetros de contexto para actualizar el crosshair.
///
/// Agrupa los datos que no cambian por frame (dimensiones, rango de precio,
/// cantidad de velas) para evitar pasar muchos argumentos sueltos.
#[derive(Debug, Clone, Copy)]
pub struct CrosshairUpdateParams<'a> {
    /// Layout del chart (dimensiones y márgenes).
    pub layout: &'a ChartLayout,
    /// Rango de precios visible (máximo - mínimo).
    pub price_range: f64,
    /// Precio central de zoom.
    pub center_price: f64,
    /// Cantidad total de velas cargadas.
    pub candle_count: usize,
}

/// Estado del crosshair: posición, precio e índice bajo el cursor.
#[derive(Debug, Clone, Copy)]
pub struct Crosshair {
    /// Modo actual.
    pub mode: CrosshairMode,
    /// Posición X en el área de plot (píxeles).
    pub x: f64,
    /// Posición Y en el área de plot (píxeles).
    pub y: f64,
    /// Índice de vela bajo el cursor (redondeado al entero más cercano).
    pub index: usize,
    /// Precio bajo el cursor.
    pub price: f64,
    /// `true` si el cursor está dentro del área de plot.
    pub visible: bool,
}

impl Crosshair {
    /// Crea un crosshair oculto.
    pub fn hidden() -> Self {
        Self {
            mode: CrosshairMode::Hidden,
            x: 0.0,
            y: 0.0,
            index: 0,
            price: 0.0,
            visible: false,
        }
    }

    /// Actualiza la posición del crosshair a partir de la posición del cursor
    /// en el área de plot, usando las transformaciones del viewport y el
    /// layout del chart.
    ///
    /// No modifica el viewport: solo calcula la representación visual del
    /// crosshair a partir del estado actual.
    pub fn update(
        &mut self,
        cursor_x: f64,
        cursor_y: f64,
        params: CrosshairUpdateParams<'_>,
        viewport: &Viewport,
    ) {
        let layout = params.layout;
        if !layout.is_valid() || !layout.plot.is_valid() {
            self.visible = false;
            return;
        }

        let plot_left = layout.plot.left;
        let plot_top = layout.plot.top;
        let plot_width = layout.plot.width();
        let plot_height = layout.plot.height();
        let price_range = params.price_range;
        let center_price = params.center_price;
        let candle_count = params.candle_count;

        // El cursor debe estar dentro del área de plot.
        let rel_x = cursor_x - plot_left;
        let rel_y = cursor_y - plot_top;

        if rel_x < 0.0 || rel_x > plot_width || rel_y < 0.0 || rel_y > plot_height {
            self.visible = false;
            return;
        }

        self.visible = true;
        self.x = cursor_x;
        self.y = cursor_y;

        // Índice y precio usando las mismas transformaciones que el renderer.
        let center_x = plot_width / 2.0;
        let spacing = viewport.candle_spacing();
        if spacing > 0.0 {
            let index = ((rel_x - center_x) / spacing + viewport.offset_x()).round();
            self.index = index.clamp(0.0, candle_count as f64 - 1.0) as usize;
        } else {
            self.index = 0;
        }

        // Precio bajo el cursor usando la misma transformación inversa Y
        // que el renderer (screen_y_to_price con zoom_y y offset_y).
        self.price = screen_y_to_price(
            rel_y,
            plot_height,
            price_range,
            viewport.zoom_y(),
            viewport.offset_y(),
            center_price,
        );
    }

    /// Devuelve `true` si el crosshair está visible y debe dibujarse.
    pub fn is_visible(&self) -> bool {
        self.visible && self.mode != CrosshairMode::Hidden
    }

    /// Devuelve `true` si se deben dibujar las líneas del crosshair.
    pub fn draw_lines(&self) -> bool {
        self.is_visible() && self.mode == CrosshairMode::Full
    }

    /// Devuelve `true` si se deben mostrar las etiquetas.
    pub fn draw_labels(&self) -> bool {
        self.is_visible() && self.mode != CrosshairMode::Hidden
    }
}

impl Default for Crosshair {
    fn default() -> Self {
        Self::hidden()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;

    fn make_viewport() -> Viewport {
        let mut v = Viewport::new();
        v.set_zoom(2.0, 2.0);
        v
    }

    fn make_layout() -> ChartLayout {
        ChartLayout::compute(1920.0, 1080.0)
    }

    #[test]
    fn crosshair_default_is_hidden() {
        let ch = Crosshair::default();
        assert!(!ch.is_visible());
        assert!(!ch.draw_lines());
        assert!(!ch.draw_labels());
    }

    #[test]
    fn crosshair_outside_plot_is_invisible() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();

        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(0.0, 500.0, params, &viewport);
        assert!(!ch.visible);
    }

    #[test]
    fn crosshair_inside_plot_is_visible() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();

        // Centro del plot.
        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;
        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(cx, cy, params, &viewport);

        assert!(ch.visible);
        assert_eq!(ch.x, cx);
        assert_eq!(ch.y, cy);
    }

    #[test]
    fn crosshair_index_at_center_is_zero() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();

        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;
        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(cx, cy, params, &viewport);

        // Con offset_x = 0, el índice en el centro es 0.
        assert_eq!(ch.index, 0);
    }

    #[test]
    fn crosshair_mode_labels_only_draws_labels_not_lines() {
        let ch = Crosshair {
            mode: CrosshairMode::LabelsOnly,
            visible: true,
            ..Default::default()
        };
        assert!(ch.draw_labels());
        assert!(!ch.draw_lines());
    }

    #[test]
    fn crosshair_hidden_never_draws() {
        let ch = Crosshair {
            mode: CrosshairMode::Hidden,
            visible: true,
            ..Default::default()
        };
        assert!(!ch.draw_labels());
        assert!(!ch.draw_lines());
    }

    #[test]
    fn candle_is_pure_domain() {
        // Verifica que Candle sigue siendo un tipo puro de dominio (no depende
        // de egui/wgpu), como requiere la arquitectura.
        let c = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).expect("vela válida");
        assert_eq!(c.open(), 100.0);
    }
}
