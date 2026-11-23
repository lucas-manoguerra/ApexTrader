//! Proveedor de datos históricos sintéticos.
//!
//! Genera series OHLCV deterministas con tendencias de varias escalas,
//! volatilidad variable, mechas asimétricas y volumen correlacionado con
//! la actividad del precio. No necesita red ni dependencias aleatorias.

use crate::market::candle::MarketCandle;
use crate::market::errors::MarketError;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;

const DEFAULT_SEED: u64 = 0xA9E5_71C3_4D2B_806F;
const MAX_CANDLES_PER_REQUEST: i128 = 1_000_000;
const PHASE_SALT: u64 = 0x9E37_79B9_7F4A_7C15;
const RETURN_SALT: u64 = 0xD1B5_4A32_D192_ED03;
const WICK_HIGH_SALT: u64 = 0x94D0_49BB_1331_11EB;
const WICK_LOW_SALT: u64 = 0xBF58_476D_1CE4_E5B9;
const VOLUME_SALT: u64 = 0xDB4F_0B91_75AE_2165;
const SPIKE_SALT: u64 = 0xBBE0_4E99_8E2A_4C31;

/// Proveedor sintético histórico.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MockMarketDataProvider {
    /// Nivel de precio alrededor del que oscila la serie.
    pub base_price: f64,
    /// Duración de cada vela en milisegundos.
    pub interval_ms: i64,
    /// Volumen típico aproximado por vela, en unidades del instrumento.
    pub base_volume: f64,
    /// Semilla adicional para generar series alternativas.
    pub seed: u64,
}

impl MockMarketDataProvider {
    /// Crea un proveedor con una configuración razonable para BTC/USDT.
    pub fn new(base_price: f64) -> Self {
        Self {
            base_price,
            interval_ms: 60_000,
            base_volume: 35.0,
            seed: DEFAULT_SEED,
        }
    }

    /// Configura el intervalo temporal de las velas.
    pub fn with_interval_ms(mut self, interval_ms: i64) -> Self {
        self.interval_ms = interval_ms;
        self
    }

    /// Configura el volumen típico por vela.
    pub fn with_base_volume(mut self, base_volume: f64) -> Self {
        self.base_volume = base_volume;
        self
    }

    /// Configura una semilla para obtener otra serie determinista.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Devuelve una semilla estable para el símbolo y la configuración actual.
    pub fn seed_for_symbol(&self, symbol: &Symbol) -> u64 {
        stable_symbol_seed(self.seed, symbol)
    }

    fn validate(&self) -> Result<(), MarketError> {
        if !self.base_price.is_finite() || self.base_price <= 0.0 {
            return Err(MarketError::InvalidResponse(
                "base_price debe ser finito y mayor que cero".to_string(),
            ));
        }
        if self.interval_ms <= 0 {
            return Err(MarketError::InvalidResponse(
                "interval_ms debe ser mayor que cero".to_string(),
            ));
        }
        if !self.base_volume.is_finite() || self.base_volume < 0.0 {
            return Err(MarketError::InvalidResponse(
                "base_volume debe ser finito y no negativo".to_string(),
            ));
        }
        Ok(())
    }

    fn price_at(&self, index: i64, symbol_seed: u64) -> f64 {
        let x = index as f64;
        let phase = unit_noise(symbol_seed, 0, PHASE_SALT) * std::f64::consts::TAU;
        let long_cycle = 0.035 * (x * 0.000_8 + phase).sin();
        let medium_cycle = 0.012 * (x * 0.011 + phase * 1.7).sin();
        let short_cycle = 0.003_5 * (x * 0.071 + phase * 0.31).sin();
        let regime_cycle = 0.002_5 * (x * 0.001_9 + phase * 0.8).sin();
        let volatility = 0.000_55
            + 0.000_35 * (0.5 + 0.5 * (x * 0.002_3 + phase).sin());
        let micro_noise = normal_noise(symbol_seed, index, RETURN_SALT) * volatility;
        self.base_price
            * (long_cycle + medium_cycle + short_cycle + regime_cycle + micro_noise).exp()
    }

