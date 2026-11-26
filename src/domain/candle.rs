use crate::errors::candle::CandleDump;
use crate::errors::candle::CandleError;

#[derive(Debug, Clone, Copy)]
pub struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {

    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Self, CandleError> {
        if !open.is_finite() || !high.is_finite() || !low.is_finite() || !close.is_finite() {
            return Err(CandleError::NonFinitePrice(CandleDump {
                open,
                high,
                low,
                close,
                volume,
            }));
        }

        if !volume.is_finite() || volume < 0.0 {
            return Err(CandleError::InvalidVolume { volume });
        }

        if low > high {
            return Err(CandleError::InvalidLowHigh { low, high });
        }

        if open < low || open > high {
            return Err(CandleError::InvalidOpen { open, low, high });
        }

        if close < low || close > high {
            return Err(CandleError::InvalidClose { close, low, high });
        }

        Ok(Self {
            open,
            high,
            low,
            close,
            volume,
        })
    }

    pub fn open(&self) -> f64 {
        self.open
    }

    pub fn high(&self) -> f64 {
        self.high
    }

    pub fn low(&self) -> f64 {
        self.low
    }

    pub fn close(&self) -> f64 {
        self.close
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }
}

impl std::fmt::Display for Candle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Candle {{ open={:.2}, high={:.2}, low={:.2}, close={:.2}, volume={:.2} }}",
            self.open, self.high, self.low, self.close, self.volume
        )
    }
}
