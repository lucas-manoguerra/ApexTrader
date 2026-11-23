//! Comando para cargar velas de mercado desde un proveedor.
//!
//! Coordina la obtención de datos (`MarketDataProvider`) y su conversión
//! a puntos de datos de chart (`domain::chart::ChartDataPoint`), listas para
//! ser ingestadas por `ChartState`.
//!
//! Política de ingestión explícita (AT-010): no se descartan velas inválidas
//! silenciosamente. Cada rechazo se registra con su motivo y se devuelve en
//! el resultado para que la capa de aplicación pueda informar al usuario.

use crate::domain::chart::chart_data::ChartDataPoint;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

/// Motivo por el cual un punto de datos fue rechazado durante la ingestión.
#[derive(Debug, Clone)]
pub struct IngestionRejection {
    /// Índice del punto dentro del lote recibido del proveedor.
    pub index: usize,
    /// Timestamp de apertura (si se pudo leer).
    pub open_time_ms: i64,
    /// Motivo del rechazo.
    pub reason: String,
}

/// Resultado de cargar velas: lista de puntos de datos válidos y los rechazados.
#[derive(Debug)]
pub struct LoadCandlesResult {
    /// Puntos de datos convertidos a dominio, ordenados por tiempo.
    pub data_points: Vec<ChartDataPoint>,
    /// Puntos de datos rechazados durante la ingestión.
    pub rejected: Vec<IngestionRejection>,
    /// Símbolo consultado.
    pub symbol: Symbol,
    /// Timestamp de inicio del rango consultado.
    pub from_ms: i64,
    /// Timestamp de fin del rango consultado.
    pub to_ms: i64,
}

impl LoadCandlesResult {
    /// Devuelve `true` si no hubo rechazos.
    pub fn is_clean(&self) -> bool {
        self.rejected.is_empty()
    }

    /// Devuelve el total de puntos recibidos (aceptados + rechazados).
    pub fn total_received(&self) -> usize {
        self.data_points.len() + self.rejected.len()
    }
}

/// Comando: cargar velas de un símbolo desde un proveedor de mercado.
pub struct LoadCandles;

impl LoadCandles {
    /// Ejecuta la carga de velas.
    ///
    /// Obtiene velas del proveedor, las convierte a `ChartDataPoint` y las
    /// devuelve ordenadas por tiempo de apertura. Errores de red, auth o
    /// rate limit se propagan como `MarketError` sin hacer panic.
    ///
    /// Velas inválidas no se descartan silenciosamente: se rechazan con un
    /// motivo registrado en `rejected`.
    pub fn execute(
        provider: &dyn MarketDataProvider,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<LoadCandlesResult, MarketError> {
        if from_ms > to_ms {
            return Err(MarketError::InvalidResponse(format!(
                "rango invertido: from_ms ({from_ms}) > to_ms ({to_ms})"
            )));
        }

        let market_candles = provider.fetch_candles(symbol.clone(), from_ms, to_ms)?;

        let mut data_points = Vec::with_capacity(market_candles.len());
        let mut rejected = Vec::new();
        let mut last_open_time: Option<i64> = None;

        for (index, mc) in market_candles.iter().enumerate() {
            if mc.symbol != symbol {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "símbolo inconsistente: esperado {}, recibido {}",
                        symbol.as_string(),
                        mc.symbol.as_string()
                    ),
                });
                continue;
            }

            if mc.open_time_ms < from_ms || mc.open_time_ms > to_ms {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "timestamp fuera del rango solicitado {}..={}",
                        from_ms, to_ms
                    ),
                });
                continue;
            }

            if let Some(last) = last_open_time
                && mc.open_time_ms <= last
            {
                rejected.push(IngestionRejection {
                    index,
                    open_time_ms: mc.open_time_ms,
                    reason: format!(
                        "orden cronológico invocado o duplicado: anterior={}, actual={}",
                        last, mc.open_time_ms
                    ),
                });
                continue;
            }

            match mc.to_chart_data_point() {
                Ok(dp) => {
                    last_open_time = Some(mc.open_time_ms);
                    data_points.push(dp);
                }
                Err(e) => {
                    rejected.push(IngestionRejection {
                        index,
                        open_time_ms: mc.open_time_ms,
                        reason: format!("vela inválida: {e}"),
                    });
                }
            }
        }

        Ok(LoadCandlesResult {
            data_points,
            rejected,
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

        assert!(!result.data_points.is_empty());
        assert_eq!(result.symbol.as_string(), "BTC/USDT");
        assert_eq!(result.from_ms, 0);
        assert_eq!(result.to_ms, 120_000);
        assert!(
            result.is_clean(),
            "no debería haber rechazos con datos mock válidos"
        );

        for dp in &result.data_points {
            assert!(dp.open().is_finite());
            assert!(dp.high().is_finite());
            assert!(dp.low().is_finite());
            assert!(dp.close().is_finite());
            assert!(dp.time_point.open_time_ms >= 0);
        }
    }

    #[test]
    fn load_candles_preserves_timestamps() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let result = LoadCandles::execute(&provider, symbol, 0, 120_000).expect("velas cargadas");

        assert_eq!(result.data_points[0].time_point.open_time_ms, 0);
        // Los tiempos deben ser ascendentes.
        for w in result.data_points.windows(2) {
            assert!(
                w[0].time_point.open_time_ms < w[1].time_point.open_time_ms,
                "puntos de datos ordenados por tiempo ascendente"
            );
        }
    }

    #[test]
    fn load_candles_empty_range() {
        let provider = MockMarketDataProvider::new(50000.0);
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let err = LoadCandles::execute(&provider, symbol, 100_000, 50_000)
            .expect_err("rango invertido debe fallar");
        assert!(matches!(err, MarketError::InvalidResponse { .. }));
    }
}