    fn generate_candle(
        &self,
        symbol: &Symbol,
        index: i64,
        symbol_seed: u64,
    ) -> Result<MarketCandle, MarketError> {
        let open_time_ms = index.checked_mul(self.interval_ms).ok_or_else(|| {
            MarketError::InvalidResponse("timestamp de apertura fuera de rango".to_string())
        })?;
        let close_time_ms = open_time_ms.checked_add(self.interval_ms).ok_or_else(|| {
            MarketError::InvalidResponse("timestamp de cierre fuera de rango".to_string())
        })?;

        let open = self.price_at(index.saturating_sub(1), symbol_seed);
        let close = self.price_at(index, symbol_seed);
        let absolute_return = ((close / open).ln()).abs();
        let typical_range = open.max(close)
            * (0.000_25 + 0.000_9 * (1.0 + (index as f64 * 0.001_7).sin()) + absolute_return * 0.35);
        let high = open.max(close)
            + normal_noise(symbol_seed, index, WICK_HIGH_SALT).abs() * typical_range;
        let low = (open.min(close)
            - normal_noise(symbol_seed, index, WICK_LOW_SALT).abs() * typical_range)
            .max(f64::MIN_POSITIVE);

        // El volumen aumenta con el retorno absoluto y presenta picos
        // ocasionales que simulan eventos de actividad excepcional.
        let activity = 0.45 + unit_noise(symbol_seed, index, VOLUME_SALT) * 1.4;
        let volatility_factor = 1.0 + (absolute_return / 0.000_7).min(4.0) * 0.55;
        let spike = if unit_noise(symbol_seed, index, SPIKE_SALT) > 0.985 {
            4.0 + unit_noise(symbol_seed, index, SPIKE_SALT ^ 0x55AA) * 3.0
        } else {
            1.0
        };
        let volume = self.base_volume * activity * volatility_factor * spike;

        let candle = MarketCandle {
            symbol: symbol.clone(),
            open_time_ms,
            open,
            high,
            low,
            close,
            volume,
            close_time_ms,
        };
        if candle.is_valid() {
            Ok(candle)
        } else {
            Err(MarketError::InvalidResponse(format!(
                "el generador produjo una vela inválida en el índice {index}"
            )))
        }
    }
}

impl Default for MockMarketDataProvider {
    fn default() -> Self {
        Self::new(50_000.0)
    }
}

impl MarketDataProvider for MockMarketDataProvider {
    fn name(&self) -> &str {
        "mock-historical"
    }

