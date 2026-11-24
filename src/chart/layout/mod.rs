//! Cálculo único y autoritativo del layout del chart.
//!
//! El plot reserva un gutter mínimo a la izquierda y una escala de precios
//! visible a la derecha, siguiendo la distribución habitual de plataformas
//! de gráficos. El panel de volumen y el eje temporal tienen áreas separadas.
//! El módulo no depende de egui ni de wgpu.

/// Rectángulo con bordes en píxeles de pantalla.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Rect {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(self) -> f64 {
        self.right - self.left
    }

    pub fn height(self) -> f64 {
        self.bottom - self.top
    }

    pub fn is_valid(self) -> bool {
        self.left.is_finite()
            && self.top.is_finite()
            && self.right.is_finite()
            && self.bottom.is_finite()
            && self.right > self.left
            && self.bottom > self.top
    }
}

/// Un panel con su rectángulo y un índice ordenado.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneRect {
    pub index: usize,
    pub rect: Rect,
}

/// Layout completo del chart: una sola fuente de verdad para dimensiones,
/// márgenes, escalas y paneles.
#[derive(Debug, Clone)]
pub struct ChartLayout {
    /// Rectángulo completo disponible (incluye todos los márgenes).
    pub full: Rect,
    /// Área útil de dibujo principal (dentro de los márgenes de ejes).
    pub plot: Rect,
    /// Gutter izquierdo sin escala visible.
    pub price_axis_left: Rect,
    /// Escala de precios visible a la derecha.
    pub price_axis_right: Rect,
    /// Eje temporal inferior, separado del panel de volumen.
    pub time_axis: Rect,
    /// Panel de volumen (debajo del plot principal).
    pub volume_panel: Rect,
    /// Paneles adicionales (indicadores, etc.).
    pub additional_panes: Vec<PaneRect>,
}

impl ChartLayout {
    /// Calcula el layout a partir del ancho y alto disponibles.
    ///
    /// La escala de precios conserva un ancho legible, el eje temporal dispone
    /// de una franja propia y el volumen utiliza una porción visible del chart.
    /// Dimensiones demasiado pequeñas o inválidas producen un layout vacío.
    pub fn compute(full_width: f64, full_height: f64) -> Self {
        let default_price_axis_width = if full_width.is_finite() && full_width > 0.0 {
            (full_width * 0.09).clamp(64.0, 96.0).min(full_width * 0.35)
        } else {
            0.0
        };
        Self::compute_with_price_axis_width(full_width, full_height, default_price_axis_width)
    }

    /// Calcula el layout reservando el ancho solicitado para la escala de precios.
    ///
    /// El ancho se limita para conservar un área útil de plot incluso en ventanas
    /// pequeñas. Un valor inválido utiliza el ancho predeterminado.
    pub fn compute_with_price_axis_width(
        full_width: f64,
        full_height: f64,
        requested_price_axis_width: f64,
    ) -> Self {
        if !full_width.is_finite()
            || !full_height.is_finite()
            || full_width <= 0.0
            || full_height <= 0.0
        {
            return Self::empty();
        }

        let full = Rect::new(0.0, 0.0, full_width, full_height);

        let left_gutter = (full_width * 0.012).clamp(8.0, 16.0).min(full_width * 0.12);
        let default_price_axis_width = (full_width * 0.09).clamp(64.0, 96.0);
        let requested_price_axis_width = if requested_price_axis_width.is_finite()
            && requested_price_axis_width > 0.0
        {
            requested_price_axis_width
        } else {
            default_price_axis_width
        };
        let price_axis_width = requested_price_axis_width.clamp(64.0, 144.0).min(full_width * 0.35);
        let title_height = (full_height * 0.02).clamp(4.0, 12.0);
        let mut time_axis_height = (full_height * 0.075).clamp(12.0, 40.0);
        let mut volume_panel_height = (full_height * 0.14).clamp(18.0, 72.0);

        // Mantener al menos 24 px para el plot incluso en ventanas pequeñas.
        let available_for_bottom = (full_height - title_height - 24.0).max(0.0);
        let reserved_bottom = time_axis_height + volume_panel_height;
        if reserved_bottom > available_for_bottom && reserved_bottom > 0.0 {
            let factor = available_for_bottom / reserved_bottom;
            time_axis_height *= factor;
            volume_panel_height *= factor;
        }

        let plot_left = left_gutter;
        let plot_right = full_width - price_axis_width;
        let plot_top = title_height;
        let plot_bottom = full_height - time_axis_height - volume_panel_height;

        let plot = Rect::new(plot_left, plot_top, plot_right, plot_bottom);
        if plot.width() < 24.0 || plot.height() < 16.0 {
            return Self::empty();
        }

        let price_axis_left = Rect::new(0.0, plot_top, plot_left, plot_bottom);
        let price_axis_right = Rect::new(plot_right, plot_top, full_width, plot_bottom);
        let volume_panel = Rect::new(
            plot_left,
            plot_bottom,
            plot_right,
            plot_bottom + volume_panel_height,
        );
        let time_axis = Rect::new(plot_left, volume_panel.bottom, plot_right, full_height);

        Self {
            full,
            plot,
            price_axis_left,
            price_axis_right,
            time_axis,
            volume_panel,
            additional_panes: Vec::new(),
        }
    }

