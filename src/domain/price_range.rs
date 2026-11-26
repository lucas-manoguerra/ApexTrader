use crate::domain::price::Price;
use crate::errors::PriceError;





#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PriceRange {
    min: Price,
    max: Price,
}

impl PriceRange {





    pub fn new(min: Price, max: Price) -> Result<Self, PriceError> {
        if min.value() > max.value() {
            return Err(PriceError::InvalidPriceRange {
                min: min.value().to_string(),
                max: max.value().to_string(),
            });
        }

        Ok(Self { min, max })
    }


    pub fn min(self) -> Price {
        self.min
    }


    pub fn max(self) -> Price {
        self.max
    }


    pub fn contains(&self, price: Price) -> bool {
        price.value() >= self.min.value() && price.value() <= self.max.value()
    }


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
