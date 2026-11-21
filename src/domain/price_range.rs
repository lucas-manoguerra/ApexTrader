use crate::domain::price::Price;
use crate::errors::PriceError;

/// Rango de precios válido: mínimo ≤ máximo.
///
/// Tipo de dominio que garantiza el invariante `min <= max`. Imposible de
/// construir en un estado inválido desde fuera del módulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceRange {
    min: Price,
    max: Price,
}

impl PriceRange {
    /// Crea un `PriceRange` validando que `min <= max`.
    ///
    /// # Errores
    ///
    /// Devuelve `PriceError::InvalidPriceRange` si `min` es mayor que `max`.
    pub fn new(min: Price, max: Price) -> Result<Self, PriceError> {
        if min.value() > max.value() {
            return Err(PriceError::InvalidPriceRange {
                min: min.value().to_string(),
                max: max.value().to_string(),
            });
        }

        Ok(Self { min, max })
    }

    /// Devuelve el precio mínimo del rango.
    pub fn min(self) -> Price {
        self.min
    }

    /// Devuelve el precio máximo del rango.
    pub fn max(self) -> Price {
        self.max
    }

    /// Devuelve `true` si el precio dado está dentro del rango (inclusive).
    pub fn contains(&self, price: Price) -> bool {
        price.value() >= self.min.value() && price.value() <= self.max.value()
    }

    /// Devuelve el punto medio del rango.
    pub fn midpoint(self) -> Price {
        let mid = (self.min.value() + self.max.value()) / rust_decimal::Decimal::new(2, 0);
        Price::new(mid).expect("el punto medio de un rango válido siempre es finito y no negativo")
    }
}

impl std::fmt::Display for PriceRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..={}", self.min, self.max)
    }
}
