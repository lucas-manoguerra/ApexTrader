use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::chart::time_scale::TimeScale;
use crate::domain::viewport::Viewport;
use crate::errors::ChartError;

/// Estado del gráfico.
///
/// `ChartState` **posee** los puntos de datos (`ownership`) y el `Viewport` (zoom/pan).
/// Es el único responsable de crear, almacenar y modificar la colección de `ChartDataPoint`.
/// Ningún otro módulo puede tomar posesión de estos datos ni del viewport.
///
/// Como `Candle` valida sus propias invariantes al construirse, `ChartState`
/// confía en que cualquier `ChartDataPoint` recibido ya es válido.
pub struct ChartState {
    data_points: Vec<ChartDataPoint>,
    viewport: Viewport,
    /// Mapeo compartido entre timestamps e índices lógicos.
    time_scale: TimeScale,
    /// Versión monótona de los datos para invalidar cachés incluso si la longitud no cambia.
    data_revision: u64,
}

impl ChartState {
    pub fn new() -> Self {
        tracing::debug!("Creando ChartState vacío");
        Self {
            data_points: Vec::new(),
            viewport: Viewport::new(),
            time_scale: TimeScale::new(),
            data_revision: 0,
        }
    }

    /// Agrega un punto de datos al estado del gráfico.
    ///
    /// El punto de datos debe haber sido creado mediante `ChartDataPoint::new()`,
    /// que valida las invariantes del dominio a través de `Candle::new()`.
    pub fn add_data_point(&mut self, data_point: ChartDataPoint) -> Result<(), ChartError> {
        tracing::debug!(
            "Agregando punto de datos: open={:.2}, high={:.2}, low={:.2}, close={:.2}, tiempo={}",
            data_point.open(),
            data_point.high(),
            data_point.low(),
            data_point.close(),
            data_point.time_point.open_time_ms
        );

        self.data_points.push(data_point);
        self.rebuild_time_scale();
        self.data_revision = self.data_revision.wrapping_add(1);
        Ok(())
    }

    /// Agrega varios puntos de datos al estado del gráfico.
    ///
    /// Los puntos de datos deben estar ordenados y sin duplicados (responsabilidad del
    /// llamante, típicamente la capa de aplicación que conecta con un
    /// `MarketDataProvider`). Cada punto se agrega mediante `add_data_point`.
    pub fn add_data_points(&mut self, data_points: &[ChartDataPoint]) -> Result<(), ChartError> {
        if data_points.is_empty() {
            return Ok(());
        }
        self.data_points.extend_from_slice(data_points);
        self.rebuild_time_scale();
        self.data_revision = self.data_revision.wrapping_add(1);
        Ok(())
    }

    /// Elimina todos los puntos de datos del estado del gráfico.
    ///
    /// Útil para recargar datos de un nuevo símbolo sin acumular datos
    /// anteriores.
    pub fn clear(&mut self) {
        if !self.data_points.is_empty() {
            self.data_points.clear();
            self.time_scale = TimeScale::new();
            self.data_revision = self.data_revision.wrapping_add(1);
        }
    }

    /// Devuelve un préstamo inmutable de los puntos de datos (`borrowing`).
    /// El llamante puede leerlos pero no modificarlos ni apropiarse de ellos.
    pub fn data_points(&self) -> &[ChartDataPoint] {
        tracing::trace!(
            "Prestando {} puntos de datos al llamante",
            self.data_points.len()
        );
        &self.data_points
    }

    /// Devuelve la escala temporal compartida del chart.
    pub fn time_scale(&self) -> &TimeScale {
        &self.time_scale
    }

    /// Centra el viewport en un timestamp del historial.
    ///
    /// Acepta timestamps situados entre dos velas y utiliza la interpolación
    /// de `TimeScale`. Devuelve `false` cuando el timestamp está fuera del
    /// historial o no hay datos.
    pub fn center_on_time(&mut self, timestamp_ms: i64) -> bool {
        let Some(logical_index) = self.time_scale.time_to_logical(timestamp_ms) else {
            return false;
        };

        self.viewport.center_on_logical(logical_index)
    }

    /// Ajusta el viewport para mostrar un intervalo temporal inclusivo.
    ///
    /// Los extremos se recortan a los límites del historial disponible.
    /// `screen_width` debe ser el ancho del área de plot, no el ancho total
    /// del widget. Devuelve `false` si el intervalo no intersecta el historial
    /// o las dimensiones no son válidas.
    pub fn fit_time_range(
        &mut self,
        start_time_ms: i64,
        end_time_ms: i64,
        screen_width: f64,
    ) -> bool {
        let Some((first, last)) = self
            .time_scale
            .time_range_to_logical_range(start_time_ms, end_time_ms)
        else {
            return false;
        };

        self.viewport.fit_logical_range(first, last, screen_width)
    }

    /// Devuelve los timestamps de la primera y última vela visibles.
    ///
    /// El rango se calcula con el mismo `Viewport::visible_range` que utiliza
    /// el renderer, evitando una segunda fórmula para la navegación temporal.
    pub fn visible_time_range(&self, screen_width: f64) -> Option<(i64, i64)> {
        if !screen_width.is_finite() || screen_width <= 0.0 {
            return None;
        }

        let visible = self
            .viewport
            .visible_range(self.time_scale.len(), screen_width);
        if visible.is_empty() {
            return None;
        }

        self.time_scale
            .logical_range_to_time_range(visible.first as f64, visible.last as f64)
    }

    /// Reconstruye los índices lógicos después de cambiar los datos.
    fn rebuild_time_scale(&mut self) {
        self.data_points
            .sort_by_key(|point| point.time_point.open_time_ms);
        self.time_scale.rebuild(
            self.data_points
                .iter()
                .map(|point| point.time_point.open_time_ms),
        );
        for point in &mut self.data_points {
            point.logical_index = self.time_scale.logical_index(point.time_point.open_time_ms);
        }
    }