    /// Devuelve un layout vacío (dimensiones cero).
    pub fn empty() -> Self {
        let zero = Rect::new(0.0, 0.0, 0.0, 0.0);
        Self {
            full: zero,
            plot: zero,
            price_axis_left: zero,
            price_axis_right: zero,
            time_axis: zero,
            volume_panel: zero,
            additional_panes: Vec::new(),
        }
    }

    /// Devuelve `true` si el layout tiene dimensiones válidas y no vacías.
    pub fn is_valid(&self) -> bool {
        self.full.is_valid() && self.plot.is_valid()
    }

    /// Devuelve el ancho del área de plot.
    pub fn plot_width(&self) -> f64 {
        self.plot.width()
    }

    /// Devuelve el alto del área de plot.
    pub fn plot_height(&self) -> f64 {
        self.plot.height()
    }
}

impl Default for ChartLayout {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_computes_valid_rects() {
        let layout = ChartLayout::compute(1920.0, 1080.0);
        assert!(layout.is_valid());
        assert_eq!(layout.full.width(), 1920.0);
        assert_eq!(layout.full.height(), 1080.0);
        assert!(layout.plot.width() > 0.0);
        assert!(layout.plot.height() > 0.0);
        assert!(layout.plot.left >= layout.full.left);
        assert!(layout.plot.right <= layout.full.right);
        assert!(layout.plot.top >= layout.full.top);
        assert!(layout.plot.bottom <= layout.full.bottom);
    }

    #[test]
    fn layout_empty_for_invalid_dimensions() {
        let layout = ChartLayout::compute(0.0, 1080.0);
        assert!(!layout.is_valid());
        assert_eq!(layout.plot.width(), 0.0);
        assert_eq!(layout.plot.height(), 0.0);
    }

    #[test]
    fn layout_negative_dimensions_are_empty() {
        let layout = ChartLayout::compute(-100.0, 1080.0);
        assert!(!layout.is_valid());
    }

    #[test]
    fn layout_non_finite_dimensions_are_empty() {
        let layout = ChartLayout::compute(f64::NAN, 1080.0);
        assert!(!layout.is_valid());
        let layout = ChartLayout::compute(1920.0, f64::INFINITY);
        assert!(!layout.is_valid());
    }

    #[test]
    fn price_scale_is_on_the_right_and_has_readable_width() {
        let layout = ChartLayout::compute(1024.0, 480.0);
        assert!(layout.is_valid());
        assert!(
            layout.plot.left < 20.0,
            "el margen izquierdo debe ser pequeño"
        );
        assert!(
            layout.price_axis_right.width() >= 64.0,
            "la escala derecha debe tener espacio para etiquetas"
        );
        assert!((layout.price_axis_right.right - layout.full.right).abs() < 1.0);
        assert!((layout.price_axis_right.left - layout.plot.right).abs() < 1.0);
    }

    #[test]
    fn price_axis_width_adapts_to_the_requested_label_width() {
        let compact = ChartLayout::compute_with_price_axis_width(1_000.0, 600.0, 72.0);
        let wide = ChartLayout::compute_with_price_axis_width(1_000.0, 600.0, 124.0);

        assert!((compact.price_axis_right.width() - 72.0).abs() < 1e-9);
        assert!((wide.price_axis_right.width() - 124.0).abs() < 1e-9);
        assert!(wide.plot.width() < compact.plot.width());
        assert_eq!(wide.plot.right, wide.price_axis_right.left);
    }

    #[test]
    fn price_axis_width_is_capped_to_preserve_a_usable_plot() {
        let layout = ChartLayout::compute_with_price_axis_width(240.0, 180.0, 500.0);

        assert!((layout.price_axis_right.width() - 84.0).abs() < 1e-9);
        assert!(layout.plot.width() >= 24.0);
    }

    #[test]
    fn volume_and_time_axis_have_separate_non_overlapping_regions() {
        let layout = ChartLayout::compute(1024.0, 480.0);
        assert!(layout.volume_panel.height() > 40.0);
        assert!(layout.time_axis.height() >= 24.0);
        assert!((layout.volume_panel.bottom - layout.time_axis.top).abs() < 1.0);
        assert!((layout.time_axis.bottom - layout.full.bottom).abs() < 1.0);
    }

    #[test]
    fn layout_remains_valid_in_a_small_window() {
        let layout = ChartLayout::compute(360.0, 180.0);
        assert!(layout.is_valid());
        assert!(layout.plot.width() >= 24.0);
        assert!(layout.plot.height() >= 16.0);
    }

    #[test]
    fn layout_empty_default() {
        let layout = ChartLayout::default();
        assert!(!layout.is_valid());
        assert_eq!(layout.plot.width(), 0.0);
    }
}
