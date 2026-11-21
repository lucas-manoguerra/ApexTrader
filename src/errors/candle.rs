/// Errores del dominio de velas (Candle).
///
/// Definidos con `thiserror` para tener tipos propios de error en lugar de
/// devolver `String`.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CandleError {
    #[error("La vela tiene precios no finitos: {0:?}")]
    NonFinitePrice(CandleDump),

    #[error(
        "El precio mínimo (low) no puede ser mayor que el máximo (high): low={low}, high={high}"
    )]
    InvalidLowHigh { low: f64, high: f64 },

    #[error(
        "El precio de apertura (open) está fuera del rango [low, high]: open={open}, low={low}, high={high}"
    )]
    InvalidOpen { open: f64, low: f64, high: f64 },

    #[error(
        "El precio de cierre (close) está fuera del rango [low, high]: close={close}, low={low}, high={high}"
    )]
    InvalidClose { close: f64, low: f64, high: f64 },
}

/// Volcado de una vela para mensajes de error (sin campos privados).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandleDump {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
