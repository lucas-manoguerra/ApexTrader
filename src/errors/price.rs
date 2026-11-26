




#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PriceError {
    #[error("El precio debe ser un valor finito (no NaN ni infinito)")]
    NonFinitePrice,

    #[error("El precio no puede ser negativo: {value}")]
    NegativePrice { value: String },

    #[error("El precio mínimo no puede ser mayor que el máximo: min={min}, max={max}")]
    InvalidPriceRange { min: String, max: String },
}
