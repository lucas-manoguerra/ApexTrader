//! Transformaciones de coordenadas del chart: única fuente de verdad.
//!
//! Este módulo concentra todas las fórmulas que convierten entre
//! **DataSpace** (índice de vela, precio) y **ScreenSpace** (píxeles de
//! pantalla), y viceversa.
//!
//! Tanto `Viewport` como los renderizadores (`GlowChartRenderer`,
//! `WgpuPipeline`) y los benchmarks deben usar estas funciones para evitar
//! fórmulas duplicadas con valores inconsistentes (la auditoría encontró
//! tres convenciones distintas: `20.0 * zoom`, `12.0 * zoom` y
//! `20.0 / zoom`).
//!
//! Relaciones definidas explícitamente:
//! - `candle_spacing`: distancia entre centros de velas consecutivas.
//! - `candle_body_width`: ancho del cuerpo de la vela, definido como una
//!   fracción del espaciamiento (no un número mágico independiente).

/// Espaciado base entre centros de velas en píxeles a zoom 1.0.
pub const CANDLE_SPACING_BASE: f64 = 20.0;

/// Fracción del espaciamiento que ocupa el ancho del cuerpo de la vela.
pub const CANDLE_BODY_WIDTH_RATIO: f64 = 0.6;

/// Margen porcentual usado en auto_scale y clamping (5% cada lado).
pub const MARGIN_RATIO: f64 = 0.05;

/// Devuelve el espaciamiento entre centros de velas en píxeles de pantalla.
///
/// A mayor zoom, mayor espaciamiento (las velas están más separadas).
pub fn candle_spacing(zoom_level: f64) -> f64 {
    CANDLE_SPACING_BASE * zoom_level
}

/// Devuelve el ancho del cuerpo de la vela en píxeles de pantalla.
///
/// Definido como una fracción del espaciamiento, para que cuerpo y
/// separación mantengan una relación explícita y no sean números mágicos
/// independientes.
pub fn candle_body_width(zoom_level: f64) -> f64 {
    candle_spacing(zoom_level) * CANDLE_BODY_WIDTH_RATIO
}

/// Convierte un índice de vela (coordenada de datos X) a píxel de pantalla X.
pub fn data_x_to_screen(index: usize, screen_width: f64, zoom_level: f64, offset_x: f64) -> f64 {
    let spacing = candle_spacing(zoom_level);
    let center_x = screen_width / 2.0;
    center_x + (index as f64 * spacing) + offset_x * spacing
}

/// Convierte un píxel de pantalla X al índice de vela más cercano.
///
/// `candle_count` limita el resultado a `[0, candle_count - 1]`.
pub fn screen_x_to_data(
    screen_x: f64,
    screen_width: f64,
    zoom_level: f64,
    offset_x: f64,
    candle_count: usize,
) -> usize {
    if candle_count == 0
        || !screen_x.is_finite()
        || !screen_width.is_finite()
        || screen_width <= 0.0
    {
        return 0;
    }

    let spacing = candle_spacing(zoom_level);
    if spacing <= 0.0 || !spacing.is_finite() {
        return 0;
    }

    let center_x = screen_width / 2.0;
    let index = (screen_x - center_x - offset_x * spacing) / spacing;
    let index = index.round();

    if index < 0.0 {
        return 0;
    }
    (index as usize).min(candle_count.saturating_sub(1))
}

/// Convierte un precio (coordenada de datos Y) a píxel de pantalla Y.
///
/// El eje Y está invertido (0 está arriba). `center_price` es el precio
/// alrededor del cual se hace zoom; `price_range` es el rango visible
/// (máximo - mínimo); `offset_y` es el desplazamiento vertical en unidades
/// de precio.
pub fn price_to_screen(
    price: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    let center_y = screen_height / 2.0;
    let pixels_per_unit = screen_height / price_range * zoom_level;
    let relative_price = price - center_price - offset_y * price_range;
    center_y - relative_price * pixels_per_unit
}

/// Convierte un píxel de pantalla Y a un precio (coordenada de datos Y).
pub fn screen_y_to_price(
    screen_y: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    let center_y = screen_height / 2.0;
    let pixels_per_unit = screen_height / price_range * zoom_level;
    let relative_price = (center_y - screen_y) / pixels_per_unit;
    center_price + offset_y * price_range + relative_price
}

