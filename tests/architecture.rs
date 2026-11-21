use apex_trader::domain::candle::Candle;
use apex_trader::domain::chart_state::ChartState;
use apex_trader::domain::viewport::Viewport;

/// Regla 1: Candle es un tipo puro de dominio (datos OHLC).
/// No debe conocer dibujo, ventanas ni GPU.
#[test]
fn candle_is_pure_domain_type() {
    let c = Candle::new(100.0, 105.0, 99.0, 103.0).expect("vela válida debe aceptarse");
    assert_eq!(c.open(), 100.0);
    assert_eq!(c.high(), 105.0);
    assert_eq!(c.low(), 99.0);
    assert_eq!(c.close(), 103.0);
}

/// Regla 2: ChartState presta velas y viewport; no las cede.
/// El llamante recibe &[Candle] y &Viewport (borrowing), no ownership.
#[test]
fn chart_state_lends_candles_and_viewport() {
    let mut chart = ChartState::new();
    chart
        .add_candle(Candle::new(1.0, 2.0, 0.5, 1.5).expect("vela válida debe aceptarse"))
        .expect("vela válida debe agregarse");

    let candles = chart.candles();
    assert_eq!(candles.len(), 1);

    let viewport = chart.viewport();
    // El viewport sigue accesible tras prestar velas (ninguna se movió).
    assert_eq!(viewport.zoom_level(), 1.0);
}

/// Regla 3: Viewport es un tipo de valor (Copy), no depende de GPU ni ventana.
#[test]
fn viewport_is_copy_value_type() {
    let v1 = Viewport::new();
    let v2 = v1; // Copy: v1 sigue válido
    assert_eq!(v1.zoom_level(), v2.zoom_level());
    assert_eq!(v1.zoom_level(), 1.0);
}

/// Regla 4: ChartState rechaza velas con precios no finitos (integridad de datos).
#[test]
fn chart_state_rejects_non_finite_prices() {
    let mut chart = ChartState::new();
    // Candle::new() rechaza precios no finitos antes de llegar a add_candle.
    let result = Candle::new(f64::NAN, 2.0, 0.5, 1.5);
    assert!(result.is_err());

    // Si la vela se crea válida, add_candle la acepta.
    let valid = Candle::new(1.0, 2.0, 0.5, 1.5).expect("vela válida");
    assert!(chart.add_candle(valid).is_ok());
}

/// Regla 5: Candle rechaza rangos de precio inválidos (low > high).
#[test]
fn candle_rejects_invalid_price_range() {
    // low (95.0) > high (90.0): inválido
    let result = Candle::new(100.0, 90.0, 95.0, 95.0);
    assert!(result.is_err());
}

/// Regla 6: Candle rechaza open fuera de [low, high].
#[test]
fn candle_rejects_open_out_of_range() {
    // open (200.0) > high (100.0): inválido
    let result = Candle::new(200.0, 100.0, 50.0, 75.0);
    assert!(result.is_err());
}

/// Regla 7: Candle rechaza close fuera de [low, high].
#[test]
fn candle_rejects_close_out_of_range() {
    // close (200.0) > high (100.0): inválido
    let result = Candle::new(75.0, 100.0, 50.0, 200.0);
    assert!(result.is_err());
}
