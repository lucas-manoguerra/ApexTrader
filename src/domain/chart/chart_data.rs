use crate::domain::candle::Candle;

#[derive(Debug, Clone, Copy)]
pub struct ChartDataPoint {

    pub candle: Candle,

    pub time_point: crate::domain::chart::time_point::TimePoint,

    pub logical_index: Option<usize>,
}

impl ChartDataPoint {

    pub fn new(candle: Candle, time_point: crate::domain::chart::time_point::TimePoint) -> Self {
        Self {
            candle,
            time_point,
            logical_index: None,
        }
    }

    pub fn open(&self) -> f64 {
        self.candle.open()
    }

    pub fn high(&self) -> f64 {
        self.candle.high()
    }

    pub fn low(&self) -> f64 {
        self.candle.low()
    }

    pub fn close(&self) -> f64 {
        self.candle.close()
    }

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
