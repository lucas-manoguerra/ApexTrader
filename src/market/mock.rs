//! Proveedor mock de datos de mercado.
//!
//! Genera velas sintéticas determinísticas para pruebas sin red. No
//! se usa en producción; permite desarrollar el chart engine sin depender
//! de un exchange real.

use crate::market::candle::MarketCandle;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

/// Proveedor sintético que genera velas con un precio base y una variación
/// sinusoidal.
#[derive(Debug, Clone, Copy, Default)]
pub struct MockMarketDataProvider {
    /// Precio base alrededor del cual fluctúan las velas.
    pub base_price: f64,
    /// Intervalo entre velas en milisegundos (ej. 60_000 = 1 minuto).
    pub interval_ms: i64,
}

impl MockMarketDataProvider {
    /// Crea un proveedor mock con precio base dado.
    pub fn new(base_price: f64) -> Self {
        Self {
            base_price,
            interval_ms: 60_000,
        }
    }
}

impl MarketDataProvider for MockMarketDataProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn fetch_candles(
        &self,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<MarketCandle>, MarketError> {
        if from_ms > to_ms {
            return Err(MarketError::InvalidResponse {
                reason: format!("from_ms ({from_ms}) > to_ms ({to_ms})"),
            });
        }

        let mut candles = Vec::new();
        let mut t = from_ms;
        let mut i = 0;

        while t <= to_ms {
            let change = (i as f64 * 0.1).sin() * 5.0;
            let open = self.base_price + change;
            let close = open + change * 0.7;
            let high = open.max(close) + 2.0;
            let low = open.min(close) - 2.0;
            let volume = 10.0 + (i as f64 * 0.5).sin().abs() * 5.0;

            let candle = MarketCandle {
                symbol,
                open_time_ms: t,
                open,
                high,
                low,
                close,
                volume,
                close_time_ms: t + self.interval_ms - 1,
            };

            if candle.is_valid() {
                candles.push(candle);
            }

            t += self.interval_ms;
            i += 1;
        }

        Ok(candles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_generates_ordered_candles() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let candles = provider
            .fetch_candles(symbol, 0, 300_000)
            .expect("velas generadas");

        // 5 velas (0, 60000, 120000, 180000, 240000, 300000)
        assert!(!candles.is_empty());
        for w in candles.windows(2) {
            assert!(
                w[0].open_time_ms < w[1].open_time_ms,
                "velas ordenadas por tiempo ascendente"
            );
        }
        for c in &candles {
            assert!(c.is_valid(), "vela válida: {:?}", c);
        }
    }

    #[test]
    fn mock_rejects_inverted_range() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let err = provider
            .fetch_candles(symbol, 100_000, 50_000)
            .expect_err("rango invertido debe fallar");
        assert!(matches!(err, MarketError::InvalidResponse { .. }));
    }
}
