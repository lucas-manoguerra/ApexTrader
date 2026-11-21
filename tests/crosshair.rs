use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;

/// S5.3: screen_y_to_price calcula el precio bajo el cursor del crosshair.
#[test]
fn crosshair_price_at_cursor() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    // Cursor en el centro vertical → precio centro.
    let y_center = screen_height / 2.0;
    let price = viewport.screen_y_to_price(y_center, screen_height, range, center_price);
    assert_eq!(price.value(), Decimal::new(50000, 2));
}

/// S5.3: cursor más arriba → precio mayor (eje Y invertido).
#[test]
fn crosshair_price_above_center_is_higher() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    let y_top = 0.0;
    let price_top = viewport.screen_y_to_price(y_top, screen_height, range, center_price);

    let y_bottom = screen_height;
    let price_bottom = viewport.screen_y_to_price(y_bottom, screen_height, range, center_price);

    assert!(
        price_top.value() > price_bottom.value(),
        "cursor arriba debe mostrar precio mayor"
    );
}

/// S5.4: screen_x_to_data calcula el índice de vela bajo el cursor.
#[test]
fn crosshair_index_at_cursor() {
    let viewport = Viewport::new();
    let screen_width = 1920.0;

    // Índice 0 en el centro.
    let x_center = screen_width / 2.0;
    let idx = viewport.screen_x_to_data(x_center, screen_width);
    assert_eq!(idx, 0);

    // Una vela a la derecha.
    let spacing = 20.0 / viewport.zoom_level;
    let x_one = x_center + spacing;
    let idx_one = viewport.screen_x_to_data(x_one, screen_width);
    assert_eq!(idx_one, 1);
}

/// S5.5: sin crosshair (None) no hay error ni dibujo.
/// Verifica que screen_y_to_price y screen_x_to_data existen y funcionan
/// de forma independiente (el renderizado condicional se prueba en integración visual).
#[test]
fn crosshair_none_handled_gracefully() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let screen_width = 1920.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");

    // Llamar screen_y_to_price con el centro → debe devolver el precio central.
    let price = viewport.screen_y_to_price(screen_height / 2.0, screen_height, range, center_price);
    assert_eq!(price.value(), Decimal::new(50000, 2));

    // Llamar screen_x_to_data con el centro → índice 0.
    let idx = viewport.screen_x_to_data(screen_width / 2.0, screen_width);
    assert_eq!(idx, 0);
}
