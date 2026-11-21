use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::quantity::Quantity;
use apex_trader::errors::PriceError;
use rust_decimal::Decimal;

/// Precio es un tipo puro de dominio: acepta valores finitos y no negativos.
#[test]
fn price_accepts_valid_positive_value() {
    let p = Price::new(Decimal::new(12345, 2)).expect("123.45 es un precio válido");
    assert_eq!(p.value(), Decimal::new(12345, 2));
    assert!(p.is_positive());
    assert!(!p.is_zero());
}

#[test]
fn price_accepts_zero() {
    let p = Price::new(Decimal::ZERO).expect("0 es un precio válido");
    assert!(p.is_zero());
    assert!(!p.is_positive());
}

#[test]
fn price_rejects_negative_value() {
    let result = Price::new(Decimal::new(-100, 2));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PriceError::NegativePrice {
            value: "-1.00".to_string()
        }
    );
}

#[test]
fn price_from_str_parses_correctly() {
    let p: Price = "52345.67".parse().expect("52345.67 es parseable");
    assert_eq!(p.value(), Decimal::new(5234567, 2));
}

#[test]
fn price_from_str_rejects_invalid_input() {
    let result: Result<Price, _> = "abc".parse();
    assert!(result.is_err());
}

/// Quantity es un tipo puro de dominio: solo acepta valores finitos y > 0.
#[test]
fn quantity_accepts_positive_value() {
    let q = Quantity::new(Decimal::new(12345, 2)).expect("123.45 es una cantidad válida");
    assert_eq!(q.value(), Decimal::new(12345, 2));
    assert!(q.is_positive());
}

#[test]
fn quantity_rejects_zero() {
    let result = Quantity::new(Decimal::ZERO);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        PriceError::NonPositiveQuantity { .. }
    ));
}

#[test]
fn quantity_rejects_negative_value() {
    let result = Quantity::new(Decimal::new(-100, 2));
    assert!(result.is_err());
}

#[test]
fn quantity_from_str_parses_correctly() {
    let q: Quantity = "0.00123".parse().expect("0.00123 es parseable");
    assert_eq!(q.value(), Decimal::new(123, 5));
}

/// PriceRange garantiza el invariante min <= max.
#[test]
fn price_range_accepts_valid_range() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let r = PriceRange::new(min, max).expect("rango válido");
    assert_eq!(r.min(), min);
    assert_eq!(r.max(), max);
}

#[test]
fn price_range_rejects_min_greater_than_max() {
    let min = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let max = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let result = PriceRange::new(min, max);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        PriceError::InvalidPriceRange { .. }
    ));
}

#[test]
fn price_range_contains_works() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let r = PriceRange::new(min, max).expect("rango válido");

    assert!(r.contains(Price::new(Decimal::new(15000, 2)).expect("150.00 válido")));
    assert!(r.contains(Price::new(Decimal::new(10000, 2)).expect("100.00 válido")));
    assert!(r.contains(Price::new(Decimal::new(20000, 2)).expect("200.00 válido")));
    assert!(!r.contains(Price::new(Decimal::new(5000, 2)).expect("50.00 válido")));
    assert!(!r.contains(Price::new(Decimal::new(25000, 2)).expect("250.00 válido")));
}

#[test]
fn price_range_midpoint_is_correct() {
    let min = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
    let max = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
    let r = PriceRange::new(min, max).expect("rango válido");
    let mid = r.midpoint();
    assert_eq!(mid.value(), Decimal::new(15000, 2));
}
