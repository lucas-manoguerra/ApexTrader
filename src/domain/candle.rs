/// Una vela japonesa con los cuatro precios OHLC.
///
/// `Candle` es un tipo de dominio: representa un dato de mercado.
/// No sabe nada de dibujo, ventanas ni GPU.
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}
