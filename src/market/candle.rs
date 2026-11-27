use crate::domain::candle::Candle;
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::chart::time_point::TimePoint;
use crate::market::errors::MarketError;
use crate::market::symbol::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub struct MarketCandle {
    pub symbol: Symbol,
    pub open_time_ms: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time_ms: i64,
}

impl MarketCandle {
    pub fn to_domain_candle(&self) -> Result<Candle, MarketError> {
        Candle::new(self.open, self.high, self.low, self.close, self.volume)
            .map_err(|e| MarketError::InvalidResponse(format!("vela inválida: {e}")))
    }

    pub fn to_chart_data_point(&self) -> Result<ChartDataPoint, MarketError> {
        let candle = self.to_domain_candle()?;
        let interval_ms = self
            .close_time_ms
            .checked_sub(self.open_time_ms)
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                MarketError::InvalidResponse(format!(
                    "intervalo temporal inválido: open={}, close={}",
                    self.open_time_ms, self.close_time_ms
                ))
            })?;
        let time_point = TimePoint::new(self.open_time_ms, self.close_time_ms, interval_ms)
            .ok_or_else(|| {
                MarketError::InvalidResponse(format!(
                    "punto temporal inválido: open={}, close={}, interval={}",
                    self.open_time_ms, self.close_time_ms, interval_ms
                ))
            })?;
        Ok(ChartDataPoint::new(candle, time_point))
    }

    pub fn is_valid(&self) -> bool {
        self.open.is_finite()
            && self.high.is_finite()
            && self.low.is_finite()
            && self.close.is_finite()
            && self.volume.is_finite()
            && self.volume >= 0.0
            && self.low <= self.high
            && self.low <= self.open
            && self.open <= self.high
            && self.low <= self.close
            && self.close <= self.high
            && self
                .close_time_ms
                .checked_sub(self.open_time_ms)
                .is_some_and(|value| value > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn valid_market_candle(open_time_ms: i64, close_time_ms: i64) -> MarketCandle {
        MarketCandle {
            symbol: Symbol::from_str("BTC/USDT").expect("símbolo válido"),
            open_time_ms,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 10.5,
            close_time_ms,
        }
    }

    #[test]
    fn market_candle_to_domain_candle() {
        let candle = valid_market_candle(1000, 2000);
        let domain = candle.to_domain_candle().expect("vela válida");
        assert!((domain.open() - 50000.0).abs() < f64::EPSILON);
        assert!((domain.high() - 51000.0).abs() < f64::EPSILON);
        assert!((domain.low() - 49000.0).abs() < f64::EPSILON);
        assert!((domain.close() - 50500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn market_candle_rejects_invalid_prices() {
        let mut candle = valid_market_candle(1000, 2000);
        candle.high = 49000.0;
        assert!(!candle.is_valid());
        assert!(candle.to_domain_candle().is_err());
    }

    #[test]
    fn market_candle_rejects_zero_duration() {
        let candle = valid_market_candle(1000, 1000);
        assert!(!candle.is_valid());
        assert!(candle.to_chart_data_point().is_err());
    }

    #[test]
    fn market_candle_rejects_inverted_time_range() {
        let candle = valid_market_candle(2000, 1000);
        assert!(!candle.is_valid());
        assert!(candle.to_chart_data_point().is_err());
    }

    #[test]
    fn market_candle_rejects_timestamp_subtraction_overflow() {
        let candle = valid_market_candle(i64::MIN, i64::MAX);
        assert!(!candle.is_valid());
        assert!(candle.to_chart_data_point().is_err());
    }

    #[test]
    fn market_candle_converts_positive_duration() {
        let candle = valid_market_candle(1000, 2000);
        let point = candle.to_chart_data_point().expect("intervalo válido");
        assert_eq!(point.time_point.open_time_ms(), 1000);
        assert_eq!(point.time_point.close_time_ms(), 2000);
        assert_eq!(point.time_point.interval_ms(), 1000);
    }
}
