//! Capa de datos de mercado.
//!
//! Provee velas (OHLCV) al chart engine, separando la obtención de datos
//! del renderizado y del estado del gráfico.
//!
//! Flujo: `MarketDataProvider` → `ChartState` → `Chart Engine` → `Renderer`.

pub mod candle;
pub mod errors;
pub mod mock;
pub mod mock_realtime;
pub mod provider;
pub mod symbol;
