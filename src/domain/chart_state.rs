use crate::domain::candle::Candle;
use crate::domain::viewport::Viewport;
use crate::errors::ChartError;

/// Estado del gráfico.
///
/// `ChartState` **posee** las velas (`ownership`) y el `Viewport` (zoom/pan).
/// Es el único responsable de crear, almacenar y modificar la colección de `Candle`.
/// Ningún otro módulo puede tomar posesión de estas velas ni del viewport.
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

    pub fn add_candle(&mut self, candle: Candle) -> Result<(), ChartError> {
        // Validar que los precios sean finitos
        if !candle.open.is_finite()
            || !candle.high.is_finite()
            || !candle.low.is_finite()
            || !candle.close.is_finite()
        {
            tracing::warn!(
                "Rechazando vela con precios no finitos: {:?}",
                candle
            );
            return Err(ChartError::NonFiniteCandle { candle });
        }

        // Validar invariante: low <= open, close <= high, etc.
        if candle.low > candle.high {
            tracing::warn!(
                "Rechazando vela: low {} > high {}",
                candle.low,
                candle.high
            );
            return Err(ChartError::InvalidPriceRange {
                min: candle.low,
                max: candle.high,
            });
        }

        tracing::debug!(
            "Agregando vela: open={:.2}, high={:.2}, low={:.2}, close={:.2}",
            candle.open,
            candle.high,
            candle.low,
            candle.close
        );

        self.candles.push(candle);
        Ok(())
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
