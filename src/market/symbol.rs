use crate::market::errors::MarketError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    base: String,
    quote: String,
}

impl Symbol {
    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn quote(&self) -> &str {
        &self.quote
    }

    pub fn as_string(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

impl FromStr for Symbol {
    type Err = MarketError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(MarketError::InvalidSymbol {
                symbol: s.to_string(),
                reason: "formato esperado BASE/QUOTE".to_string(),
            });
        }
        let base = parts[0];
        let quote = parts[1];
        if base.is_empty() || quote.is_empty() {
            return Err(MarketError::InvalidSymbol {
                symbol: s.to_string(),
                reason: "base y quote no pueden estar vacíos".to_string(),
            });
        }
        if !base.chars().all(|c| c.is_ascii_alphanumeric())
            || !quote.chars().all(|c| c.is_ascii_alphanumeric())
        {
            return Err(MarketError::InvalidSymbol {
                symbol: s.to_string(),
                reason: "solo caracteres alfanuméricos ASCII".to_string(),
            });
        }
        Ok(Self {
            base: base.to_string(),
            quote: quote.to_string(),
        })
    }
}

impl TryFrom<&str> for Symbol {
    type Error = MarketError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn symbol_from_valid_pair() {
        let s = Symbol::from_str("BTC/USDT").expect("BTC/USDT válido");
        assert_eq!(s.base(), "BTC");
        assert_eq!(s.quote(), "USDT");
        assert_eq!(s.as_string(), "BTC/USDT");
    }

    #[test]
    fn symbol_rejects_invalid_format() {
        assert!("BTCUSDT".parse::<Symbol>().is_err());
        assert!("BTC/".parse::<Symbol>().is_err());
        assert!("/USDT".parse::<Symbol>().is_err());

        assert!("btc/usdt".parse::<Symbol>().is_ok());
    }
}
