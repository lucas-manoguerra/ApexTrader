use crate::application::commands::load_candles::{LoadCandles, LoadCandlesResult};
use crate::domain::chart_state::ChartState;
use crate::errors::ChartError;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

#[derive(Debug, thiserror::Error)]
pub enum LoadHistoricalDataError {
    #[error(transparent)]
    Market(#[from] MarketError),
    #[error(transparent)]
    Chart(#[from] ChartError),
}

pub struct LoadHistoricalData;

impl LoadHistoricalData {
    pub fn execute(
        chart: &mut ChartState,
        provider: &dyn MarketDataProvider,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<LoadCandlesResult, LoadHistoricalDataError> {
        let result = match LoadCandles::execute(provider, symbol, from_ms, to_ms) {
            Ok(result) => result,
            Err(error) => {
                chart.clear();
                return Err(error.into());
            }
        };

        chart.clear();
        chart.add_data_points(&result.data_points)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;
    use crate::domain::chart::time_point::TimePoint;
    use crate::market::candle::MarketCandle;
    use crate::market::mock::MockMarketDataProvider;
    use std::str::FromStr;

    struct FailingMarketDataProvider;

    impl MarketDataProvider for FailingMarketDataProvider {
        fn name(&self) -> &str {
            "failing-provider"
        }

        fn fetch_candles(
            &self,
            _symbol: Symbol,
            _from_ms: i64,
            _to_ms: i64,
        ) -> Result<Vec<MarketCandle>, MarketError> {
            Err(MarketError::Network("provider unavailable".to_string()))
        }
    }

    fn populated_chart() -> ChartState {
        let mut chart = ChartState::new();
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).expect("vela válida");
        let time = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        chart
            .add_data_point(crate::domain::chart::chart_data::ChartDataPoint::new(
                candle, time,
            ))
            .expect("punto válido");
        chart
    }

    #[test]
    fn historical_load_replaces_chart_data_with_validated_provider_data() {
        let mut chart = populated_chart();
        let provider = MockMarketDataProvider::new(50_000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result =
            LoadHistoricalData::execute(&mut chart, &provider, symbol, 0, 120_000)
                .expect("carga histórica válida");

        assert_eq!(chart.len(), result.data_points.len());
        assert_eq!(
            chart.data_points()[0].time_point.open_time_ms(),
            result.data_points[0].time_point.open_time_ms()
        );
        assert_eq!(chart.data_points()[0].close(), result.data_points[0].close());
        assert!(result.is_clean());
    }

    #[test]
    fn failed_historical_load_clears_previous_chart_data() {
        let mut chart = populated_chart();
        let provider = FailingMarketDataProvider;
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadHistoricalData::execute(&mut chart, &provider, symbol, 0, 60_000);

        assert!(matches!(
            result,
            Err(LoadHistoricalDataError::Market(MarketError::Network(_)))
        ));
        assert!(chart.is_empty());
    }

    #[test]
    fn invalid_historical_range_clears_previous_chart_data() {
        let mut chart = populated_chart();
        let provider = MockMarketDataProvider::new(50_000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadHistoricalData::execute(&mut chart, &provider, symbol, 120_000, 0);

        assert!(matches!(
            result,
            Err(LoadHistoricalDataError::Market(
                MarketError::InvalidResponse(_)
            ))
        ));
        assert!(chart.is_empty());
    }
}
