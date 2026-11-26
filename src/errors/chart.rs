use crate::domain::candle::Candle;






#[derive(Debug, thiserror::Error)]
pub enum ChartError {
    #[error("El rango de precios es inválido: mínimo {min} > máximo {max}")]
    InvalidPriceRange { min: f64, max: f64 },

    #[error("La vela tiene precios no finitos: {candle:?}")]
    NonFiniteCandle { candle: Candle },

    #[error("No hay velas disponibles para renderizar")]
    NoCandlesAvailable,

    #[error("Índice de vela fuera de rango: {index} >= {len}")]
    CandleIndexOutOfRange { index: usize, len: usize },
}
