use crate::market::candle::MarketCandle;
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeMarketUpdate {

    pub candle: MarketCandle,

    pub is_closed: bool,
}

pub trait RealtimeMarketDataProvider: Send {

    fn name(&self) -> &str;

    fn next_update(&mut self) -> Result<RealtimeMarketUpdate, MarketError>;
}