    /// Devuelve un préstamo inmutable del viewport (`borrowing`).
    /// El renderer lee esto para aplicar zoom y pan al dibujar.
    pub fn viewport(&self) -> &Viewport {
        tracing::trace!("Prestando viewport al renderer");
        &self.viewport
    }

    /// Devuelve un préstamo mutable del viewport para manejar input de usuario.
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        tracing::trace!("Prestando viewport mutable para input de usuario");
        &mut self.viewport
    }

    /// Revisión monótona que cambia cada vez que se modifica la colección de datos.
    pub fn data_revision(&self) -> u64 {
        self.data_revision
    }

    pub fn len(&self) -> usize {
        self.data_points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data_points.is_empty()
    }
}

impl Default for ChartState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;
    use crate::domain::chart::time_point::TimePoint;

    #[test]
    fn data_revision_changes_when_points_are_added_and_cleared() {
        let mut state = ChartState::new();
        let initial_revision = state.data_revision();

        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();
        let time = TimePoint::new(0, 60_000, 60_000).unwrap();
        state
            .add_data_point(ChartDataPoint::new(candle, time))
            .unwrap();

        let after_insert = state.data_revision();
        assert_ne!(after_insert, initial_revision);

        state.clear();
        assert_ne!(state.data_revision(), after_insert);
        assert!(state.is_empty());
    }

    #[test]
    fn clearing_an_already_empty_chart_does_not_change_revision() {
        let mut state = ChartState::new();
        let initial_revision = state.data_revision();

        state.clear();

        assert_eq!(state.data_revision(), initial_revision);
    }

    #[test]
    fn add_data_points_stores_all() {
        let mut state = ChartState::new();
        let candle1 = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).unwrap();
        let candle2 = Candle::new(103.0, 108.0, 102.0, 106.0, 1200.0).unwrap();
        let tp1 = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let tp2 = TimePoint::new(60_000, 120_000, 60_000).expect("intervalo válido");
        let points = [
            ChartDataPoint::new(candle1, tp1),
            ChartDataPoint::new(candle2, tp2),
        ];

        assert!(state.add_data_points(&points).is_ok());
        assert_eq!(state.len(), 2);
        assert!(!state.is_empty());
        assert_eq!(state.data_points()[0].open(), 100.0);
        assert_eq!(state.data_points()[1].close(), 106.0);
        assert_eq!(state.data_points()[0].time_point.open_time_ms, 0);
        assert_eq!(state.data_points()[1].time_point.open_time_ms, 60_000);
    }
    #[test]
    fn chart_state_assigns_stable_logical_indices_in_time_order() {
        let mut state = ChartState::new();
        let later = TimePoint::new(120_000, 180_000, 60_000).unwrap();
        let earlier = TimePoint::new(0, 60_000, 60_000).unwrap();
        let middle = TimePoint::new(60_000, 120_000, 60_000).unwrap();
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();

        state
            .add_data_points(&[
                ChartDataPoint::new(candle, later),
                ChartDataPoint::new(candle, earlier),
                ChartDataPoint::new(candle, middle),
            ])
            .unwrap();

        let points = state.data_points();
        assert_eq!(points[0].time_point.open_time_ms, 0);
        assert_eq!(points[1].time_point.open_time_ms, 60_000);
        assert_eq!(points[2].time_point.open_time_ms, 120_000);
        assert_eq!(
            points
                .iter()
                .map(|point| point.logical_index)
                .collect::<Vec<_>>(),
            vec![Some(0), Some(1), Some(2)]
        );
        assert_eq!(state.time_scale().len(), 3);
    }

    fn chart_with_irregular_times() -> ChartState {
        let mut state = ChartState::new();
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();
        let points = [
            ChartDataPoint::new(candle, TimePoint::new(0, 60_000, 60_000).unwrap()),
            ChartDataPoint::new(
                candle,
                TimePoint::new(60_000, 120_000, 60_000).unwrap(),
            ),
            ChartDataPoint::new(
                candle,
                TimePoint::new(3_600_000, 3_660_000, 60_000).unwrap(),
            ),
            ChartDataPoint::new(
                candle,
                TimePoint::new(3_660_000, 3_720_000, 60_000).unwrap(),
            ),
        ];
        state.add_data_points(&points).unwrap();
        state
    }

    #[test]
    fn chart_state_navigates_to_interpolated_historical_time() {
        let mut state = chart_with_irregular_times();

        assert!(state.center_on_time(1_830_000));
        assert!((state.viewport().screen_to_logical(250.0, 500.0) - 1.5).abs() < 1e-10);
        assert!(!state.center_on_time(-1));
        assert!(!state.center_on_time(3_720_001));
    }

    #[test]
    fn chart_state_fits_and_reports_visible_time_range() {
        let mut state = chart_with_irregular_times();

        assert!(state.fit_time_range(0, 3_660_000, 500.0));
        let visible = state.visible_time_range(500.0);
        assert_eq!(visible, Some((0, 3_660_000)));
        assert_eq!(state.visible_time_range(f64::NAN), None);
        assert!(!state.fit_time_range(3_720_001, 4_000_000, 500.0));
        assert!(!state.fit_time_range(3_660_000, 0, 500.0));
    }

    #[test]
    fn clearing_chart_also_clears_time_scale() {
        let mut state = ChartState::new();
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();
        let time = TimePoint::new(0, 60_000, 60_000).unwrap();
        state
            .add_data_point(ChartDataPoint::new(candle, time))
            .unwrap();
        state.clear();
        assert!(state.time_scale().is_empty());
    }
}
