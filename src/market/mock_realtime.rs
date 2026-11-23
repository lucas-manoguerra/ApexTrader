//! Proveedor mock de actualizaciones realtime.
//!
//! Simula ticks deterministas sobre una vela en formación. Cada actualización
//! conserva el timestamp de apertura de la vela hasta cerrarla; la siguiente
//! actualización comienza una vela nueva contigua a la anterior.

use crate::market::candle::MarketCandle;
use crate::market::errors::MarketError;
use crate::market::mock::{normal_noise, unit_noise};
use crate::market::provider::{RealtimeMarketDataProvider, RealtimeMarketUpdate};
use crate::market::symbol::Symbol;

/// Proveedor de realtime basado en ticks sintéticos.
#[derive(Debug, Clone)]
pub struct MockRealtimeMarketDataProvider {
    symbol: Symbol,
    interval_ms: i64,
    ticks_per_candle: u32,
    base_volume: f64,
    seed: u64,
    next_open_time_ms: i64,
    previous_close: f64,
    current_candle: Option<MarketCandle>,
    current_tick: u32,
    tick_sequence: i64,
}

impl MockRealtimeMarketDataProvider {
    /// Inicializa el stream a partir de la última vela histórica cerrada.
    ///
    /// La siguiente vela comienza exactamente en el timestamp de cierre de
    /// la última vela y abre al precio de cierre del histórico.
    pub fn from_last_candle(
        last_candle: MarketCandle,
        ticks_per_candle: u32,
        base_volume: f64,
        seed: u64,
    ) -> Result<Self, MarketError> {
        if !last_candle.is_valid() {
            return Err(MarketError::InvalidResponse(
                "la última vela histórica no es válida".to_string(),
            ));
        }
        let interval_ms = last_candle
            .close_time_ms
            .checked_sub(last_candle.open_time_ms)
            .filter(|interval| *interval > 0);
        let Some(interval_ms) = interval_ms else {
            return Err(MarketError::InvalidResponse(
                "la última vela no tiene un intervalo válido".to_string(),
            ));
        };
        if ticks_per_candle == 0 {
            return Err(MarketError::InvalidResponse(
                "ticks_per_candle debe ser mayor que cero".to_string(),
            ));
        }
        if !base_volume.is_finite() || base_volume < 0.0 {
            return Err(MarketError::InvalidResponse(
                "base_volume debe ser finito y no negativo".to_string(),
            ));
        }

        Ok(Self {
            symbol: last_candle.symbol,
            interval_ms,
            ticks_per_candle,
            base_volume,
            seed,
            next_open_time_ms: last_candle.close_time_ms,
            previous_close: last_candle.close,
            current_candle: None,
            current_tick: 0,
            tick_sequence: 0,
        })
    }
}

impl RealtimeMarketDataProvider for MockRealtimeMarketDataProvider {
    fn name(&self) -> &str {
        "mock-realtime"
    }

