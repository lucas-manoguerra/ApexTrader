/// Errores de tipos numéricos de mercado (precio, cantidad, rango de precios).
///
/// Definidos con `thiserror` para tener tipos propios de error en lugar de
/// devolver `String`. Esto permite que el llamante haga `match` sobre el tipo
/// de error y reaccione de forma específica.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PriceError {
    #[error("El precio debe ser un valor finito (no NaN ni infinito)")]
    NonFinitePrice,

    #[error("El precio no puede ser negativo: {value}")]
    NegativePrice { value: String },

    #[error("La cantidad debe ser un valor finito (no NaN ni infinito)")]
    NonFiniteQuantity,

    #[error("La cantidad debe ser mayor que cero: {value}")]
    NonPositiveQuantity { value: String },

    #[error("El precio mínimo no puede ser mayor que el máximo: min={min}, max={max}")]
    InvalidPriceRange { min: String, max: String },

    #[error("El precio no puede ser cero")]
    ZeroPrice,
}
