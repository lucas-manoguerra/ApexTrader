//! Comando para cargar velas de mercado desde un proveedor.
//!
//! Coordina la obtención de datos (`MarketDataProvider`) y su conversión
//! a velas de dominio (`domain::Candle`), listas para ser ingestadas por
//! `ChartState`.

use crate::domain::candle::Candle;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

/// Resultado de cargar velas: lista de velas de dominio listas para
/// `ChartState::add_candles`.
#[derive(Debug)]
pub struct LoadCandlesResult {
    /// Velas convertidas a dominio, ordenadas por tiempo.
    pub candles: Vec<Candle>,
    /// Símbolo consultado.
    pub symbol: Symbol,
    /// Timestamp de inicio del rango consultado.
    pub from_ms: i64,
    /// Timestamp de fin del rango consultado.
    pub to_ms: i64,
}

/// Comando: cargar velas de un símbolo desde un proveedor de mercado.
pub struct LoadCandles;

impl LoadCandles {
    /// Ejecuta la carga de velas.
    ///
    /// Obtiene velas del proveedor, las convierte a `domain::Candle` y
    /// las devuelve ordenadas por tiempo de apertura. Errores de red,
    /// auth o rate limit se propagan como `MarketError` sin hacer panic.
    pub fn execute(
        provider: &dyn MarketDataProvider,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<LoadCandlesResult, MarketError> {
        let market_candles = provider.fetch_candles(symbol.clone(), from_ms, to_ms)?;

        // Convertir a domain::Candle, descartando las inválidas.
        let mut candles = Vec::with_capacity(market_candles.len());
        for mc in &market_candles {
            if let Ok(candle) = mc.to_domain_candle() {
                candles.push(candle);
            }
        }

        // Las velas del proveedor ya vienen ordenadas por tiempo (contrato
        // de MarketDataProvider). El dominio no tiene timestamp, así que el
        // orden de inserción es el orden cronológico.
        //
        // Nota: no usamos sort_by aquí porque domain::Candle no expone
        // timestamp; el orden se preserva del proveedor.

        Ok(LoadCandlesResult {
            candles,
            symbol,
            from_ms,
            to_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::mock::MockMarketDataProvider;
    use std::str::FromStr;

    #[test]
    fn load_candles_from_mock_provider() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadCandles::execute(&provider, symbol, 0, 120_000).expect("velas cargadas");

        assert!(!result.candles.is_empty());
        assert_eq!(result.symbol.as_string(), "BTC/USDT");
        assert_eq!(result.from_ms, 0);
        assert_eq!(result.to_ms, 120_000);

        // Todas las velas deben ser válidas (ya se filtraron las inválidas).
        for c in &result.candles {
            assert!(c.open().is_finite());
            assert!(c.high().is_finite());
            assert!(c.low().is_finite());
            assert!(c.close().is_finite());
        }
    }

    #[test]
    fn load_candles_empty_range() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        // Rango sin velas (from > to genera error en el mock)
        let err = LoadCandles::execute(&provider, symbol, 100_000, 50_000)
            .expect_err("rango invertido debe fallar");
        assert!(matches!(err, MarketError::InvalidResponse { .. }));
    }
}
