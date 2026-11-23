use crate::errors::candle::CandleDump;
use crate::errors::candle::CandleError;

/// Una vela japonesa con los cuatro precios OHLC y volumen.
///
/// `Candle` es un tipo de dominio: representa un dato de mercado.
/// No sabe nada de dibujo, ventanas ni GPU.
///
/// Todos los campos son privados. Se construye mediante `Candle::new()`,
/// que valida las invariantes del dominio:
///
/// - Todos los precios deben ser finitos (no `NaN`, no infinitos).
/// - `low <= high`.
/// - `low <= open <= high`.
/// - `low <= close <= high`.
/// - `volume` debe ser finito y `>= 0.0`.
///
/// Así es imposible crear una vela financieramente inválida desde fuera
/// del módulo.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl Candle {
    /// Crea una `Candle` validando todas las invariantes del dominio.
    ///
    /// # Errores
    ///
    /// Devuelve `CandleError::NonFinitePrice` si algún precio no es finito.
    /// Devuelve `CandleError::InvalidLowHigh` si `low > high`.
    /// Devuelve `CandleError::InvalidOpen` si `open` está fuera de `[low, high]`.
    /// Devuelve `CandleError::InvalidClose` si `close` está fuera de `[low, high]`.
    /// Devuelve `CandleError::InvalidVolume` si el volumen es negativo o no finito.
    pub fn new(
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
    ) -> Result<Self, CandleError> {
        // Validar que todos los precios sean finitos
        if !open.is_finite() || !high.is_finite() || !low.is_finite() || !close.is_finite() {
            return Err(CandleError::NonFinitePrice(CandleDump {
                open,
                high,
                low,
                close,
                volume,
            }));
        }

        // Validar que el volumen sea finito y no negativo
        if !volume.is_finite() || volume < 0.0 {
            return Err(CandleError::InvalidVolume { volume });
        }

        // Validar que low <= high
        if low > high {
            return Err(CandleError::InvalidLowHigh { low, high });
        }

        // Validar que low <= open <= high
        if open < low || open > high {
            return Err(CandleError::InvalidOpen { open, low, high });
        }

        // Validar que low <= close <= high
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

    /// Devuelve el precio de apertura.
    pub fn open(&self) -> f64 {
        self.open
    }

    /// Devuelve el precio máximo.
    pub fn high(&self) -> f64 {
        self.high
    }

    /// Devuelve el precio mínimo.
    pub fn low(&self) -> f64 {
        self.low
    }

    /// Devuelve el precio de cierre.
    pub fn close(&self) -> f64 {
        self.close
    }

    /// Devuelve el volumen negociado en el período.
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
