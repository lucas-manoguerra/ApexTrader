use apex_trader::domain::viewport::Viewport;

#[test]
fn pan_limited_to_candle_range_left() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(1.0, 1.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.set_offset_x(-1000.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

    let x_first = viewport.data_x_to_screen(0, screen_width);
    let margin = screen_width * 0.05;
    assert!(
        x_first >= margin,
        "Primera vela debe estar dentro del margen izquierdo: x_first={}, margin={}",
        x_first,
        margin
    );
}

#[test]
fn pan_limited_to_candle_range_right() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(1.0, 1.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.set_offset_x(1000.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

    let x_last = viewport.data_x_to_screen(candle_count - 1, screen_width);
    let margin = screen_width * 0.05;
    assert!(
        x_last <= screen_width - margin,
        "Última vela debe estar dentro del margen derecho: x_last={}, screen_width-margin={}",
        x_last,
        screen_width - margin
    );
}

#[test]
fn zoom_max_limited() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(100.0, 100.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_x() <= 5.0,
        "Zoom máximo no debe exceder 5.0: zoom={}",
        viewport.zoom_x()
    );
}

#[test]
fn zoom_min_limited() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(0.001, 0.001);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_x() >= 0.1,
        "Zoom mínimo debe ser al menos min_zoom (0.1): zoom={}",
        viewport.zoom_x()
    );
}

#[test]
fn zoom_min_limited_by_screen_width() {
    let mut viewport = Viewport::new();
    viewport.set_min_zoom(0.001);

    let candle_count = 50;
    let screen_width = 50.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_x() >= 0.8,
        "Con pantalla pequeña, zoom mínimo debe ser mayor: zoom={}",
        viewport.zoom_x()
    );
}

#[test]
fn zoom_max_limited_by_candle_spacing() {
    let mut viewport = Viewport::new();
    viewport.set_max_zoom(100.0);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_x() <= 5.0,
        "Zoom máximo por espaciamiento de velas no debe exceder 5.0: zoom={}",
        viewport.zoom_x()
    );
}

#[test]
fn pan_single_candle_keeps_visible() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(1.0, 1.0);

    let candle_count = 1;
    let screen_width = 1920.0;

    viewport.set_offset_x(500.0);
    viewport.clamp_pan_to_candles(candle_count, screen_width);

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

#[test]
fn set_zoom_level_respects_candle_limits() {
    let mut viewport = Viewport::new();
    viewport.set_max_zoom(100.0);
    viewport.set_min_zoom(0.001);

    let candle_count = 50;
    let screen_width = 1920.0;

    viewport.set_zoom(50.0, 50.0);
    viewport.clamp_zoom_to_candles(candle_count, screen_width);

    assert!(
        viewport.zoom_x() <= 5.0,
        "set_zoom_level + clamp_zoom_to_candles no debe exceder 5.0: zoom={}",
        viewport.zoom_x()
    );
}
