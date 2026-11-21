use apex_trader::domain::candle::Candle;
use apex_trader::domain::chart_state::ChartState;
use apex_trader::domain::viewport::Viewport;

/// Regla 1: Candle es un tipo puro de dominio (datos OHLC).
/// No debe conocer dibujo, ventanas ni GPU.
#[test]
fn candle_is_pure_domain_type() {
    let c = Candle {
        open: 100.0,
        high: 105.0,
        low: 99.0,
        close: 103.0,
    };
    assert_eq!(c.open, 100.0);
    assert_eq!(c.high, 105.0);
    assert_eq!(c.low, 99.0);
    assert_eq!(c.close, 103.0);
}

/// Regla 2: ChartState presta velas y viewport; no las cede.
/// El llamante recibe &[Candle] y &Viewport (borrowing), no ownership.
#[test]
fn chart_state_lends_candles_and_viewport() {
    let mut chart = ChartState::new();
    chart
        .add_candle(Candle {
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
        })
        .expect("vela válida debe aceptarse");

    let candles = chart.candles();
    assert_eq!(candles.len(), 1);

    let viewport = chart.viewport();
    // El viewport sigue accesible tras prestar velas (ninguna se movió).
    assert_eq!(viewport.zoom_level, 1.0);
}

/// Regla 3: Viewport es un tipo de valor (Copy), no depende de GPU ni ventana.
#[test]
fn viewport_is_copy_value_type() {
    let v1 = Viewport::new();
    let v2 = v1; // Copy: v1 sigue válido
    assert_eq!(v1.zoom_level, v2.zoom_level);
    assert_eq!(v1.zoom_level, 1.0);
}

/// Regla 4: ChartState rechaza velas con precios no finitos (integridad de datos).
#[test]
fn chart_state_rejects_non_finite_prices() {
    let mut chart = ChartState::new();
    let result = chart.add_candle(Candle {
        open: f64::NAN,
        high: 2.0,
        low: 0.5,
        close: 1.5,
    });
    assert!(result.is_err());
}

/// Regla 5: ChartState rechaza rangos de precio inválidos (low > high).
#[test]
fn chart_state_rejects_invalid_price_range() {
    let mut chart = ChartState::new();
    let result = chart.add_candle(Candle {
        open: 100.0,
        high: 90.0, // high < open, inválido
        low: 95.0,
        close: 95.0,
    });
    assert!(result.is_err());
}
