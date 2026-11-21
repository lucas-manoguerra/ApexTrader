use crate::errors::PriceError;

/// Precio de mercado.
///
/// Tipo de dominio que envuelve un `Decimal` de `rust_decimal` para garantizar
/// que los precios nunca sean `NaN`, infinitos ni negativos.
///
/// Usar `Price` en lugar de `f64` elimina los errores clásicos de punto flotante
/// (ej. `0.1 + 0.2 ≠ 0.3`) y hace imposible construir un precio inválido desde
/// fuera del módulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price {
    value: rust_decimal::Decimal,
}

impl Price {
    /// Crea un `Price` validando que el valor sea finito y no negativo.
    ///
    /// # Errores
    ///
    /// Devuelve `PriceError::NonFinitePrice` si el valor es `NaN` o infinito.
    /// Devuelve `PriceError::NegativePrice` si el valor es menor que cero.
    pub fn new(value: rust_decimal::Decimal) -> Result<Self, PriceError> {
        if value.is_sign_negative() {
            return Err(PriceError::NegativePrice {
                value: value.to_string(),
            });
        }

        Ok(Self { value })
    }

    /// Crea un `Price` a partir de un entero y una escala decimal.
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

    /// Devuelve `true` si el precio es cero.
    pub fn is_zero(self) -> bool {
        self.value.is_zero()
    }

    /// Devuelve `true` si el precio es mayor que cero.
    pub fn is_positive(self) -> bool {
        !self.value.is_zero() && self.value.is_sign_positive()
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl std::str::FromStr for Price {
    type Err = PriceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = rust_decimal::Decimal::from_str(s).map_err(|_| PriceError::NonFinitePrice)?;
        Self::new(decimal)
    }
}
