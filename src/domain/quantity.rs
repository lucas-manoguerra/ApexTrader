use crate::errors::QuantityError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity {
    value: rust_decimal::Decimal,
}

impl Quantity {
    pub fn new(value: rust_decimal::Decimal) -> Result<Self, QuantityError> {
        if value.is_zero() || value.is_sign_negative() {
            return Err(QuantityError::NonPositiveQuantity {
                value: value.to_string(),
            });
        }

        Ok(Self { value })
    }

    pub fn new_from_raw(value: i64, scale: u32) -> Result<Self, QuantityError> {
        let decimal = rust_decimal::Decimal::new(value, scale);
        Self::new(decimal)
    }

    pub fn value(self) -> rust_decimal::Decimal {
        self.value
    }

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
    type Err = QuantityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal =
            rust_decimal::Decimal::from_str(s).map_err(|_| QuantityError::NonFiniteQuantity)?;
        Self::new(decimal)
    }
}
