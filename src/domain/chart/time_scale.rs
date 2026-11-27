use crate::domain::chart::time_point::TimePoint;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimeScale {
    timestamps_ms: Vec<i64>,
}

impl TimeScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild<I>(&mut self, timestamps: I)
    where
        I: IntoIterator<Item = i64>,
    {
        self.timestamps_ms = timestamps.into_iter().collect();
        self.timestamps_ms.sort_unstable();
        self.timestamps_ms.dedup();
    }

    pub fn push(&mut self, timestamp_ms: i64) -> bool {
        if self
            .timestamps_ms
            .last()
            .is_some_and(|last_timestamp| timestamp_ms <= *last_timestamp)
        {
            return false;
        }
        self.timestamps_ms.push(timestamp_ms);
        true
    }

    pub fn len(&self) -> usize {
        self.timestamps_ms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timestamps_ms.is_empty()
    }

    pub fn timestamps(&self) -> &[i64] {
        &self.timestamps_ms
    }

    pub fn logical_index(&self, timestamp_ms: i64) -> Option<usize> {
        self.timestamps_ms.binary_search(&timestamp_ms).ok()
    }

    pub fn rebuild_from_series<I, S>(&mut self, series: I)
    where
        I: IntoIterator<Item = S>,
        S: IntoIterator<Item = i64>,
    {
        self.rebuild(series.into_iter().flatten());
    }

    pub fn time_range_to_logical_range(
        &self,
        start_time_ms: i64,
        end_time_ms: i64,
    ) -> Option<(f64, f64)> {
        let first = *self.timestamps_ms.first()?;
        let last = *self.timestamps_ms.last()?;
        if start_time_ms > end_time_ms || end_time_ms < first || start_time_ms > last {
            return None;
        }

        let clipped_start = start_time_ms.max(first);
        let clipped_end = end_time_ms.min(last);
        Some((
            self.time_to_logical(clipped_start)?,
            self.time_to_logical(clipped_end)?,
        ))
    }

    pub fn indices_in_time_range(
        &self,
        start_time_ms: i64,
        end_time_ms: i64,
    ) -> Option<std::ops::RangeInclusive<usize>> {
        if self.timestamps_ms.is_empty() || start_time_ms > end_time_ms {
            return None;
        }

        let first = self
            .timestamps_ms
            .partition_point(|time| *time < start_time_ms);
        let after_last = self
            .timestamps_ms
            .partition_point(|time| *time <= end_time_ms);

        if first < after_last {
            Some(first..=after_last - 1)
        } else {
            None
        }
    }

    pub fn logical_range_to_time_range(
        &self,
        first_logical: f64,
        last_logical: f64,
    ) -> Option<(i64, i64)> {
        if !first_logical.is_finite() || !last_logical.is_finite() || first_logical > last_logical {
            return None;
        }

        Some((
            self.logical_to_time(first_logical)?,
            self.logical_to_time(last_logical)?,
        ))
    }

    pub fn time_to_logical(&self, timestamp_ms: i64) -> Option<f64> {
        if self.timestamps_ms.is_empty() {
            return None;
        }

        match self.timestamps_ms.binary_search(&timestamp_ms) {
            Ok(index) => Some(index as f64),
            Err(0) => None,
            Err(index) if index >= self.timestamps_ms.len() => None,
            Err(right) => {
                let left = right - 1;
                let start = self.timestamps_ms[left] as i128;
                let end = self.timestamps_ms[right] as i128;
                let current = timestamp_ms as i128;
                let span = (end - start) as f64;
                if span <= 0.0 {
                    return Some(left as f64);
                }
                let fraction = (current - start) as f64 / span;
                Some(left as f64 + fraction)
            }
        }
    }

    pub fn logical_to_time(&self, logical_index: f64) -> Option<i64> {
        if self.timestamps_ms.is_empty()
            || !logical_index.is_finite()
            || logical_index < 0.0
            || logical_index > (self.timestamps_ms.len() - 1) as f64
        {
            return None;
        }

        let left = logical_index.floor() as usize;
        let right = logical_index.ceil() as usize;
        if left == right {
            return Some(self.timestamps_ms[left]);
        }

        let fraction = logical_index - left as f64;
        let start = self.timestamps_ms[left] as i128;
        let end = self.timestamps_ms[right] as i128;
        let interpolated = start as f64 + (end - start) as f64 * fraction;
        if !interpolated.is_finite()
            || interpolated < i64::MIN as f64
            || interpolated > i64::MAX as f64
        {
            return None;
        }
        Some(interpolated.round() as i64)
    }

    pub fn format_time_label(timestamp_ms: i64) -> String {
        let seconds = timestamp_ms.div_euclid(1_000);
        let days = seconds.div_euclid(86_400);
        let day_seconds = seconds.rem_euclid(86_400);
        let (_year, month, day) = civil_from_days(days);
        let hour = day_seconds / 3_600;
        let minute = (day_seconds % 3_600) / 60;
        format!("{day:02}/{month:02} {hour:02}:{minute:02} UTC")
    }

    pub fn format_time_axis_label(
        timestamp_ms: i64,
        tick_interval_ms: i64,
        previous_timestamp_ms: Option<i64>,
    ) -> String {
        const DAY_MS: i64 = 86_400_000;
        const MONTH_MS: i64 = 28 * DAY_MS;
        const YEAR_MS: i64 = 365 * DAY_MS;

        let interval_ms = tick_interval_ms.max(1);
        let seconds = timestamp_ms.div_euclid(1_000);
        let days = seconds.div_euclid(86_400);
        let day_seconds = seconds.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);

        if interval_ms >= YEAR_MS {
            return format!("{year}");
        }
        if interval_ms >= MONTH_MS {
            return format!("{month:02}/{year}");
        }

        let previous_date = previous_timestamp_ms.map(|previous| {
            let previous_days = previous.div_euclid(DAY_MS);
            civil_from_days(previous_days)
        });
        let crossed_day = previous_timestamp_ms
            .is_some_and(|previous| previous.div_euclid(DAY_MS) != timestamp_ms.div_euclid(DAY_MS));
        let crossed_year = previous_date.is_some_and(|(previous_year, _, _)| previous_year != year);
        if interval_ms >= DAY_MS || crossed_day {
            return if crossed_year {
                format!("{day:02}/{month:02}/{year}")
            } else {
                format!("{day:02}/{month:02}")
            };
        }

        let hour = day_seconds / 3_600;
        let minute = (day_seconds % 3_600) / 60;
        format!("{hour:02}:{minute:02}")
    }

    pub fn format_time_point(time_point: TimePoint) -> String {
        Self::format_time_label(time_point.open_time_ms())
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    };
    if month <= 2 {
        year += 1;
    }
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scale() -> TimeScale {
        let mut scale = TimeScale::new();

        scale.rebuild([0, 60_000, 3_600_000, 3_660_000]);
        scale
    }

    #[test]
    fn time_axis_labels_include_year_when_crossing_calendar_year() {
        const DAY_MS: i64 = 86_400_000;
        let december_31 = 364 * DAY_MS;
        let january_1 = 365 * DAY_MS;

        assert_eq!(
            TimeScale::format_time_axis_label(january_1, DAY_MS, Some(december_31)),
            "01/01/1971"
        );
    }

    #[test]
    fn time_axis_labels_adapt_between_time_date_month_and_year() {
        const DAY_MS: i64 = 86_400_000;

        assert_eq!(TimeScale::format_time_axis_label(0, 60_000, None), "00:00");
        assert_eq!(
            TimeScale::format_time_axis_label(DAY_MS, 60_000, Some(0)),
            "02/01"
        );
        assert_eq!(
            TimeScale::format_time_axis_label(DAY_MS + 60_000, 60_000, Some(DAY_MS)),
            "00:01"
        );
        assert_eq!(TimeScale::format_time_axis_label(0, DAY_MS, None), "01/01");
        assert_eq!(
            TimeScale::format_time_axis_label(0, 30 * DAY_MS, None),
            "01/1970"
        );
        assert_eq!(
            TimeScale::format_time_axis_label(0, 365 * DAY_MS, None),
            "1970"
        );
    }

    #[test]
    fn push_appends_only_strictly_increasing_timestamps() {
        let mut scale = TimeScale::new();
        assert!(scale.push(10));
        assert!(scale.push(20));
        assert!(!scale.push(20));
        assert!(!scale.push(15));
        assert_eq!(scale.timestamps(), &[10, 20]);
    }

    #[test]
    fn assigns_consecutive_logical_indices_and_deduplicates_times() {
        let mut scale = TimeScale::new();
        scale.rebuild([120, 0, 60, 60]);
        assert_eq!(scale.timestamps(), &[0, 60, 120]);
        assert_eq!(scale.logical_index(0), Some(0));
        assert_eq!(scale.logical_index(60), Some(1));
        assert_eq!(scale.logical_index(120), Some(2));
        assert_eq!(scale.len(), 3);
    }

    #[test]
    fn converts_time_to_logical_index_across_irregular_intervals() {
        let scale = scale();
        assert_eq!(scale.time_to_logical(0), Some(0.0));
        assert_eq!(scale.time_to_logical(60_000), Some(1.0));
        assert_eq!(scale.time_to_logical(1_830_000), Some(1.5));
        assert_eq!(scale.time_to_logical(-1), None);
        assert_eq!(scale.time_to_logical(3_660_001), None);
    }

    #[test]
    fn converts_fractional_logical_index_back_to_time() {
        let scale = scale();
        assert_eq!(scale.logical_to_time(0.0), Some(0));
        assert_eq!(scale.logical_to_time(1.5), Some(1_830_000));
        assert_eq!(scale.logical_to_time(3.0), Some(3_660_000));
        assert_eq!(scale.logical_to_time(4.0), None);
        assert_eq!(scale.logical_to_time(f64::NAN), None);
    }

    #[test]
    fn shared_scale_merges_irregular_series_and_assigns_common_indices() {
        let mut scale = TimeScale::new();
        scale.rebuild_from_series([vec![10, 20, 30], vec![15, 30, 45, 60], vec![20, 40]]);

        assert_eq!(scale.timestamps(), &[10, 15, 20, 30, 40, 45, 60]);
        assert_eq!(scale.logical_index(20), Some(2));
        assert_eq!(scale.logical_index(45), Some(5));
        assert_eq!(scale.logical_index(25), None);
    }

    #[test]
    fn temporal_ranges_map_to_interpolated_and_existing_logical_positions() {
        let mut scale = TimeScale::new();
        scale.rebuild([10, 15, 20, 30, 45, 60]);

        let (first, last) = scale.time_range_to_logical_range(16, 46).unwrap();
        assert!((first - 1.2).abs() < 1e-12);
        assert!((last - (4.0 + 1.0 / 15.0)).abs() < 1e-12);
        assert_eq!(scale.indices_in_time_range(16, 46), Some(2..=4));
        assert_eq!(scale.time_range_to_logical_range(-10, 15), Some((0.0, 1.0)));
        assert_eq!(scale.indices_in_time_range(31, 44), None);
        assert_eq!(scale.indices_in_time_range(-10, 5), None);
        assert_eq!(scale.indices_in_time_range(61, 90), None);
        assert_eq!(scale.indices_in_time_range(10, 10), Some(0..=0));
        assert_eq!(scale.time_range_to_logical_range(61, 90), None);
        assert_eq!(scale.time_range_to_logical_range(30, 20), None);
    }

    #[test]
    fn logical_ranges_round_trip_to_irregular_timestamps() {
        let scale = scale();
        assert_eq!(
            scale.logical_range_to_time_range(0.5, 2.5),
            Some((30_000, 3_630_000))
        );
        assert_eq!(scale.logical_range_to_time_range(2.5, 1.5), None);
        assert_eq!(scale.logical_range_to_time_range(f64::NAN, 1.0), None);
    }

    #[test]
    fn formats_epoch_and_negative_timestamps_as_utc() {
        assert_eq!(TimeScale::format_time_label(0), "01/01 00:00 UTC");
        assert_eq!(TimeScale::format_time_label(86_400_000), "02/01 00:00 UTC");
        assert_eq!(TimeScale::format_time_label(-1), "31/12 23:59 UTC");
    }
    proptest::proptest! {
        #[test]
        fn exact_timestamps_round_trip_to_consecutive_logical_indices(
            mut timestamps in proptest::collection::vec(-10_000_000i64..10_000_000i64, 1..100)
        ) {
            timestamps.sort_unstable();
            timestamps.dedup();
            let mut scale = TimeScale::new();
            scale.rebuild(timestamps.clone());

            for (expected_index, timestamp) in timestamps.iter().enumerate() {
                proptest::prop_assert_eq!(
                    scale.time_to_logical(*timestamp),
                    Some(expected_index as f64)
                );
                proptest::prop_assert_eq!(
                    scale.logical_to_time(expected_index as f64),
                    Some(*timestamp)
                );
            }
        }
    }
}