/// Valida que un rango de precios sea apto para calcular una escala visual.
///
/// Un rango con `min == max` es válido como dato, pero no apto para calcular
/// una escala (división por cero). Devuelve el rango de precios como `f64`
/// o `None` si no es finito o es degenerado.
pub fn validate_price_range(min: f64, max: f64) -> Option<f64> {
    if !min.is_finite() || !max.is_finite() {
        return None;
    }
    let range = max - min;
    if !range.is_finite() || range == 0.0 {
        return None;
    }
    Some(range)
}

// ---------------------------------------------------------------------------
// Espacios de coordenadas y conversiones (S4)
// ---------------------------------------------------------------------------

/// Punto en coordenadas de pantalla: píxeles desde la esquina superior
/// izquierda del rect de dibujo recibido por el renderer (incluye márgenes
/// de ejes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

/// Punto en coordenadas del área útil de chart: píxeles desde el inicio del
/// área de dibujo útil (sin los márgenes de ejes de precios y tiempo).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
}

/// Punto en coordenadas de datos: índice de vela (como f64) y precio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub index: f64,
    pub price: f64,
}

/// Convierte un `ScreenPoint` (rect completo) a `ChartPoint` (área útil),
/// restando los márgenes de ejes `left` y `top`.
pub fn screen_to_chart_point(screen: ScreenPoint, left: f64, top: f64) -> ChartPoint {
    ChartPoint {
        x: screen.x - left,
        y: screen.y - top,
    }
}

/// Convierte un `ChartPoint` a un `DataPoint` (índice de vela y precio)
/// usando el estado del viewport y las dimensiones del área de dibujo.
#[allow(clippy::too_many_arguments)]
pub fn chart_point_to_data_point(
    chart: ChartPoint,
    screen_width: f64,
    screen_height: f64,
    zoom_level: f64,
    offset_x: f64,
    offset_y: f64,
    price_range: f64,
    center_price: f64,
    candle_count: usize,
) -> DataPoint {
    let index = screen_x_to_data(chart.x, screen_width, zoom_level, offset_x, candle_count) as f64;
    let price = screen_y_to_price(
        chart.y,
        screen_height,
        price_range,
        zoom_level,
        offset_y,
        center_price,
    );
    DataPoint { index, price }
}

/// Rango de velas visibles en el área de dibujo.
///
/// El renderer debe iterar solo `first..=last`, de modo que el trabajo por
/// frame sea O(N_visible) en lugar de O(N_total).
///
/// Un rango es vacío cuando `first > last`. Un rango `{first: 0, last: 0}`
/// **no** es vacío (representaría la vela cero); use [`VisibleRange::empty`]
/// para representar "sin velas visibles".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    pub first: usize,
    pub last: usize,
}

impl VisibleRange {
    /// Crea un rango vacío (no hay velas visibles).
    ///
    /// Devuelve `{first: 1, last: 0}`, de modo que `is_empty()` sea `true`
    /// y nunca se confunda con un rango que contiene la vela cero.
    pub fn empty() -> Self {
        Self { first: 1, last: 0 }
    }

    /// Crea un rango con las velas dadas.
    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }

    /// Devuelve `true` si no hay velas visibles.
    pub fn is_empty(&self) -> bool {
        self.first > self.last
    }

    /// Devuelve la cantidad de velas visibles.
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.last - self.first + 1
        }
    }
}

impl Default for VisibleRange {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_empty_is_empty() {
        let r = VisibleRange::empty();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!((r.first, r.last), (1, 0));
    }

    #[test]
    fn visible_range_default_is_empty() {
        let r = VisibleRange::default();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn visible_range_new_reports_length() {
        let r = VisibleRange::new(10, 20);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 11);
    }

    #[test]
    fn visible_range_single_candle() {
        let r = VisibleRange::new(5, 5);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn visible_range_zero_zero_is_not_empty() {
        // {first: 0, last: 0} representa la vela cero, NO un rango vacío.
        let r = VisibleRange::new(0, 0);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }
}
