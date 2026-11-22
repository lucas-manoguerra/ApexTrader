//! Velas de mercado con metadatos de símbolo y timestamp.
//!
//! `MarketCandle` extiende la vela de dominio (`domain::Candle`) con
//! símbolo, timestamp y volumen, para ordenar y dedup datos del mercado.

use crate::domain::candle::Candle;
use crate::market::errors::MarketError;
use crate::market::symbol::Symbol;

/// Vela de mercado con metadatos de símbolo y timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct MarketCandle {
    /// Símbolo al que pertenece la vela.
    pub symbol: Symbol,
    /// Timestamp de apertura en milisegundos Unix (UTC).
    pub open_time_ms: i64,
    /// Precio de apertura.
    pub open: f64,
    /// Precio máximo.
    pub high: f64,
    /// Precio mínimo.
    pub low: f64,
    /// Precio de cierre.
    pub close: f64,
    /// Volumen negociado en el período.
    pub volume: f64,
    /// Timestamp de cierre en milisegundos Unix (UTC).
    pub close_time_ms: i64,
}

impl MarketCandle {
    /// Convierte a `domain::Candle` (solo OHLC, sin metadatos).
    pub fn to_domain_candle(&self) -> Result<Candle, MarketError> {
        Candle::new(self.open, self.high, self.low, self.close)
            .map_err(|e| MarketError::InvalidResponse(format!("vela inválida: {e}")))
    }

    /// Devuelve `true` si la vela es financieramente válida.
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
            high: 49000.0, // high < open → inválido
            low: 49000.0,
            close: 49500.0,
            volume: 10.5,
            close_time_ms: 3000,
        };
        assert!(!candle.is_valid());
        assert!(candle.to_domain_candle().is_err());
    }
}
