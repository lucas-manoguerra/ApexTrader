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
pub fn data_x_to_screen(
    index: usize,
    screen_width: f64,
    zoom_level: f64,
    offset_x: f64,
) -> f64 {
    let spacing = candle_spacing(zoom_level);
    let center_x = screen_width / 2.0;
    center_x + (index as f64 * spacing) + offset_x * spacing
}

/// Convierte un píxel de pantalla X al índice de vela más cercano.
pub fn screen_x_to_data(
    screen_x: f64,
    screen_width: f64,
    zoom_level: f64,
    offset_x: f64,
) -> usize {
    let spacing = candle_spacing(zoom_level);
    let center_x = screen_width / 2.0;
    let index = (screen_x - center_x - offset_x * spacing) / spacing;
    index.round() as usize
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
