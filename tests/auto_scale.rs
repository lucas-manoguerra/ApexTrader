use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;

/// S4: Autoescala ajusta zoom_level para que el rango de precios quepa en pantalla.
#[test]
fn auto_scale_zoom_within_limits() {
    let mut viewport = Viewport::new();
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    viewport.auto_scale(range, 100, 1920.0, 1080.0);

    assert!(viewport.zoom_x() >= viewport.min_zoom());
    assert!(viewport.zoom_x() <= viewport.max_zoom());
}

/// S4: Con una sola vela, el zoom no es extremo (no se acerca a max_zoom innecesariamente).
#[test]
fn auto_scale_single_candle_reasonable_zoom() {
    let mut viewport = Viewport::new();
    let min = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let max = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido (precio igual)");

    viewport.auto_scale(range, 1, 1920.0, 1080.0);

    assert!(viewport.zoom_x() > 0.0);
    assert!(viewport.zoom_x() <= viewport.max_zoom());
}

/// S4: Después de auto_scale, el precio mínimo y máximo mapean dentro de la pantalla.
#[test]
fn auto_scale_prices_within_screen_bounds() {
    let mut viewport = Viewport::new();
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    viewport.auto_scale(range, 50, 1920.0, 1080.0);

    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(15000, 2)).expect("150.00 válido");

    let margin = 1080.0 * 0.05;
    let y_min = viewport.price_to_screen(min, screen_height, range, center_price);
    let y_max = viewport.price_to_screen(max, screen_height, range, center_price);

    let tolerance = 1.0;
    assert!(
        y_min >= margin - tolerance,
        "precio mínimo debe estar dentro de la pantalla: y_min={}, margin={}",
        y_min,
        margin
    );
    assert!(
        y_min <= screen_height - margin + tolerance,
        "precio mínimo debe estar dentro de la pantalla: y_min={}, screen_height-margin={}",
        y_min,
        screen_height - margin
    );
    assert!(
        y_max >= margin - tolerance,
        "precio máximo debe estar dentro de la pantalla: y_max={}, margin={}",
        y_max,
        margin
    );
    assert!(
        y_max <= screen_height - margin + tolerance,
        "precio máximo debe estar dentro de la pantalla: y_max={}, screen_height-margin={}",
        y_max,
        screen_height - margin
    );
}

/// S4: El centro del rango de precios mapea al centro vertical de la pantalla.
#[test]
fn auto_scale_center_price_maps_to_center_y() {
    let mut viewport = Viewport::new();
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    viewport.auto_scale(range, 50, 1920.0, 1080.0);

    let screen_height = 1080.0;
    let mid_price = Price::new(Decimal::new(15000, 2)).expect("150.00 válido");
    let y_mid = viewport.price_to_screen(mid_price, screen_height, range, mid_price);

    assert!((y_mid - screen_height / 2.0).abs() < 1.0);
}

/// S4: auto_scale solo afecta al eje Y (precio); el eje X (tiempo/velas)
/// se deja intacto.
#[test]
fn auto_scale_only_affects_y_axis() {
    let mut viewport = Viewport::new();
    viewport.set_zoom(3.0, 3.0);

    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    let zoom_x_before = viewport.zoom_x();
    let zoom_y_before = viewport.zoom_y();

    viewport.auto_scale(range, 50, 1920.0, 1080.0);

    assert!(
        (viewport.zoom_x() - zoom_x_before).abs() < f64::EPSILON,
        "auto_scale no debe modificar zoom_x: {} != {}",
        viewport.zoom_x(),
        zoom_x_before
    );
    assert!(
        (viewport.zoom_y() - zoom_y_before).abs() >= f64::EPSILON,
        "auto_scale debe recalcular zoom_y: {} == {}",
        viewport.zoom_y(),
        zoom_y_before
    );
}

/// S4: El índice de la primera vela mapea a la izquierda con margen.
/// (auto_scale solo ajusta el eje Y; el centrado horizontal del eje X lo
/// establece el llamante con set_offset_x.)
#[test]
fn auto_scale_first_candle_within_horizontal_margin() {
    let mut viewport = Viewport::new();
    let candle_count = 50usize;
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    viewport.set_offset_x(-(candle_count as f64 - 1.0) / 2.0);

    viewport.auto_scale(range, candle_count, 1920.0, 1080.0);

    let screen_width = 1920.0;
    let margin = screen_width * 0.05;

    let x_first = viewport.data_x_to_screen(0, screen_width);
    let x_last = viewport.data_x_to_screen(49, screen_width);

    assert!(
        x_first >= margin,
        "primera vela debe estar dentro del margen izquierdo"
    );
    assert!(
        x_first <= screen_width - margin,
        "primera vela debe estar dentro de la pantalla"
    );
    assert!(
        x_last >= margin,
        "última vela debe estar dentro del margen izquierdo"
    );
    assert!(
        x_last <= screen_width - margin,
        "última vela debe estar dentro de la pantalla"
    );
}

/// S4: Llamar auto_scale dos veces es idempotente en cuanto a límites.
#[test]
fn auto_scale_idempotent() {
    let mut viewport = Viewport::new();
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    viewport.auto_scale(range, 50, 1920.0, 1080.0);
    let zoom1 = viewport.zoom_x();

    viewport.auto_scale(range, 50, 1920.0, 1080.0);
    let zoom2 = viewport.zoom_x();

    assert!((zoom1 - zoom2).abs() < f64::EPSILON);
}
