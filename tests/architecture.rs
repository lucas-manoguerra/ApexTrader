use apex_trader::domain::candle::Candle;
use apex_trader::domain::chart::chart_data::ChartDataPoint;
use apex_trader::domain::chart::time_point::TimePoint;
use apex_trader::domain::chart_state::ChartState;
use apex_trader::domain::viewport::Viewport;

#[test]
fn candle_is_pure_domain_type() {
    let c = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).expect("vela válida debe aceptarse");
    assert_eq!(c.open(), 100.0);
    assert_eq!(c.high(), 105.0);
    assert_eq!(c.low(), 99.0);
    assert_eq!(c.close(), 103.0);
}

#[test]
fn chart_state_lends_data_points_and_viewport() {
    let mut chart = ChartState::new();
    let candle = Candle::new(1.0, 2.0, 0.5, 1.5, 100.0).expect("vela válida debe aceptarse");
    let tp = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
    chart
        .add_data_point(ChartDataPoint::new(candle, tp))
        .expect("punto de datos válido debe agregarse");

    let points = chart.data_points();
    assert_eq!(points.len(), 1);

    let viewport = chart.viewport();

    assert_eq!(viewport.zoom_x(), 1.0);
}

#[test]
fn viewport_is_copy_value_type() {
    let v1 = Viewport::new();
    let v2 = v1;
    assert_eq!(v1.zoom_x(), v2.zoom_x());
    assert_eq!(v1.zoom_x(), 1.0);
}

#[test]
fn chart_state_rejects_non_finite_prices() {
    let mut chart = ChartState::new();

    let result = Candle::new(f64::NAN, 2.0, 0.5, 1.5, 0.0);
    assert!(result.is_err());

    let valid = Candle::new(1.0, 2.0, 0.5, 1.5, 100.0).expect("vela válida");
    let tp = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
    assert!(chart.add_data_point(ChartDataPoint::new(valid, tp)).is_ok());
}

#[test]
fn candle_rejects_invalid_price_range() {

    let result = Candle::new(100.0, 90.0, 95.0, 95.0, 0.0);
    assert!(result.is_err());
}

#[test]
fn candle_rejects_open_out_of_range() {

    let result = Candle::new(200.0, 100.0, 50.0, 75.0, 0.0);
    assert!(result.is_err());
}

#[test]
fn candle_rejects_close_out_of_range() {

    let result = Candle::new(75.0, 100.0, 50.0, 200.0, 0.0);
    assert!(result.is_err());
}
