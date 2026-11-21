use crate::errors::PriceError;

/// Cantidad de activo (tamaño de orden, posición, etc.).
///
/// Tipo de dominio que envuelve un `Decimal` de `rust_decimal` para garantizar
/// que las cantidades nunca sean `NaN`, infinitas, cero ni negativas.
///
/// Usar `Quantity` en lugar de `f64` elimina errores de punto flotante en
/// cálculos de tamaño de orden y posiciones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity {
    value: rust_decimal::Decimal,
}

impl Quantity {
    /// Crea una `Quantity` validando que el valor sea finito y estrictamente
    /// mayor que cero.
    ///
    /// # Errores
    ///
    /// Devuelve `PriceError::NonFiniteQuantity` si el valor es `NaN` o infinito.
    /// Devuelve `PriceError::NonPositiveQuantity` si el valor es cero o menor.
    pub fn new(value: rust_decimal::Decimal) -> Result<Self, PriceError> {
        if value.is_zero() || value.is_sign_negative() {
            return Err(PriceError::NonPositiveQuantity {
                value: value.to_string(),
            });
        }

        Ok(Self { value })
    }

    /// Crea una `Quantity` a partir de un entero y una escala decimal.
    ///
    /// `scale` es la cantidad de decimales. Ejemplo: `new_from_raw(12345, 2)`
    /// produce `123.45`.
    pub fn new_from_raw(value: i64, scale: u32) -> Result<Self, PriceError> {
        let decimal = rust_decimal::Decimal::new(value, scale);
        Self::new(decimal)
    }

    /// Devuelve el valor interno como `Decimal`.
    pub fn value(self) -> rust_decimal::Decimal {
        self.value
    }

    /// Devuelve `true` si la cantidad es mayor que cero (siempre `true` tras
    /// una construcción exitosa).
    pub fn is_positive(self) -> bool {
        self.value.is_sign_positive()
    }
}

impl std::fmt::Display for Quantity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::str::FromStr for Quantity {
    type Err = PriceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal =
            rust_decimal::Decimal::from_str(s).map_err(|_| PriceError::NonFiniteQuantity)?;
        Self::new(decimal)
    }
}
