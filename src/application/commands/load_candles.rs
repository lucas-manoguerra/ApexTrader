use crate::domain::chart::chart_data::ChartDataPoint;
use crate::market::cancellation::CancellationToken;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

#[derive(Debug, Clone)]
pub struct IngestionRejection {
    pub index: usize,
    pub open_time_ms: i64,
    pub reason: String,
}

#[derive(Debug)]
pub struct LoadCandlesResult {
    pub data_points: Vec<ChartDataPoint>,
    pub rejected: Vec<IngestionRejection>,
    pub symbol: Symbol,
    pub from_ms: i64,
    pub to_ms: i64,
}

impl LoadCandlesResult {
    pub fn is_clean(&self) -> bool {
        self.rejected.is_empty()
    }

    pub fn total_received(&self) -> usize {
        self.data_points.len() + self.rejected.len()
    }
}

pub struct LoadCandles;

impl LoadCandles {
    pub fn execute(
        provider: &dyn MarketDataProvider,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<LoadCandlesResult, MarketError> {
        Self::execute_cancellable(
            provider,
            symbol,
            from_ms,
            to_ms,
            &CancellationToken::new(),
        )
    }

    pub fn execute_cancellable(
        provider: &dyn MarketDataProvider,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
        cancellation: &CancellationToken,
    ) -> Result<LoadCandlesResult, MarketError> {
        if from_ms > to_ms {
            return Err(MarketError::InvalidResponse(format!(
                "rango invertido: from_ms ({from_ms}) > to_ms ({to_ms})"
            )));
        }

        let market_candles = provider.fetch_candles_cancellable(
            symbol.clone(),
            from_ms,
            to_ms,
            cancellation,
        )?;

        let mut data_points = Vec::with_capacity(market_candles.len());
        let mut rejected = Vec::new();
        let mut last_open_time: Option<i64> = None;

        for (index, mc) in market_candles.iter().enumerate() {
            if let Some(last) = last_open_time
                && mc.open_time_ms <= last
            {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "orden cronológico invertido o duplicado: anterior={}, actual={}",
                        last, mc.open_time_ms
                    ),
                });
                continue;
            }

            last_open_time = Some(mc.open_time_ms);

            if mc.symbol != symbol {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "símbolo inconsistente: esperado {}, recibido {}",
                        symbol.as_string(),
                        mc.symbol.as_string()
                    ),
                });
                continue;
            }

            if mc.open_time_ms < from_ms || mc.open_time_ms > to_ms {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "timestamp fuera del rango solicitado {}..={}",
                        from_ms, to_ms
                    ),
                });
                continue;
            }

            match mc.to_chart_data_point() {
                Ok(dp) => data_points.push(dp),
                Err(e) => {
                    rejected.push(IngestionRejection {
                        index,
                        open_time_ms: mc.open_time_ms,
                        reason: format!("vela inválida: {e}"),
                    });
                }
            }
        }

        Ok(LoadCandlesResult {
            data_points,
            rejected,
            symbol,
            from_ms,
            to_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::candle::MarketCandle;
    use crate::market::mock::MockMarketDataProvider;
    use std::str::FromStr;

    struct SequenceMarketDataProvider {
        candles: Vec<MarketCandle>,
    }

    impl MarketDataProvider for SequenceMarketDataProvider {
        fn name(&self) -> &str {
            "sequence-provider"
        }

        fn fetch_candles(
            &self,
            _symbol: Symbol,
            _from_ms: i64,
            _to_ms: i64,
        ) -> Result<Vec<MarketCandle>, MarketError> {
            Ok(self.candles.clone())
        }
    }

    fn market_candle(symbol: &Symbol, open_time_ms: i64) -> MarketCandle {
        MarketCandle {
            symbol: symbol.clone(),
            open_time_ms,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 101.0,
            volume: 10.0,
            close_time_ms: open_time_ms + 60_000,
        }
    }

    #[test]
    fn load_candles_from_mock_provider() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadCandles::execute(&provider, symbol, 0, 120_000).expect("velas cargadas");

        assert!(!result.data_points.is_empty());
        assert_eq!(result.symbol.as_string(), "BTC/USDT");
        assert_eq!(result.from_ms, 0);
        assert_eq!(result.to_ms, 120_000);
        assert!(
            result.is_clean(),
            "no debería haber rechazos con datos mock válidos"
        );

        for dp in &result.data_points {
            assert!(dp.open().is_finite());
            assert!(dp.high().is_finite());
            assert!(dp.low().is_finite());
            assert!(dp.close().is_finite());
            assert!(dp.time_point.open_time_ms() >= 0);
        }
    }

    #[test]
    fn load_candles_preserves_timestamps() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadCandles::execute(&provider, symbol, 0, 120_000).expect("velas cargadas");

        assert_eq!(result.data_points[0].time_point.open_time_ms(), 0);

        for w in result.data_points.windows(2) {
            assert!(
                w[0].time_point.open_time_ms() < w[1].time_point.open_time_ms(),
                "puntos de datos ordenados por tiempo ascendente"
            );
        }
    }

    #[test]
    fn invalid_candle_does_not_hide_a_later_out_of_order_timestamp() {
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
        let first = market_candle(&symbol, 0);
        let mut invalid = market_candle(&symbol, 60_000);
        invalid.high = 99.0;
        let first_out_of_order = market_candle(&symbol, 30_000);
        let second_out_of_order = market_candle(&symbol, 45_000);
        let provider = SequenceMarketDataProvider {
            candles: vec![first, invalid, first_out_of_order, second_out_of_order],
        };

        let result = LoadCandles::execute(&provider, symbol, 0, 60_000)
            .expect("la ingestión debe informar rechazos sin abortar");

        assert_eq!(result.data_points.len(), 1);
        assert_eq!(result.rejected.len(), 3);
        assert!(result.rejected[0].reason.contains("vela inválida"));
        assert!(
            result.rejected[1]
                .reason
                .contains("orden cronológico invertido")
        );
        assert!(
            result.rejected[2]
                .reason
                .contains("orden cronológico invertido")
        );
    }

    #[test]
    fn load_candles_empty_range() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let err = LoadCandles::execute(&provider, symbol, 100_000, 50_000)
            .expect_err("rango invertido debe fallar");
        assert!(matches!(err, MarketError::InvalidResponse { .. }));
    }
}