    fn fetch_candles(
        &self,
        symbol: Symbol,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<MarketCandle>, MarketError> {
        if from_ms > to_ms {
            return Err(MarketError::InvalidResponse(format!(
                "from_ms ({from_ms}) > to_ms ({to_ms})"
            )));
        }
        self.validate()?;

        // Devuelve velas completas cuyo timestamp de apertura pertenece
        // al rango inclusivo solicitado. También funciona con tiempos negativos.
        let mut first_index = from_ms.div_euclid(self.interval_ms);
        if from_ms.rem_euclid(self.interval_ms) != 0 {
            first_index = first_index.checked_add(1).ok_or_else(|| {
                MarketError::InvalidResponse("rango temporal fuera de rango".to_string())
            })?;
        }
        let last_index = to_ms.div_euclid(self.interval_ms);
        if first_index > last_index {
            return Ok(Vec::new());
        }

        let candle_count = last_index as i128 - first_index as i128 + 1;
        if candle_count > MAX_CANDLES_PER_REQUEST {
            return Err(MarketError::InvalidResponse(format!(
                "la solicitud excede el máximo de {MAX_CANDLES_PER_REQUEST} velas"
            )));
        }

        let symbol_seed = self.seed_for_symbol(&symbol);
        let mut candles = Vec::with_capacity(candle_count as usize);
        for index in first_index..=last_index {
            candles.push(self.generate_candle(&symbol, index, symbol_seed)?);
        }
        Ok(candles)
    }
}

/// Mezcla determinista de 64 bits (SplitMix64).
pub(crate) fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

pub(crate) fn unit_noise(seed: u64, index: i64, salt: u64) -> f64 {
    let key = seed
        ^ (index as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93)
        ^ salt;
    ((mix64(key) >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
}

pub(crate) fn normal_noise(seed: u64, index: i64, salt: u64) -> f64 {
    let first = unit_noise(seed, index, salt).max(f64::MIN_POSITIVE);
    let second = unit_noise(seed, index, salt ^ 0xA076_1D64_78BD_642F);
    (-2.0 * first.ln()).sqrt() * (std::f64::consts::TAU * second).cos()
}

fn stable_symbol_seed(seed: u64, symbol: &Symbol) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325_u64 ^ seed;
    for byte in symbol.as_string().bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    mix64(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn symbol() -> Symbol {
        Symbol::from_str("BTC/USDT").expect("símbolo válido")
    }

    #[test]
    fn historical_series_is_deterministic_and_continuous() {
        let provider = MockMarketDataProvider::new(50_000.0);
        let first = provider
            .fetch_candles(symbol(), 0, 120_000)
            .expect("histórico generado");
        let second = provider
            .fetch_candles(symbol(), 0, 120_000)
            .expect("histórico repetido");

        assert_eq!(first, second);
        assert_eq!(first.len(), 3);
        for candle in &first {
            assert!(candle.is_valid());
            assert_eq!(candle.close_time_ms - candle.open_time_ms, 60_000);
        }
        for pair in first.windows(2) {
            assert_eq!(pair[0].close, pair[1].open);
            assert!(pair[0].open_time_ms < pair[1].open_time_ms);
        }
    }

    #[test]
    fn historical_series_has_variable_ranges_and_volumes() {
        let provider = MockMarketDataProvider::new(50_000.0);
        let candles = provider
            .fetch_candles(symbol(), 0, 600 * 60_000)
            .expect("histórico generado");

        let distinct_closes = candles
            .iter()
            .map(|candle| (candle.close * 100.0).round() as i64)
            .collect::<std::collections::HashSet<_>>();
        let distinct_volumes = candles
            .iter()
            .map(|candle| (candle.volume * 100.0).round() as i64)
            .collect::<std::collections::HashSet<_>>();

        assert!(distinct_closes.len() > 500);
        assert!(distinct_volumes.len() > 500);
        assert!(candles.iter().any(|candle| candle.high - candle.low > 50.0));
        assert!(candles.iter().all(|candle| candle.volume > 0.0));
    }

    #[test]
    fn volume_increases_with_absolute_price_movement() {
        let provider = MockMarketDataProvider::new(50_000.0);
        let candles = provider
            .fetch_candles(symbol(), 0, 2_000 * 60_000)
            .expect("histórico generado");
        let mut activity = candles
            .windows(2)
            .map(|pair| {
                let return_size = ((pair[1].close / pair[1].open).ln()).abs();
                (return_size, pair[1].volume)
            })
            .collect::<Vec<_>>();
        activity.sort_by(|left, right| left.0.total_cmp(&right.0));

        let quartile = activity.len() / 4;
        let low_volume_mean = activity[..quartile].iter().map(|item| item.1).sum::<f64>()
            / quartile as f64;
        let high_volume_mean = activity[activity.len() - quartile..]
            .iter()
            .map(|item| item.1)
            .sum::<f64>()
            / quartile as f64;

        assert!(
            high_volume_mean > low_volume_mean,
            "el volumen debe acompañar a los movimientos fuertes: bajo={low_volume_mean}, alto={high_volume_mean}"
        );
    }

    #[test]
    fn range_is_aligned_and_only_includes_open_times_inside_it() {
        let provider = MockMarketDataProvider::new(50_000.0);
        let candles = provider
            .fetch_candles(symbol(), 1, 120_000)
            .expect("histórico generado");

        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].open_time_ms, 60_000);
        assert_eq!(candles[1].open_time_ms, 120_000);
    }

    #[test]
    fn provider_rejects_invalid_configuration_and_inverted_ranges() {
        let invalid = MockMarketDataProvider::new(0.0);
        assert!(invalid.fetch_candles(symbol(), 0, 60_000).is_err());

        let invalid_interval = MockMarketDataProvider::default().with_interval_ms(0);
        assert!(invalid_interval.fetch_candles(symbol(), 0, 60_000).is_err());

        let provider = MockMarketDataProvider::default();
        assert!(provider.fetch_candles(symbol(), 100_000, 50_000).is_err());
    }

    #[test]
    fn separate_ranges_return_identical_overlapping_candles() {
        let provider = MockMarketDataProvider::default();
        let whole = provider
            .fetch_candles(symbol(), 0, 600_000)
            .expect("histórico completo");
        let partial = provider
            .fetch_candles(symbol(), 120_000, 360_000)
            .expect("histórico parcial");

        assert_eq!(&whole[2..7], partial.as_slice());
    }
}
