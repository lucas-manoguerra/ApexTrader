//! Deterministic planning for historical candle-gap recovery.

/// Inclusive range of candle open timestamps that are missing between two
/// observed candles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoricalGap {
    pub from_ms: i64,
    pub to_ms: i64,
}

/// Finds the missing candle-open range between two observations.
///
/// Returns no range when observations are contiguous, overlap, are out of
/// order, or cannot form a valid inclusive range. The interval must be positive.
pub fn detect_gap(
    previous_open_ms: i64,
    next_open_ms: i64,
    interval_ms: i64,
) -> Option<HistoricalGap> {
    if interval_ms <= 0 {
        return None;
    }

    let first_missing = previous_open_ms.checked_add(interval_ms)?;
    let last_missing = next_open_ms.checked_sub(interval_ms)?;

    (first_missing <= last_missing).then_some(HistoricalGap {
        from_ms: first_missing,
        to_ms: last_missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: i64 = 60_000;

    #[test]
    fn contiguous_observations_have_no_gap() {
        assert_eq!(detect_gap(0, MINUTE, MINUTE), None);
    }

    #[test]
    fn detects_one_missing_candle() {
        assert_eq!(
            detect_gap(0, 2 * MINUTE, MINUTE),
            Some(HistoricalGap {
                from_ms: MINUTE,
                to_ms: MINUTE,
            })
        );
    }

    #[test]
    fn detects_multiple_missing_candles_as_one_range() {
        assert_eq!(
            detect_gap(0, 4 * MINUTE, MINUTE),
            Some(HistoricalGap {
                from_ms: MINUTE,
                to_ms: 3 * MINUTE,
            })
        );
    }

    #[test]
    fn ignores_overlapping_or_out_of_order_observations() {
        assert_eq!(detect_gap(2 * MINUTE, MINUTE, MINUTE), None);
        assert_eq!(detect_gap(2 * MINUTE, 2 * MINUTE, MINUTE), None);
        assert_eq!(detect_gap(MINUTE, MINUTE, MINUTE), None);
    }

    #[test]
    fn rejects_non_positive_intervals() {
        assert_eq!(detect_gap(0, 10, 0), None);
        assert_eq!(detect_gap(0, 10, -1), None);
    }

    #[test]
    fn handles_timestamp_overflow_without_panicking() {
        assert_eq!(detect_gap(i64::MAX, i64::MAX, MINUTE), None);
        assert_eq!(detect_gap(i64::MIN, i64::MIN, MINUTE), None);
    }
}
