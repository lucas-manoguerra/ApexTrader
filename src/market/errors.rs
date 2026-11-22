//! Errores de la capa de datos de mercado.
//!
//! Tipos propios de error para fallos de red, autenticación, rate limit
//! y símbolos inválidos, sin panic ante fallos externos.

use thiserror::Error;

/// Error al operar con datos de mercado.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MarketError {
    /// Símbolo no válido o con formato incorrecto.
    #[error("símbolo inválido '{symbol}': {reason}")]
    InvalidSymbol { symbol: String, reason: String },

    /// Error de red al consultar el proveedor de datos.
    #[error("error de red: {0}")]
    Network(String),

    /// Error de autenticación con el proveedor.
    #[error("autenticación fallida: {0}")]
    Auth(String),

    /// Límite de solicitudes excedido (rate limit).
    #[error("rate limit excedido: {0}")]
    RateLimit(String),

    /// Símbolo no soportado por el proveedor.
    #[error("símbolo no soportado: {0}")]
    UnsupportedSymbol(String),

    /// No hay datos disponibles para el símbolo y rango solicitados.
    #[error("sin datos para {symbol} en el rango solicitado")]
    NoData { symbol: String },

    /// Respuesta del proveedor no se pudo interpretar.
    #[error("respuesta inválida del proveedor: {0}")]
    InvalidResponse(String),
}
