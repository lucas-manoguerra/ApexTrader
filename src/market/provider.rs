//! Proveedor de datos de mercado.
//!
//! `MarketDataProvider` es el trait que abstrae la obtención de velas
//! (OHLCV) de un símbolo. Permite probar con un proveedor mock sin red
//! y conectar proveedores reales (Binance, etc.) sin tocar el engine de
//! chart.

use crate::market::candle::MarketCandle;
use crate::market::errors::MarketError;
use crate::market::symbol::Symbol;

/// Proveedor de datos de mercado.
///
/// Implementaciones posibles: mock (pruebas sin red), Binance REST/WebSocket,
/// Yahoo Finance, etc.
pub trait MarketDataProvider: Send + Sync {
    /// Devuelve el nombre del proveedor (para logs y depuración).
    fn name(&self) -> &str;

    /// Obtiene velas de un símbolo en un rango de tiempo.
    ///
    /// `from_ms` y `to_ms` son timestamps Unix en milisegundos (UTC).
    /// Devuelve las velas ordenadas por `open_time_ms` ascendente, sin duplicados.
    ///
    /// Errores de red, auth o rate limit se reportan como `MarketError` sin
    /// hacer panic.
    fn fetch_candles(
        &self,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<MarketCandle>, MarketError>;
}


/// Snapshot de una actualización recibida desde un stream realtime.
#[derive(Debug, Clone, PartialEq)]
pub struct RealtimeMarketUpdate {
    /// Estado OHLCV más reciente de la vela en formación.
    pub candle: MarketCandle,
    /// Indica que la actualización cierra la vela y la siguiente tendrá otro timestamp.
    pub is_closed: bool,
}

/// Proveedor de actualizaciones continuas de mercado.
///
/// Las implementaciones pueden usar WebSocket, IPC u otra fuente. Cada llamada
/// devuelve el snapshot más reciente de la vela; varios snapshots pueden
/// compartir el mismo timestamp de apertura mientras la vela siga abierta.
pub trait RealtimeMarketDataProvider: Send {
    /// Nombre legible del proveedor.
    fn name(&self) -> &str;

    /// Obtiene la siguiente actualización de la vela activa.
    fn next_update(&mut self) -> Result<RealtimeMarketUpdate, MarketError>;
}
