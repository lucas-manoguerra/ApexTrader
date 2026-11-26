




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
        let candle = Candle::new(self.open, self.high, self.low, self.close, self.volume)
            .map_err(|e| MarketError::InvalidResponse(format!("vela inválida: {e}")))?;
        let interval_ms = self.close_time_ms - self.open_time_ms;
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
            && self.close_time_ms >= self.open_time_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn market_candle_to_domain_candle() {
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
        let candle = MarketCandle {
            symbol,
            open_time_ms: 1000,
            open: 50000.0,
            high: 51000.0,
            low: 49000.0,
            close: 50500.0,
            volume: 10.5,
            close_time_ms: 2000,
        };
        let domain = candle.to_domain_candle().expect("vela válida");
        assert!((domain.open() - 50000.0).abs() < f64::EPSILON);
        assert!((domain.high() - 51000.0).abs() < f64::EPSILON);
        assert!((domain.low() - 49000.0).abs() < f64::EPSILON);
        assert!((domain.close() - 50500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn market_candle_rejects_invalid() {
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
        let candle = MarketCandle {
            symbol,
            open_time_ms: 2000,
            open: 50000.0,
            high: 49000.0,
            low: 49000.0,
            close: 49500.0,
            volume: 10.5,
            close_time_ms: 3000,
        };
        assert!(!candle.is_valid());
        assert!(candle.to_domain_candle().is_err());
    }
}
