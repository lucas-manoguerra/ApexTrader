//! Datos de chart: vela OHLCV asociada a su punto temporal y índice lógico.
//!
//! `ChartDataPoint` combina el dato financiero (`Candle`) con su posición
//! temporal (`TimePoint`) y su índice lógico en el chart. Es la unidad
//! fundamental que el viewport y el renderer consumen.

use crate::domain::candle::Candle;

/// Punto de datos del chart: vela OHLCV + punto temporal + índice lógico.
///
/// `logical_index` es `None` hasta que el `TimeScale` le asigna un índice.
/// Esto permite representar huecos temporales y velas fuera del rango de
/// datos sin fabricar velas de mercado.
#[derive(Debug, Clone, Copy)]
pub struct ChartDataPoint {
    /// Vela OHLCV.
    pub candle: Candle,
    /// Punto temporal asociado (timestamp y intervalo).
    pub time_point: crate::domain::chart::time_point::TimePoint,
    /// Índice lógico en el chart (None hasta asignación del viewport).
    pub logical_index: Option<usize>,
}

impl ChartDataPoint {
    /// Crea un `ChartDataPoint` a partir de una vela y un punto temporal.
    pub fn new(candle: Candle, time_point: crate::domain::chart::time_point::TimePoint) -> Self {
        Self {
            candle,
            time_point,
            logical_index: None,
        }
    }

    /// Devuelve el precio de apertura.
    pub fn open(&self) -> f64 {
        self.candle.open()
    }

    /// Devuelve el precio máximo.
    pub fn high(&self) -> f64 {
        self.candle.high()
    }

    /// Devuelve el precio mínimo.
    pub fn low(&self) -> f64 {
        self.candle.low()
    }

    /// Devuelve el precio de cierre.
    pub fn close(&self) -> f64 {
        self.candle.close()
    }

    /// Devuelve el volumen.
    pub fn volume(&self) -> f64 {
        self.candle.volume()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::chart::time_point::TimePoint;

    #[test]
    fn chart_data_point_delegates_to_candle() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).expect("vela válida");
        let tp = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let dp = ChartDataPoint::new(candle, tp);

        assert!((dp.open() - 100.0).abs() < f64::EPSILON);
        assert!((dp.high() - 105.0).abs() < f64::EPSILON);
        assert!((dp.low() - 99.0).abs() < f64::EPSILON);
        assert!((dp.close() - 103.0).abs() < f64::EPSILON);
        assert!((dp.volume() - 1000.0).abs() < f64::EPSILON);
        assert!(dp.logical_index.is_none());
    }
}
