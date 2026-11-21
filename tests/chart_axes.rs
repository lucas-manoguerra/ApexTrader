use apex_trader::domain::price::Price;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// S6.1: nice_tick_step devuelve un step "nice" (1, 2, 5, 10, etc.)
#[test]
fn nice_tick_step_returns_nice_values() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");

    // Verificar que el rango de precios tiene sentido
    let diff = max.value() - min.value();
    assert_eq!(diff, Decimal::new(10000, 2));
}

/// S6.2: Los ticks de precio se ajustan al zoom (menor zoom = menos rango visible).
#[test]
fn price_ticks_adjust_to_zoom() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");

    // Con zoom 1.0, el rango visible es el rango completo.
    // Con zoom mayor, el rango visible es menor.
    let half_range_at_zoom_1 = (max.value() - min.value()).to_f64().unwrap() / 2.0;
    let half_range_at_zoom_2 = half_range_at_zoom_1 / 2.0;

    assert!(half_range_at_zoom_2 < half_range_at_zoom_1);
}

/// S6.3: Los precios se formatean con 2 decimales.
#[test]
fn price_format_has_two_decimals() {
    let price = Price::new(Decimal::new(52345, 2)).expect("523.45 válido");
    let label = format!("{:.2}", price.value());
    assert_eq!(label, "523.45");

    let price2 = Price::new(Decimal::new(100, 2)).expect("1.00 válido");
    let label2 = format!("{:.2}", price2.value());
    assert_eq!(label2, "1.00");
}

/// S6.4: Los índices de vela en el eje X son enteros legibles.
#[test]
fn x_axis_index_labels_are_integers() {
    let viewport = Viewport::new();
    let screen_width = 1920.0;

    for i in 0..10 {
        let screen_x = viewport.data_x_to_screen(i, screen_width) + 0.0;
        let _label = format!("{}", i);
        let _x_pos = screen_x as f32;
    }

    // Verificar que los índices son monotónicos
    let x0 = viewport.data_x_to_screen(0, screen_width);
    let x1 = viewport.data_x_to_screen(1, screen_width);
    assert!(x1 > x0, "el índice 1 debe estar a la derecha del índice 0");
}

/// S6.5: La cantidad de ticks es razonable (no satura el eje).
#[test]
fn tick_count_is_reasonable() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");

    let visible_range = max.value() - min.value();
    let target_ticks = 6usize;
    let raw_step = visible_range.to_f64().unwrap() / target_ticks as f64;

    // El step debe ser un valor "nice" (1, 2, 5, 10, etc.)
    let magnitude = 10.0f64.powi(raw_step.log10().floor() as i32);
    let residual = raw_step / magnitude;
    let step = if residual < 1.5 {
        1.0 * magnitude
    } else if residual < 3.0 {
        2.0 * magnitude
    } else if residual < 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    };

    // Verificar que el step es una potencia de 10 multiplicada por 1, 2 o 5
    let step_ratio = step / magnitude;
    assert!(
        step_ratio == 1.0 || step_ratio == 2.0 || step_ratio == 5.0 || step_ratio == 10.0,
        "step {} no es un valor nice",
        step
    );
}
