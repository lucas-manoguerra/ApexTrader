use crate::errors::PriceError;









#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price {
    value: rust_decimal::Decimal,
}

impl Price {






    pub fn new(value: rust_decimal::Decimal) -> Result<Self, PriceError> {
        if value.is_sign_negative() {
            return Err(PriceError::NegativePrice {
                value: value.to_string(),
            });
        }

        Ok(Self { value })
    }





    pub fn new_from_raw(value: i64, scale: u32) -> Result<Self, PriceError> {
        let decimal = rust_decimal::Decimal::new(value, scale);
        Self::new(decimal)
    }


    pub fn value(self) -> rust_decimal::Decimal {
        self.value
    }


    pub fn is_zero(self) -> bool {
        self.value.is_zero()
    }


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
