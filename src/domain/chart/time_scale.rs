//! Escala temporal compartida del chart.
//!
//! La escala separa el timestamp de la coordenada lógica. Cada timestamp
//! distinto ocupa una posición lógica consecutiva; por eso los cierres de
//! sesión y los huecos entre velas no generan espacios artificiales.
//! La interpolación entre dos puntos temporales conserva posiciones
//! fraccionarias para crosshair y navegación.
//!
//! Este tipo no posee zoom ni pan. Esa cámara sigue siendo propiedad de
//! `Viewport`; TimeScale únicamente traduce tiempo ↔ índice lógico.

use crate::domain::chart::time_point::TimePoint;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimeScale {
    timestamps_ms: Vec<i64>,
}

impl TimeScale {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconstruye la escala a partir de timestamps, ordenándolos y
    /// eliminando duplicados.
    pub fn rebuild<I>(&mut self, timestamps: I)
    where
        I: IntoIterator<Item = i64>,
    {
        self.timestamps_ms = timestamps.into_iter().collect();
        self.timestamps_ms.sort_unstable();
        self.timestamps_ms.dedup();
    }

    /// Cantidad de posiciones temporales únicas.
    pub fn len(&self) -> usize {
        self.timestamps_ms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.timestamps_ms.is_empty()
    }

    pub fn timestamps(&self) -> &[i64] {
        &self.timestamps_ms
    }

    /// Devuelve el índice lógico entero asociado exactamente al timestamp.
    pub fn logical_index(&self, timestamp_ms: i64) -> Option<usize> {
        self.timestamps_ms.binary_search(&timestamp_ms).ok()
    }

    /// Convierte un timestamp en una coordenada lógica fraccionaria.
    ///
    /// Los intervalos sin datos quedan comprimidos: el tiempo entre dos
    /// timestamps consecutivos se interpola entre sus índices lógicos.
    /// No extrapola fuera del historial disponible.
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

    /// Convierte un índice lógico (también fraccionario) en timestamp.
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

    /// Etiqueta UTC compacta para el eje temporal.
    pub fn format_time_label(timestamp_ms: i64) -> String {
        let seconds = timestamp_ms.div_euclid(1_000);
        let days = seconds.div_euclid(86_400);
        let day_seconds = seconds.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let hour = day_seconds / 3_600;
        let minute = (day_seconds % 3_600) / 60;
        format!("{day:02}/{month:02} {hour:02}:{minute:02} UTC")
    }

    /// Etiqueta de un punto temporal.
    pub fn format_time_point(time_point: TimePoint) -> String {
        Self::format_time_label(time_point.open_time_ms)
    }
}

/// Convierte días desde Unix epoch a fecha gregoriana (algoritmo civil UTC).
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
        // Salto temporal intencional: no crea índices intermedios.
        scale.rebuild([0, 60_000, 3_600_000, 3_660_000]);
        scale
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
