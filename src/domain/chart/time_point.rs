#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimePoint {
    open_time_ms: i64,
    close_time_ms: i64,
    interval_ms: i64,
}

impl TimePoint {
    pub fn new(open_time_ms: i64, close_time_ms: i64, interval_ms: i64) -> Option<Self> {
        if close_time_ms < open_time_ms || interval_ms <= 0 {
            return None;
        }
        Some(Self {
            open_time_ms,
            close_time_ms,
            interval_ms,
        })
    }

    pub const fn open_time_ms(self) -> i64 {\n        self.open_time_ms\n    }\n\n    pub const fn close_time_ms(self) -> i64 {\n        self.close_time_ms\n    }\n\n    pub const fn interval_ms(self) -> i64 {\n        self.interval_ms\n    }\n\n    pub fn midpoint_ms(self) -> f64 {
        (self.open_time_ms as f64 + self.close_time_ms as f64) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_point_creates_valid() {
        let tp = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        assert_eq!(tp.open_time_ms(), 0);
        assert_eq!(tp.close_time_ms(), 60_000);
        assert_eq!(tp.interval_ms(), 60_000);
    }

    #[test]
    fn time_point_rejects_invalid_interval() {
        assert!(TimePoint::new(0, 60_000, 0).is_none());
        assert!(TimePoint::new(0, 60_000, -1).is_none());
    }

    #[test]
    fn time_point_rejects_inverted_range() {
        assert!(TimePoint::new(60_000, 0, 60_000).is_none());
    }

    #[test]
    fn time_point_midpoint() {
        let tp = TimePoint::new(0, 100_000, 60_000).expect("válido");
        assert!((tp.midpoint_ms() - 50_000.0).abs() < f64::EPSILON);
    }
}
