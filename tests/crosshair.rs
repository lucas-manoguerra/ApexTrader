use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;


#[test]
fn crosshair_price_at_cursor() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");


    let y_center = screen_height / 2.0;
    let price = viewport.screen_y_to_price(y_center, screen_height, range, center_price);
    assert_eq!(price.value(), Decimal::new(50000, 2));
}


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


#[test]
fn crosshair_index_at_cursor() {
    let viewport = Viewport::new();
    let screen_width = 1920.0;


    let x_center = screen_width / 2.0;
    let idx = viewport.screen_x_to_data(x_center, screen_width, 1000);
    assert_eq!(idx, 0);


    let spacing = 20.0 / viewport.zoom_x();
    let x_one = x_center + spacing;
    let idx_one = viewport.screen_x_to_data(x_one, screen_width, 1000);
    assert_eq!(idx_one, 1);
}




#[test]
fn crosshair_none_handled_gracefully() {
    let viewport = Viewport::new();
    let screen_height = 1080.0;
    let screen_width = 1920.0;
    let center_price = Price::new(Decimal::new(50000, 2)).expect("500.00 válido");
    let min = Price::new(Decimal::new(40000, 2)).expect("400.00 válido");
    let max = Price::new(Decimal::new(60000, 2)).expect("600.00 válido");
    let range = PriceRange::new(min, max).expect("rango válido");


    let price = viewport.screen_y_to_price(screen_height / 2.0, screen_height, range, center_price);
    assert_eq!(price.value(), Decimal::new(50000, 2));


    let idx = viewport.screen_x_to_data(screen_width / 2.0, screen_width, 1000);
    assert_eq!(idx, 0);
}
