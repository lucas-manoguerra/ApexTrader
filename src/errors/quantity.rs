





#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum QuantityError {
    #[error("La cantidad debe ser un valor finito (no NaN ni infinito)")]
    NonFiniteQuantity,

    #[error("La cantidad debe ser mayor que cero: {value}")]
    NonPositiveQuantity { value: String },
}
