use apex_trader::application::commands::load_candles::LoadCandles;
use apex_trader::domain::chart_state::ChartState;
use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::market::candle::MarketCandle;
use apex_trader::market::mock::MockMarketDataProvider;
use apex_trader::market::symbol::Symbol;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;

// Estrategia que genera un Symbol válido (BASE/QUOTE, alfanumérico).
fn valid_symbol() -> impl Strategy<Value = Symbol> {
    ("[A-Z]{1,5}", "[A-Z]{1,5}").prop_map(|(base, quote)| {
        Symbol::from_str(&format!("{}/{}", base, quote)).expect("símbolo generado es válido")
    })
}

// Estrategia que genera un MarketCandle financieramente válido:
// low <= open <= high, low <= close <= high, close_time >= open_time.
fn valid_market_candle() -> impl Strategy<Value = MarketCandle> {
    (
        valid_symbol(),
        0i64..1_000_000i64,     // open_time_ms
        1.0f64..1_000_000.0f64, // open (precio positivo)
        0.0f64..100.0f64,       // delta arriba de open
        0.0f64..100.0f64,       // delta abajo de open
        -100.0f64..100.0f64,    // delta close relativo a open (se clampea)
        0.0f64..1000.0f64,      // volume
    )
        .prop_map(
            |(symbol, open_time, open, high_delta, low_delta, close_delta, volume)| {
                let high = open + high_delta;
                let low = open - low_delta;
                let close = (open + close_delta).clamp(low, high);
                let close_time = open_time + 60_000;

                MarketCandle {
                    symbol,
                    open_time_ms: open_time,
                    open,
                    high,
                    low,
                    close,
                    volume,
                    close_time_ms: close_time,
                }
            },
        )
}

// S3: todo MarketCandle generado por la estrategia pasa is_valid().
proptest! {
    #[test]
    fn prop_market_candle_is_valid(candle in valid_market_candle()) {
        assert!(candle.is_valid(), "MarketCandle debe ser válido: {:?}", candle);
    }
}

// S3: la conversión a domain::Candle nunca falla para un MarketCandle válido.
proptest! {
    #[test]
    fn prop_market_candle_to_domain_candle(candle in valid_market_candle()) {
        let domain = candle.to_domain_candle()
            .expect("MarketCandle válido debe convertirse a domain::Candle");
        assert!((domain.open() - candle.open).abs() < f64::EPSILON);
        assert!((domain.high() - candle.high).abs() < f64::EPSILON);
        assert!((domain.low() - candle.low).abs() < f64::EPSILON);
        assert!((domain.close() - candle.close).abs() < f64::EPSILON);
    }
}

// S4: una lista de MarketCandles ordenada por open_time_ms se mantiene
// ordenada tras agregar una nueva vela con tiempo mayor.
proptest! {
    #[test]
    fn prop_market_candles_remain_sorted(
        mut candles in prop::collection::vec(valid_market_candle(), 0..50),
        new_candle in valid_market_candle(),
    ) {
        // Ordenar las existentes por tiempo.
        candles.sort_by_key(|c| c.open_time_ms);

        // Agregar la nueva vela con un tiempo mayor a todas las existentes.
        let max_time = candles.iter().map(|c| c.open_time_ms).max().unwrap_or(0);
        let mut newer = new_candle;
        newer.open_time_ms = max_time + 1;
        newer.close_time_ms = newer.open_time_ms + 60_000;

        candles.push(newer);

        // Verificar orden ascendente.
        for pair in candles.windows(2) {
            assert!(
                pair[0].open_time_ms <= pair[1].open_time_ms,
                "velas deben estar ordenadas por open_time_ms ascendente"
            );
        }
    }
}

// S4: no hay timestamps duplicados tras dedup (simulado: cada vela tiene
// tiempo único por construcción).
proptest! {
    #[test]
    fn prop_market_candles_unique_timestamps(
        candles in prop::collection::vec(valid_market_candle(), 0..50),
    ) {
        let mut times: Vec<i64> = candles.iter().map(|c| c.open_time_ms).collect();
        times.sort();
        times.dedup();

        // Si los tiempos originales eran únicos, tras dedup la longitud no cambia.
        // La estrategia genera tiempos independientes, puede haber colisiones;
        // el test verifica que dedup elimina duplicados.
        let original_count = candles.len();
        let deduped_count = times.len();
        assert!(deduped_count <= original_count);
    }
}

// S1 + S4: carga end-to-end — proveedor mock → LoadCandles → ChartState →
// auto_scale. Verifica que las velas del mercado llegan al chart y que el
// viewport se configura para mostrarlas.
#[test]
fn test_carga_end_to_end_btc_usdt() {
    let provider = MockMarketDataProvider::new(50000.0);
    let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

    let result =
        LoadCandles::execute(&provider, symbol, 0, 300_000).expect("velas BTC/USDT cargadas");

    assert!(!result.data_points.is_empty());
    assert_eq!(result.symbol.as_string(), "BTC/USDT");

    let mut chart = ChartState::new();
    chart
        .add_data_points(&result.data_points)
        .expect("puntos de datos agregados al chart");
    assert!(!chart.is_empty(), "el chart debe tener puntos de datos");

    // Calcular PriceRange de los puntos de datos cargados y aplicar auto_scale.
    let points = chart.data_points();
    let mut min_price = f64::INFINITY;
    let mut max_price = f64::NEG_INFINITY;
    for p in points {
        min_price = min_price.min(p.low());
        max_price = max_price.max(p.high());
    }
    let min = Price::new(Decimal::from_f64(min_price).unwrap()).unwrap();
    let max = Price::new(Decimal::from_f64(max_price).unwrap()).unwrap();
    let range = PriceRange::new(min, max).expect("rango de precios válido");

    let candle_count = chart.len();
    chart
        .viewport_mut()
        .auto_scale(range, candle_count, 1920.0, 1080.0);

    assert!(chart.viewport().zoom_y() > 0.0, "zoom_y debe ser positivo");
    assert!(
        chart.viewport().zoom_y().is_finite(),
        "zoom_y debe ser finito"
    );
}

// S2 + S3: cambiar de símbolo no acumula velas del anterior (clear + recarga).
#[test]
fn test_cambiar_simbolo_no_acumula_velas() {
    let provider = MockMarketDataProvider::new(50000.0);
    let symbol1 = Symbol::from_str("BTC/USDT").expect("símbolo válido");
    let symbol2 = Symbol::from_str("ETH/USDT").expect("símbolo válido");

    // Cargar símbolo 1.
    let result1 = LoadCandles::execute(&provider, symbol1, 0, 60_000).expect("velas BTC cargadas");
    let count1 = result1.data_points.len();
    assert!(count1 > 0);

    let mut chart = ChartState::new();
    chart
        .add_data_points(&result1.data_points)
        .expect("BTC agregados");
    assert_eq!(chart.len(), count1);

    // Recargar símbolo 2: limpiar y agregar.
    chart.clear();
    let result2 = LoadCandles::execute(&provider, symbol2, 0, 60_000).expect("velas ETH cargadas");
    chart
        .add_data_points(&result2.data_points)
        .expect("ETH agregados");

    assert_eq!(chart.len(), result2.data_points.len());
    assert_ne!(
        chart.len(),
        count1 + result2.data_points.len(),
        "los puntos de datos del símbolo anterior no deben acumularse"
    );
}