    fn next_update(&mut self) -> Result<RealtimeMarketUpdate, MarketError> {
        if self.current_candle.is_none() || self.current_tick >= self.ticks_per_candle {
            if let Some(previous) = self.current_candle.take() {
                self.previous_close = previous.close;
                self.next_open_time_ms = previous.close_time_ms;
            }
            let close_time_ms = self
                .next_open_time_ms
                .checked_add(self.interval_ms)
                .ok_or_else(|| {
                    MarketError::InvalidResponse("timestamp realtime fuera de rango".to_string())
                })?;
            self.current_candle = Some(MarketCandle {
                symbol: self.symbol.clone(),
                open_time_ms: self.next_open_time_ms,
                open: self.previous_close,
                high: self.previous_close,
                low: self.previous_close,
                close: self.previous_close,
                volume: 0.0,
                close_time_ms,
            });
            self.current_tick = 0;
        }

        self.tick_sequence = self.tick_sequence.wrapping_add(1);
        self.current_tick += 1;

        let tick_index = self.tick_sequence as i64;
        let shock = normal_noise(self.seed, tick_index, 0xA24B_AED4_963E_E407);
        let drift = 0.000_015 + (tick_index as f64 * 0.17).sin() * 0.000_025;
        let tick_return = drift + shock * 0.000_32;
        let previous_price = self
            .current_candle
            .as_ref()
            .map(|candle| candle.close)
            .ok_or_else(|| {
                MarketError::InvalidResponse("no se pudo crear la vela activa realtime".to_string())
            })?;
        let close = previous_price * tick_return.exp();
        let wick_range = previous_price
            * (0.000_025 + unit_noise(self.seed, tick_index, 0x9FB2_1C65_1E98_DF25) * 0.000_11);
        let upper_wick =
            normal_noise(self.seed, tick_index, 0xC13F_A9A9_02A6_328F).abs() * wick_range;
        let lower_wick =
            normal_noise(self.seed, tick_index, 0x91E1_0DA5_C79E_7B1D).abs() * wick_range;

        let volume_noise = unit_noise(self.seed, tick_index, 0xD6E8_FEB8_6659_FD93);
        let movement_factor = 0.55 + (tick_return.abs() / 0.000_32).min(4.0) * 0.45;
        let activity = 0.55 + volume_noise * 0.8;
        let volume_spike = if volume_noise > 0.985 { 4.0 } else { 1.0 };
        let volume_increment = self.base_volume / self.ticks_per_candle as f64
            * movement_factor
            * activity
            * volume_spike;

        let candle = self.current_candle.as_mut().ok_or_else(|| {
            MarketError::InvalidResponse("no existe una vela realtime activa".to_string())
        })?;
        candle.close = close;
        candle.high = candle.high.max(close + upper_wick);
        candle.low = candle.low.min((close - lower_wick).max(f64::MIN_POSITIVE));
        candle.volume += volume_increment;

        if !candle.is_valid() {
            return Err(MarketError::InvalidResponse(
                "el generador realtime produjo una vela inválida".to_string(),
            ));
        }

        Ok(RealtimeMarketUpdate {
            candle: candle.clone(),
            is_closed: self.current_tick == self.ticks_per_candle,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::mock::MockMarketDataProvider;
    use crate::market::provider::{MarketDataProvider, RealtimeMarketDataProvider};
    use std::str::FromStr;

    fn provider(ticks_per_candle: u32) -> MockRealtimeMarketDataProvider {
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
        let history = MockMarketDataProvider::new(50_000.0)
            .fetch_candles(symbol, 0, 0)
            .expect("vela histórica");
        MockRealtimeMarketDataProvider::from_last_candle(
            history.last().expect("última vela").clone(),
            ticks_per_candle,
            35.0,
            42,
        )
        .expect("proveedor realtime")
    }

    #[test]
    fn realtime_ticks_update_one_open_candle_until_it_closes() {
        let mut realtime = provider(4);
        let first = realtime.next_update().expect("primer tick");
        let second = realtime.next_update().expect("segundo tick");
        let third = realtime.next_update().expect("tercer tick");

        assert_eq!(first.candle.open_time_ms, second.candle.open_time_ms);
        assert_eq!(second.candle.open_time_ms, third.candle.open_time_ms);
        assert!(!first.is_closed);
        assert!(!second.is_closed);
        assert!(!third.is_closed);
        assert!(third.candle.volume > first.candle.volume);
        assert!(third.candle.is_valid());
    }

    #[test]
    fn closing_tick_starts_the_next_candle_without_a_time_or_price_gap() {
        let mut realtime = provider(2);
        let first = realtime.next_update().expect("primer tick");
        let closed = realtime.next_update().expect("tick de cierre");
        let next = realtime
            .next_update()
            .expect("primer tick de vela siguiente");

        assert!(!first.is_closed);
        assert!(closed.is_closed);
        assert_eq!(closed.candle.open_time_ms, first.candle.open_time_ms);
        assert_eq!(next.candle.open_time_ms, closed.candle.close_time_ms);
        assert_eq!(next.candle.open, closed.candle.close);
        assert!(next.candle.is_valid());
    }

    #[test]
    fn realtime_updates_convert_to_chart_data_points() {
        let mut realtime = provider(3);
        let update = realtime.next_update().expect("tick realtime");
        let point = update
            .candle
            .to_chart_data_point()
            .expect("punto temporal válido");

        assert_eq!(point.time_point.open_time_ms, update.candle.open_time_ms);
        assert_eq!(point.time_point.interval_ms, 60_000);
        assert_eq!(point.volume(), update.candle.volume);
    }

    #[test]
    fn realtime_provider_rejects_zero_ticks_per_candle() {
        let history = MockMarketDataProvider::new(50_000.0)
            .fetch_candles(Symbol::from_str("BTC/USDT").unwrap(), 0, 0)
            .unwrap();
        assert!(
            MockRealtimeMarketDataProvider::from_last_candle(history[0].clone(), 0, 35.0, 42,)
                .is_err()
        );
    }
}
