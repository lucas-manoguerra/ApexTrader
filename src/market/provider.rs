use crate::market::candle::MarketCandle;
use crate::market::cancellation::CancellationToken;
use crate::market::errors::MarketError;
use crate::market::symbol::Symbol;

pub trait MarketDataProvider: Send + Sync {
    fn name(&self) -> &str;

    fn fetch_candles(
        &self,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<MarketCandle>, MarketError>;

    /// Cancellation-aware entry point for workers.
    ///
    /// The default preserves compatibility but can only check before and
    /// after the legacy call. Network providers must override it and use
    /// finite I/O timeouts plus checks between bounded operations.
    fn fetch_candles_cancellable(
        &self,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
        cancellation: &CancellationToken,
    ) -> Result<Vec<MarketCandle>, MarketError> {
        cancellation.check()?;
        let result = self.fetch_candles(symbol, from_ms, to_ms);
        cancellation.check()?;
        result
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeMarketUpdate {
    pub candle: MarketCandle,
    pub is_closed: bool,
}

impl RealtimeMarketUpdate {
    pub fn validate_symbol(&self, expected: &Symbol) -> Result<(), MarketError> {
        if &self.candle.symbol == expected {
            Ok(())
        } else {
            Err(MarketError::InvalidResponse(format!(
                "símbolo realtime inesperado: esperado {expected}, recibido {}",
                self.candle.symbol
            )))
        }
    }
}

pub trait RealtimeMarketDataProvider: Send {
    fn name(&self) -> &str;

    fn next_update(&mut self) -> Result<RealtimeMarketUpdate, MarketError>;

    /// Cancellation-aware entry point for workers.
    ///
    /// Providers with blocking reads must override this method and bound
    /// those reads with finite timeouts so cancellation can be observed.
    fn next_update_cancellable(
        &mut self,
        cancellation: &CancellationToken,
    ) -> Result<RealtimeMarketUpdate, MarketError> {
        cancellation.check()?;
        let result = self.next_update();
        cancellation.check()?;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn update(symbol: &str) -> RealtimeMarketUpdate {
        RealtimeMarketUpdate {
            candle: MarketCandle {
                symbol: Symbol::from_str(symbol).expect("símbolo válido"),
                open_time_ms: 0,
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 103.0,
                volume: 1_000.0,
                close_time_ms: 60_000,
            },
            is_closed: false,
        }
    }

    #[test]
    fn realtime_update_accepts_the_expected_symbol() {
        let expected = Symbol::from_str("BTC/USDT").expect("símbolo válido");
        assert!(update("BTC/USDT").validate_symbol(&expected).is_ok());
    }

    #[test]
    fn realtime_update_rejects_a_different_symbol() {
        let expected = Symbol::from_str("ETH/USDT").expect("símbolo válido");
        let error = update("BTC/USDT")
            .validate_symbol(&expected)
            .expect_err("símbolo inesperado");
        assert!(
            matches!(error, MarketError::InvalidResponse(message) if message.contains("BTC/USDT") && message.contains("ETH/USDT"))
        );
    }
}
