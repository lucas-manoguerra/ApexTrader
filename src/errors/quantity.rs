//! Errores del tipo de dominio `Quantity`.
//!
//! `Quantity` tiene sus propios errores en lugar de reusar `PriceError`,
//! ya que semánticamente son conceptos distintos (precio vs. cantidad).

/// Errores al construir o validar una `Quantity`.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QuantityError {
    #[error("La cantidad debe ser un valor finito (no NaN ni infinito)")]
    NonFiniteQuantity,

    #[error("La cantidad debe ser mayor que cero: {value}")]
    NonPositiveQuantity { value: String },
}
