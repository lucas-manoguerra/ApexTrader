use crate::domain::candle::Candle;
use crate::domain::viewport::Viewport;
use crate::errors::ChartError;

/// Estado del gráfico.
///
/// `ChartState` **posee** las velas (`ownership`) y el `Viewport` (zoom/pan).
/// Es el único responsable de crear, almacenar y modificar la colección de `Candle`.
/// Ningún otro módulo puede tomar posesión de estas velas ni del viewport.
///
/// Como `Candle` valida sus propias invariantes al construirse, `ChartState`
/// confía en que cualquier `Candle` recibido ya es válido.
pub struct ChartState {
    candles: Vec<Candle>,
    viewport: Viewport,
}

impl ChartState {
    pub fn new() -> Self {
        tracing::debug!("Creando ChartState vacío");
        Self {
            candles: Vec::new(),
            viewport: Viewport::new(),
        }
    }

    /// Agrega una vela al estado del gráfico.
    ///
    /// La vela debe haber sido creada mediante `Candle::new()`, que ya valida
    /// todas las invariantes del dominio (precios finitos, `low <= high`,
    /// `low <= open <= high`, `low <= close <= high`). Por lo tanto, este
    /// método no necesita repetir esas validaciones.
    pub fn add_candle(&mut self, candle: Candle) -> Result<(), ChartError> {
        tracing::debug!(
            "Agregando vela: open={:.2}, high={:.2}, low={:.2}, close={:.2}",
            candle.open(),
            candle.high(),
            candle.low(),
            candle.close()
        );

        self.candles.push(candle);
        Ok(())
    }

    /// Agrega varias velas al estado del gráfico.
    ///
    /// Las velas deben estar ordenadas y sin duplicados (responsabilidad del
    /// llamante, típicamente la capa de aplicación que conecta con un
    /// `MarketDataProvider`). Cada vela se agrega mediante `add_candle`.
    pub fn add_candles(&mut self, candles: &[Candle]) -> Result<(), ChartError> {
        for candle in candles {
            self.add_candle(*candle)?;
        }
        Ok(())
    }

    /// Elimina todas las velas del estado del gráfico.
    ///
    /// Útil para recargar datos de un nuevo símbolo sin acumular velas
    /// anteriores.
    pub fn clear(&mut self) {
        self.candles.clear();
    }

    /// Devuelve un préstamo inmutable de las velas (`borrowing`).
    /// El llamante puede leerlas pero no modificarlas ni apropiarse de ellas.
    pub fn candles(&self) -> &[Candle] {
        tracing::trace!("Prestando {} velas al llamante", self.candles.len());
        &self.candles
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
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
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

    #[test]
    fn add_candles_stores_all() {
        let mut state = ChartState::new();
        let candles = [
            Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).unwrap(),
            Candle::new(103.0, 108.0, 102.0, 106.0, 1200.0).unwrap(),
        ];

        assert!(state.add_candles(&candles).is_ok());
        assert_eq!(state.len(), 2);
        assert!(!state.is_empty());
        assert_eq!(state.candles()[0].open(), 100.0);
        assert_eq!(state.candles()[1].close(), 106.0);
    }
}
