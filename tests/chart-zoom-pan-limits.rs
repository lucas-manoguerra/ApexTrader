use apex_trader::domain::viewport::Viewport;

/// S7.1: Pan horizontal limitado al rango de velas.
#[test]
fn pan_limited_to_candle_range_left() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(1.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    // Hacer pan a la izquierda (offset_x negativo grande).
    viewport.set_offset_x(-1000.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

    // La primera vela (index 0) debe estar visible.
    let x_first = viewport.data_x_to_screen(0, screen_width);
    let margin = screen_width * 0.05;
    assert!(
        x_first >= margin,
        "Primera vela debe estar dentro del margen izquierdo: x_first={}, margin={}",
        x_first,
        margin
    );
}

/// S7.1: Pan horizontal limitado al rango de velas (derecha).
#[test]
fn pan_limited_to_candle_range_right() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(1.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    // Hacer pan a la derecha (offset_x positivo grande).
    viewport.set_offset_x(1000.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

    // La última vela (index N-1) debe estar visible.
    let x_last = viewport.data_x_to_screen(candle_count - 1, screen_width);
    let margin = screen_width * 0.05;
    assert!(
        x_last <= screen_width - margin,
        "Última vela debe estar dentro del margen derecho: x_last={}, screen_width-margin={}",
        x_last,
        screen_width - margin
    );
}

/// S7.2: Zoom máximo limitado (una vela no puede llenar toda la pantalla).
#[test]
fn zoom_max_limited() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(100.0); // Intentar zoom extremo

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_level() <= 5.0,
        "Zoom máximo no debe exceder 5.0: zoom={}",
        viewport.zoom_level()
    );
}

/// S7.3: Zoom mínimo limitado (al menos 2 velas deben caber).
#[test]
fn zoom_min_limited() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(0.001); // Zoom extremadamente bajo

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    // Con screen_width=1920, min_zoom_by_candles = 40.0 / 1920.0 ≈ 0.0208
    // Pero también está limitado por min_zoom=0.1 del Viewport.
    assert!(
        viewport.zoom_level() >= 0.1,
        "Zoom mínimo debe ser al menos min_zoom (0.1): zoom={}",
        viewport.zoom_level()
    );
}

/// S7.3: Con pantalla pequeña, el zoom mínimo aumenta para que 2 velas quepan.
#[test]
fn zoom_min_limited_by_screen_width() {
    let mut viewport = Viewport::new();
    viewport.set_min_zoom(0.001); // Bajar min_zoom para probar el límite por pantalla

    let candle_count = 50;
    let screen_width = 50.0; // Pantalla muy pequeña

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    // 40.0 / 50.0 = 0.8, así que zoom mínimo debe ser al menos 0.8
    assert!(
        viewport.zoom_level() >= 0.8,
        "Con pantalla pequeña, zoom mínimo debe ser mayor: zoom={}",
        viewport.zoom_level()
    );
}

/// S7.2: Con pantalla ancha, el zoom máximo sigue siendo 5.0.
#[test]
fn zoom_max_limited_by_candle_spacing() {
    let mut viewport = Viewport::new();
    viewport.set_max_zoom(100.0); // Permitir zoom muy alto

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_level() <= 5.0,
        "Zoom máximo por espaciamiento de velas no debe exceder 5.0: zoom={}",
        viewport.zoom_level()
    );
}

/// S7.1: Con una sola vela, el pan debe mantenerla centrada en pantalla.
#[test]
fn pan_single_candle_keeps_visible() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(1.0);

    let candle_count = 1;
    let screen_width = 1920.0;

    // Hacer pan a la derecha (offset_x positivo grande).
    viewport.set_offset_x(500.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

    // La única vela (index 0) debe estar visible dentro del margen.
    let x_first = viewport.data_x_to_screen(0, screen_width);
    let margin = screen_width * 0.05;
    assert!(
        x_first >= margin && x_first <= screen_width - margin,
        "Única vela debe estar dentro de los márgenes: x={}, margin={}, screen_width-margin={}",
        x_first,
        margin,
        screen_width - margin
    );
}

/// S7.2 y S7.3: set_zoom_level respeta los límites de velas.
#[test]
fn set_zoom_level_respects_candle_limits() {
    let mut viewport = Viewport::new();
    viewport.set_max_zoom(100.0);
    viewport.set_min_zoom(0.001);

    let candle_count = 50;
    let screen_width = 1920.0;

    // Intentar zoom muy alto.
    viewport.set_zoom_level(50.0);
    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_level() <= 5.0,
        "set_zoom_level + clamp_zoom_to_candles no debe exceder 5.0: zoom={}",
        viewport.zoom_level()
    );
}
