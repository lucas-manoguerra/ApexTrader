//! Cálculo único y autoritativo del layout del chart.
//!
//! Proporciona un solo `ChartLayout` que tanto el sistema de input como
//! el renderer consumen, eliminando los cálculos de margen hardcodeados
//! distribuidos por separado (AT-002).
//!
//! El módulo no depende de `egui` ni de `wgpu`: trabaja con valores `f64`
//! planos. Las capas que lo consumen convierten a `egui::Rect` según sea
//! necesario.

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

/// Layout completo del chart: una sola fuente de verdad para dimensiones
/// y márgenes de ejes y paneles.
#[derive(Debug, Clone)]
pub struct ChartLayout {
    /// Rectángulo completo disponible (incluye todos los márgenes).
    pub full: Rect,
    /// Área útil de dibujo principal (dentro de los márgenes de ejes).
    pub plot: Rect,
    /// Eje de precios izquierdo.
    pub price_axis_left: Rect,
    /// Eje de precios derecho.
    pub price_axis_right: Rect,
    /// Eje de tiempo inferior.
    pub time_axis: Rect,
    /// Panel de volumen (debajo del plot principal).
    pub volume_panel: Rect,
    /// Paneles adicionales (indicadores, etc.).
    pub additional_panes: Vec<PaneRect>,
}

impl ChartLayout {
    /// Calcula el layout a partir del ancho y alto disponibles.
    ///
    /// Los márgenes son proporcionales al tamaño disponible para que el
    /// layout responda a cambios de tamaño y escalado de pantalla.
    /// Dimensiones inválidas producen un layout vacío (`is_valid() == false`).
    pub fn compute(full_width: f64, full_height: f64) -> Self {
        if !full_width.is_finite()
            || !full_height.is_finite()
            || full_width <= 0.0
            || full_height <= 0.0
        {
            return Self::empty();
        }

        let full = Rect::new(0.0, 0.0, full_width, full_height);

        // Márgenes proporcionales al tamaño disponible.
        let price_axis_width = full_width * 0.05;
        let right_margin = full_width * 0.02;
        let title_height = full_height * 0.04;
        let time_axis_height = full_height * 0.04;
        let volume_panel_height = full_height * 0.05;

        let plot_left = price_axis_width;
        let plot_right = full_width - right_margin;
        let plot_top = title_height;
        let plot_bottom = full_height - time_axis_height - volume_panel_height;

        let plot = Rect::new(plot_left, plot_top, plot_right, plot_bottom);

        let price_axis_left = Rect::new(0.0, plot_top, price_axis_width, plot_bottom);
        let price_axis_right = Rect::new(plot_right, plot_top, full_width, plot_bottom);
        let time_axis = Rect::new(plot_left, plot_bottom, plot_right, full_height);
        let volume_panel = Rect::new(
            plot_left,
            plot_bottom,
            plot_right,
            plot_bottom + volume_panel_height,
        );

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
        // El plot está contenido dentro del rect completo.
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
    fn layout_respects_proportions() {
        let layout = ChartLayout::compute(1920.0, 1080.0);
        // El eje de precios izquierdo ocupa ~5% del ancho.
        let expected_price_axis_width = 1920.0 * 0.05;
        assert!(
            (layout.price_axis_left.width() - expected_price_axis_width).abs() < 1.0,
            "price_axis_left width = {}, expected ~{}",
            layout.price_axis_left.width(),
            expected_price_axis_width
        );
        // El panel de volumen ocupa ~5% del alto.
        let expected_volume_height = 1080.0 * 0.05;
        assert!(
            (layout.volume_panel.height() - expected_volume_height).abs() < 1.0,
            "volume_panel height = {}, expected ~{}",
            layout.volume_panel.height(),
            expected_volume_height
        );
    }

    #[test]
    fn layout_empty_default() {
        let layout = ChartLayout::default();
        assert!(!layout.is_valid());
        assert_eq!(layout.plot.width(), 0.0);
    }
}
