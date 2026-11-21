use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;

/// S3.1: data_x_to_screen convierte índice de vela a píxel X de pantalla.
#[test]
fn data_x_to_screen_returns_pixel_x() {
    let viewport = Viewport::new();
    let screen_width = 1920.0;

    // Índice 0 en el centro con zoom 1.0
    let x = viewport.data_x_to_screen(0, screen_width);
    assert!((x - screen_width / 2.0).abs() < f64::EPSILON);
}

/// S3.1: data_x_to_screen respeta el zoom y el offset.
#[test]
fn data_x_to_screen_respects_zoom_and_offset() {
    let mut viewport = Viewport::new();
    viewport.set_zoom_level(2.0);
    viewport.set_offset_x(5.0);

    let screen_width = 1920.0;
    let x0 = viewport.data_x_to_screen(0, screen_width);
    let x1 = viewport.data_x_to_screen(1, screen_width);

    // Con zoom 2.0, candle_spacing = 20.0 * 2.0 = 40.0. El offset_x se aplica
    // por igual a ambas velas, por lo que la diferencia entre x1 y x0 es solo
    // candle_spacing (el offset se cancela al restar).
    let expected_spacing = 20.0 * 2.0;
    assert!((x1 - x0 - expected_spacing).abs() < f64::EPSILON);
}

/// S3.2: screen_x_to_data es el inverso de data_x_to_screen.
#[test]
fn screen_x_to_data_is_inverse_of_data_x_to_screen() {
    let viewport = Viewport::new();
    let screen_width = 1920.0;

    for i in 0..100 {
        let screen_x = viewport.data_x_to_screen(i, screen_width);
        let recovered = viewport.screen_x_to_data(screen_x, screen_width);
        assert_eq!(
            recovered, i,
            "screen_x_to_data(data_x_to_screen({})) should be {}",
            i, i
        );
    }
}

/// S3.3: price_to_screen convierte Price a píxel Y de pantalla.
#[test]
fn price_to_screen_returns_pixel_y() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    let price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let y = viewport.price_to_screen(price, screen_height, range, center_price);

    // Precio igual al centro → Y en el centro de la pantalla
    assert!((y - screen_height / 2.0).abs() < f64::EPSILON);
}

/// S3.3: price_to_screen respeta el eje Y invertido (precio mayor → Y menor).
#[test]
fn price_to_screen_inverts_y_axis() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    let y_low = viewport.price_to_screen(
        Price::new(Decimal::new(40000, 2)).expect("400.00 válido"),
        screen_height,
        range,
        center_price,
    );
    let y_high = viewport.price_to_screen(
        Price::new(Decimal::new(60000, 2)).expect("600.00 válido"),
        screen_height,
        range,
        center_price,
    );

    // Precio más bajo → Y más grande (más abajo en pantalla)
    assert!(y_low > y_high);
}

/// S3.4: screen_y_to_price es el inverso de price_to_screen.
#[test]
fn screen_y_to_price_is_inverse_of_price_to_screen() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    for price_val in [41000, 45000, 50000, 55000, 59000] {
        let price = Price::new(Decimal::new(price_val, 2)).expect("precio válido");
        let y = viewport.price_to_screen(price, screen_height, range, center_price);
        let recovered = viewport.screen_y_to_price(y, screen_height, range, center_price);

        assert!(
            (recovered.value() - price.value()).abs() < Decimal::new(1, 2),
            "screen_y_to_price(price_to_screen({})) should be ≈ {}",
            price_val,
            price_val
        );
    }
}

/// S3.4: screen_y_to_price devuelve center_price como fallback si el resultado es inválido.
#[test]
fn screen_y_to_price_falls_back_to_center() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    // Y muy abajo de la pantalla produce precio muy bajo (negativo → inválido para Price)
    let y_far_below = 10000.0;
    let recovered = viewport.screen_y_to_price(y_far_below, screen_height, range, center_price);
    assert_eq!(recovered, center_price);
}
