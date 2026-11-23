use crate::domain::chart::chart_data::ChartDataPoint;
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
}

impl ChartState {
    pub fn new() -> Self {
        tracing::debug!("Creando ChartState vacío");
        Self {
            data_points: Vec::new(),
            viewport: Viewport::new(),
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
        Ok(())
    }

    /// Agrega varios puntos de datos al estado del gráfico.
    ///
    /// Los puntos de datos deben estar ordenados y sin duplicados (responsabilidad del
    /// llamante, típicamente la capa de aplicación que conecta con un
    /// `MarketDataProvider`). Cada punto se agrega mediante `add_data_point`.
    pub fn add_data_points(&mut self, data_points: &[ChartDataPoint]) -> Result<(), ChartError> {
        for data_point in data_points {
            self.add_data_point(*data_point)?;
        }
        Ok(())
    }

    /// Elimina todos los puntos de datos del estado del gráfico.
    ///
    /// Útil para recargar datos de un nuevo símbolo sin acumular datos
    /// anteriores.
    pub fn clear(&mut self) {
        self.data_points.clear();
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
}
