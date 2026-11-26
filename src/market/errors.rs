use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MarketError {
    #[error("símbolo inválido '{symbol}': {reason}")]
    InvalidSymbol { symbol: String, reason: String },

    #[error("error de red: {0}")]
    Network(String),

    #[error("autenticación fallida: {0}")]
    Auth(String),

    #[error("rate limit excedido: {0}")]
    RateLimit(String),

    #[error("símbolo no soportado: {0}")]
    UnsupportedSymbol(String),

    #[error("sin datos para {symbol} en el rango solicitado")]
    NoData { symbol: String },

    #[error("respuesta inválida del proveedor: {0}")]
    InvalidResponse(String),
}
