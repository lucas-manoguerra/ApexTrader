/// Viewport del gráfico: estado de zoom y pan.
///
/// `Viewport` **posee** el estado de la cámara del chart.
/// Es responsable de calcular cómo se transforman las coordenadas de datos
/// (precio, tiempo) a coordenadas de pantalla y viceversa.
///
/// Tres espacios de coordenadas están definidos:
///
/// - **DataSpace**: coordenadas del mercado (índice de vela, precio).
///   `offset_x` y `offset_y` se expresan en unidades de datos.
/// - **ViewportSpace**: estado de la cámara (zoom_x, zoom_y, min_zoom, max_zoom).
/// - **ScreenSpace**: píxeles de pantalla (0..=screen_width, 0..=screen_height).
///
/// Las transformaciones entre espacios son explícitas y unidireccionales.
///
/// Los ejes X e Y escalan de forma independiente (S4): el zoom vertical no
/// distorsiona el horizontal y viceversa. Cada eje tiene su propio nivel de
/// zoom, pero ambos comparten los mismos `min_zoom` y `max_zoom` del usuario.
///
/// Todos los campos son privados. Se modifican únicamente a través de
/// métodos públicos que validan las invariantes del dominio:
///
/// - `zoom_x` y `zoom_y` deben estar siempre en `[min_zoom, max_zoom]`.
/// - `min_zoom` debe ser siempre `<= max_zoom`.
/// - `zoom_x`, `zoom_y`, `offset_x`, `offset_y` deben ser siempre finitos.
/// - `delta_x == 0.0 && delta_y == 0.0` en `zoom_at` no modifica el estado.
///
/// ## Ownership
///
/// `Viewport` es el único dueño del estado de escala del chart:
/// `zoom_x`, `zoom_y`, `offset_x`, `offset_y`, `min_zoom` y `max_zoom`.
/// Ningún otro tipo debe duplicar este estado. Antes de esta consolidación,
/// `ChartScales`/`TimeScale`/`PriceScale` en `chart::scale` duplicaban el
/// mismo estado; fueron eliminados (ver T11).
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use crate::domain::transforms;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Margen porcentual usado en auto_scale y clamping (5% cada lado).
const MARGIN_RATIO: f64 = 0.05;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Nivel de zoom del eje X (tiempo / índice de vela).
    /// 1.0 = sin zoom (1 unidad de precio = 1 píxel base).
    /// Valores mayores = más cerca (más detalle).
    /// Valores menores = más alejado (más velas visibles).
    zoom_x: f64,

    /// Nivel de zoom del eje Y (precio).
    /// 1.0 = escala base. Valores mayores = precios más ampliados verticalmente.
    zoom_y: f64,

    /// Desplazamiento horizontal en coordenadas de datos (índices de vela).
    /// Positivo = se mueve a la derecha (velas más recientes a la izquierda).
    offset_x: f64,

    /// Desplazamiento vertical en coordenadas de datos (precio).
    /// Positivo = se mueve hacia arriba (precios más altos en el centro).
    offset_y: f64,

    /// Factor de zoom por cada "tick" de scroll del mouse.
    zoom_step: f64,

    /// Nivel mínimo de zoom permitido (ambos ejes).
    min_zoom: f64,

    /// Nivel máximo de zoom permitido (ambos ejes).
    max_zoom: f64,
}

impl Viewport {
    /// Crea un viewport con valores por defecto.
    pub fn new() -> Self {
        Self {
            zoom_x: 1.0,
            zoom_y: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom_step: 0.01,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Devuelve el nivel de zoom actual del eje X (tiempo / índice de vela).
    pub fn zoom_x(&self) -> f64 {
        self.zoom_x
    }

    /// Devuelve el nivel de zoom actual del eje Y (precio).
    pub fn zoom_y(&self) -> f64 {
        self.zoom_y
    }

    /// Devuelve el desplazamiento horizontal actual (índices de vela).
    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }

    /// Devuelve el desplazamiento vertical actual (precio).
    pub fn offset_y(&self) -> f64 {
        self.offset_y
    }

    /// Devuelve el factor de zoom por tick de scroll.
    pub fn zoom_step(&self) -> f64 {
        self.zoom_step
    }

    /// Devuelve el nivel mínimo de zoom permitido.
    pub fn min_zoom(&self) -> f64 {
        self.min_zoom
    }

    /// Devuelve el nivel máximo de zoom permitido.
    pub fn max_zoom(&self) -> f64 {
        self.max_zoom
    }

    /// Devuelve el espaciamiento entre centros de velas en píxeles de pantalla (eje X).
    ///
    /// A mayor `zoom_x`, mayor espaciamiento (las velas están más separadas).
    /// Consiste con el ancho de vela usado en el renderer.
    pub fn candle_spacing(&self) -> f64 {
        transforms::candle_spacing(self.zoom_x)
    }

    /// Establece el nivel mínimo de zoom permitido (ambos ejes).
    ///
    /// Si el nuevo valor es mayor que `max_zoom`, se ajusta `max_zoom`
    /// para mantener `min_zoom <= max_zoom`. Se clampean `zoom_x` y `zoom_y`
    /// al nuevo rango.
    pub fn set_min_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.min_zoom = value;
        if self.min_zoom > self.max_zoom {
            self.max_zoom = self.min_zoom;
        }
        // Asegurar que ambos niveles de zoom estén dentro del nuevo rango.
        self.zoom_x = self.zoom_x.clamp(self.min_zoom, self.max_zoom);
        self.zoom_y = self.zoom_y.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el nivel máximo de zoom permitido (ambos ejes).
    ///
    /// Si el nuevo valor es menor que `min_zoom`, se ajusta `min_zoom`
    /// para mantener `min_zoom <= max_zoom`. Se clampean `zoom_x` y `zoom_y`
    /// al nuevo rango.
    pub fn set_max_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.max_zoom = value;
        if self.max_zoom < self.min_zoom {
            self.min_zoom = self.max_zoom;
        }
        // Asegurar que ambos niveles de zoom estén dentro del nuevo rango.
        self.zoom_x = self.zoom_x.clamp(self.min_zoom, self.max_zoom);
        self.zoom_y = self.zoom_y.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el nivel de zoom de ambos ejes directamente.
    ///
    /// Cada valor se clampea a `[min_zoom, max_zoom]`. Si no es finito, se ignora ese eje.
    pub fn set_zoom(&mut self, x: f64, y: f64) {
        self.set_zoom_x(x);
        self.set_zoom_y(y);
    }

    /// Establece el nivel de zoom del eje X directamente.
    ///
    /// El valor se clampea a `[min_zoom, max_zoom]`. Si no es finito, se ignora.
    pub fn set_zoom_x(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.zoom_x = value.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el nivel de zoom del eje Y directamente.
    ///
    /// El valor se clampea a `[min_zoom, max_zoom]`. Si no es finito, se ignora.
    pub fn set_zoom_y(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.zoom_y = value.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el desplazamiento horizontal directamente.
    ///
    /// Si no es finito, se ignora.
    pub fn set_offset_x(&mut self, value: f64) {
        if value.is_finite() {
            self.offset_x = value;
        }
    }

    /// Establece el desplazamiento vertical directamente.
    ///
    /// Si no es finito, se ignora.
    pub fn set_offset_y(&mut self, value: f64) {
        if value.is_finite() {
            self.offset_y = value;
        }
    }

    /// Limita el pan horizontal para que la primera y última vela
    /// permanezcan visibles dentro de la pantalla.
    ///
    /// `candle_count` es el número total de velas cargadas.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Se deja un margen del 5% en cada lado.
    pub fn clamp_pan_to_candles(&mut self, candle_count: usize, screen_width: f64) {
        if candle_count == 0 || !screen_width.is_finite() || screen_width <= 0.0 {
            return;
        }

        let candle_spacing = self.candle_spacing();
        if candle_spacing <= 0.0 || !candle_spacing.is_finite() {
            return;
        }
        let center_x = screen_width / 2.0;
        let margin = screen_width * MARGIN_RATIO;

        // Primera vela (index 0) no puede salir por la izquierda:
        // data_x_to_screen(0) >= margin
        // center_x + offset_x * candle_spacing >= margin
        // offset_x >= (margin - center_x) / candle_spacing
        let min_offset_x = (margin - center_x) / candle_spacing;

        // Última vela (index N-1) no puede salir por la derecha:
        // data_x_to_screen(N-1) <= screen_width - margin
        // center_x + (N-1) * candle_spacing + offset_x * candle_spacing <= screen_width - margin
        // offset_x <= (screen_width - margin - center_x - (N-1) * candle_spacing) / candle_spacing
        let max_offset_x =
            (screen_width - margin - center_x - (candle_count as f64 - 1.0) * candle_spacing)
                / candle_spacing;

        // Si las velas no caben en pantalla (min_offset_x > max_offset_x),
        // el pan horizontal no tiene límites significativos: el usuario debe
        // poder hacer scroll libremente. No aplicar clamp para evitar panic
        // cuando min > max.
        if min_offset_x <= max_offset_x {
            self.offset_x = self.offset_x.clamp(min_offset_x, max_offset_x);
        }
    }

    /// Limita el zoom del eje X para que una sola vela no llene toda la pantalla
    /// y para que al menos dos velas quepan simultáneamente.
    ///
    /// `candle_count` es el número total de velas cargadas.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Solo afecta al eje X (horizontal). El eje Y no tiene un equivalente de
    /// "velas quepan", así que queda sujeto únicamente a `[min_zoom, max_zoom]`.
    pub fn clamp_zoom_to_candles(&mut self, candle_count: usize, screen_width: f64) {
        if candle_count == 0 || !screen_width.is_finite() || screen_width <= 0.0 {
            return;
        }

        // candle_spacing = CANDLE_SPACING_BASE * zoom_x
        // Máximo zoom X: candle_spacing >= 4.0 => zoom_x <= 5.0
        // (una vela base de 20px no ocupa más de 100px de ancho).
        let max_zoom_by_candle = 5.0;

        let min_allowed = self.min_zoom;
        let max_allowed = self.max_zoom.min(max_zoom_by_candle);

        if min_allowed <= max_allowed {
            self.zoom_x = self.zoom_x.clamp(min_allowed, max_allowed);
        }
    }

    /// Aplica un zoom alrededor de un punto dado (coordenadas de pantalla), por eje de forma independiente.
    ///
    /// `delta_x` y `delta_y` son el cambio de scroll: positivo = zoom in, negativo = zoom out.
    /// `center_x` y `center_y` son el punto alrededor del cual hacer zoom
    /// como fracciones 0..=1 del área de pantalla (0.0 = izquierda/arriba, 1.0 = derecha/abajo).
    /// `screen_width` y `screen_height` son las dimensiones del área de dibujo en píxeles.
    ///
    /// Si `delta_x == 0.0`, el eje X no se modifica. Si `delta_y == 0.0`, el eje Y no se modifica.
    /// Si alguno de los parámetros no es finito, no se modifica el estado.
    pub fn zoom_at(
        &mut self,
        delta_x: f64,
        delta_y: f64,
        center_x: f64,
        center_y: f64,
        screen_width: f64,
        screen_height: f64,
    ) {
        if !delta_x.is_finite()
            || !delta_y.is_finite()
            || !center_x.is_finite()
            || !center_y.is_finite()
            || !screen_width.is_finite()
            || !screen_height.is_finite()
        {
            return;
        }

        if delta_x == 0.0 && delta_y == 0.0 {
            return;
        }

        let factor_x = if delta_x > 0.0 {
            1.0 + self.zoom_step
        } else if delta_x < 0.0 {
            1.0 - self.zoom_step
        } else {
            1.0
        };
        let factor_y = if delta_y > 0.0 {
            1.0 + self.zoom_step
        } else if delta_y < 0.0 {
            1.0 - self.zoom_step
        } else {
            1.0
        };

        let new_zoom_x = (self.zoom_x * factor_x).clamp(self.min_zoom, self.max_zoom);
        let new_zoom_y = (self.zoom_y * factor_y).clamp(self.min_zoom, self.max_zoom);

        // Convertir coordenadas normalizadas 0..=1 a píxeles de pantalla.
        let center_x_px = center_x * screen_width;
        let center_y_px = center_y * screen_height;

        let zoom_ratio_x = new_zoom_x / self.zoom_x;
        let zoom_ratio_y = new_zoom_y / self.zoom_y;

        let offset_x_px = self.offset_x * transforms::candle_spacing(self.zoom_x);
        let offset_y_px = self.offset_y * screen_height * self.zoom_y;

        let new_offset_x_px = center_x_px + (offset_x_px - center_x_px) * zoom_ratio_x;
        let new_offset_y_px = center_y_px + (offset_y_px - center_y_px) * zoom_ratio_y;

        self.offset_x = new_offset_x_px / transforms::candle_spacing(new_zoom_x);
        self.offset_y = new_offset_y_px / (screen_height * new_zoom_y);

        self.zoom_x = new_zoom_x;
        self.zoom_y = new_zoom_y;

        tracing::debug!(
            "Zoom: x={:.2}, y={:.2}, offset=({:.2}, {:.2})",
            self.zoom_x,
            self.zoom_y,
            self.offset_x,
            self.offset_y
        );
    }

    /// Aplica un desplazamiento (pan) al viewport.
    ///
    /// `dx` y `dy` son el cambio en píxeles de pantalla.
    /// `screen_height` es la altura real del área de dibujo en píxeles.
    ///
    /// Si alguno de los parámetros no es finito, o la altura no es positiva,
    /// no se modifica el estado.
    pub fn pan(&mut self, dx: f64, dy: f64, screen_height: f64) {
        if !dx.is_finite() || !dy.is_finite() || !screen_height.is_finite() || screen_height <= 0.0
        {
            return;
        }

        self.offset_x += dx / self.candle_spacing();
        self.offset_y -= dy / (screen_height * self.zoom_y);

        tracing::trace!(
            "Pan: offset=({:.2}, {:.2}), zoom=({:.2}, {:.2})",
            self.offset_x,
            self.offset_y,
            self.zoom_x,
            self.zoom_y
        );
    }

    /// Ajusta automáticamente el zoom y el offset vertical para que todo el
    /// rango de precios quepa en el área de dibujo con un margen del 5% en
    /// cada lado.
    ///
    /// `price_range` es el rango de precios (mínimo y máximo) de las velas a mostrar.
    /// `candle_count` es la cantidad total de velas (solo se usa para validar
    /// que haya algo que mostrar).
    /// `screen_width` y `screen_height` son las dimensiones del área de dibujo en píxeles.
    ///
    /// `auto_scale` ajusta solo el eje Y (precio): `zoom_y` se rige por la
    /// escala de precio y `offset_y` se centra en el punto medio del rango.
    /// El eje X (tiempo y velas) se deja intacto: el llamante puede seguir
    /// ajustando `zoom_x`, `offset_x` con `zoom_at` o `pan`.
    pub fn auto_scale(
        &mut self,
        price_range: PriceRange,
        candle_count: usize,
        screen_width: f64,
        screen_height: f64,
    ) {
        if candle_count == 0
            || !screen_width.is_finite()
            || !screen_height.is_finite()
            || screen_width <= 0.0
            || screen_height <= 0.0
        {
            return;
        }

        let min_price_f64 = price_range
            .min()
            .value()
            .to_f64()
            .expect("precio mínimo convertible");
        let max_price_f64 = price_range
            .max()
            .value()
            .to_f64()
            .expect("precio máximo convertible");

        let Some(price_range_value) =
            transforms::validate_price_range(min_price_f64, max_price_f64)
        else {
            return;
        };

        let margin = MARGIN_RATIO;
        let available_height = screen_height * (1.0 - 2.0 * margin);

        let zoom_vertical_max = available_height / screen_height;

        let max_allowed_y = zoom_vertical_max.min(self.max_zoom);
        let new_zoom_y = if self.min_zoom > max_allowed_y {
            self.min_zoom
        } else {
            self.zoom_y.clamp(self.min_zoom, max_allowed_y)
        };

        self.offset_y = 0.0;
        self.zoom_y = new_zoom_y;

        tracing::debug!(
            "Autoescala (solo eje Y/precio): zoom_y={:.4}, offset_y={:.4}, \
             rango_precio={:.2}..{:.2}, centro_precio={:.2}, \
             pantalla={:.0}x{:.0}, rango_valor={:.2}",
            self.zoom_y,
            self.offset_y,
            min_price_f64,
            max_price_f64,
            price_range.midpoint().value(),
            screen_width,
            screen_height,
            price_range_value
        );
    }

    /// Restablece el viewport a valores por defecto.
    pub fn reset(&mut self) {
        self.zoom_x = 1.0;
        self.zoom_y = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        tracing::debug!("Viewport restablecido a valores por defecto");
    }

    /// Convierte un índice de vela (coordenada de datos X) a coordenada de pantalla.
    ///
    /// `index` es el índice de la vela en el array (basado en 0).
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve la coordenada X en píxeles de pantalla. Usa `zoom_x` (eje X).
    pub fn data_x_to_screen(&self, index: usize, screen_width: f64) -> f64 {
        transforms::data_x_to_screen(index, screen_width, self.zoom_x, self.offset_x)
    }

    /// Convierte una coordenada de pantalla X a un índice de vela (coordenada de datos X).
    ///
    /// `screen_x` es la posición en píxeles de pantalla.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve el índice de vela más cercano (basado en 0), clampado a
    /// `[0, candle_count - 1]`. Usa `zoom_x` (eje X).
    pub fn screen_x_to_data(&self, screen_x: f64, screen_width: f64, candle_count: usize) -> usize {
        transforms::screen_x_to_data(
            screen_x,
            screen_width,
            self.zoom_x,
            self.offset_x,
            candle_count,
        )
    }

    /// Convierte un precio (coordenada de datos Y) a coordenada de pantalla.
    ///
    /// `price` es el precio a convertir.
    /// `screen_height` es el alto del área de dibujo en píxeles.
    /// `price_range` es el rango de precios visible (mínimo y máximo).
    /// `center_price` es el precio central alrededor del cual se hace zoom.
    ///
    /// Devuelve la coordenada Y en píxeles de pantalla (eje Y invertido: 0 es arriba).
    /// Usa `zoom_y` (eje Y).
    pub fn price_to_screen(
        &self,
        price: Price,
        screen_height: f64,
        price_range: PriceRange,
        center_price: Price,
    ) -> f64 {
        if !screen_height.is_finite() || screen_height <= 0.0 {
            return screen_height / 2.0;
        }

        let center_y = screen_height / 2.0;

        let min_price_f64 = price_range
            .min()
            .value()
            .to_f64()
            .expect("el precio mínimo debe ser convertible a f64 para renderizado");
        let max_price_f64 = price_range
            .max()
            .value()
            .to_f64()
            .expect("el precio máximo debe ser convertible a f64 para renderizado");
        let Some(range_value_f64) = transforms::validate_price_range(min_price_f64, max_price_f64)
        else {
            return center_y;
        };

        let pixels_per_unit = screen_height / range_value_f64 * self.zoom_y;
        let price_f64 = price
            .value()
            .to_f64()
            .expect("el precio debe ser convertible a f64 para renderizado");
        let center_price_f64 = center_price
            .value()
            .to_f64()
            .expect("el precio central debe ser convertible a f64 para renderizado");
        let relative_price = price_f64 - center_price_f64 - self.offset_y * range_value_f64;
        center_y - relative_price * pixels_per_unit
    }

    /// Convierte una coordenada de pantalla Y a un precio (coordenada de datos Y).
    ///
    /// `screen_y` es la posición en píxeles de pantalla.
    /// `screen_height` es el alto del área de dibujo en píxeles.
    /// `price_range` es el rango de precios visible (mínimo y máximo).
    /// `center_price` es el precio central alrededor del cual se hace zoom.
    ///
    /// Devuelve el `Price` calculado. Si el resultado no es un precio válido
    /// (negativo o no convertible), devuelve `center_price` como fallback.
    /// Usa `zoom_y` (eje Y).
    pub fn screen_y_to_price(
        &self,
        screen_y: f64,
        screen_height: f64,
        price_range: PriceRange,
        center_price: Price,
    ) -> Price {
        let center_y = screen_height / 2.0;
        let range_value = price_range.max().value() - price_range.min().value();
        let range_value_f64 = range_value
            .to_f64()
            .expect("el rango de precios debe ser convertible a f64 para renderizado");
        let pixels_per_unit = screen_height / range_value_f64 * self.zoom_y;
        let relative_price = (center_y - screen_y) / pixels_per_unit;
        let center_price_f64 = center_price
            .value()
            .to_f64()
            .expect("el precio central debe ser convertible a f64 para renderizado");
        let price_value = center_price_f64 + self.offset_y * range_value_f64 + relative_price;

        let decimal = Decimal::from_f64(price_value);
        match decimal {
            Some(d) => Price::new(d).unwrap_or(center_price),
            None => center_price,
        }
    }

    /// Calcula el rango de velas visibles en el área de dibujo.
    ///
    /// Una vela se considera visible si su centro cae dentro de
    /// `[0, screen_width]` según `data_x_to_screen`. El renderer debe
    /// iterar solo `first..=last` para lograr O(N_visible) por frame.
    /// Usa `zoom_x` (eje X).
    pub fn visible_range(
        &self,
        candle_count: usize,
        screen_width: f64,
    ) -> transforms::VisibleRange {
        if candle_count == 0 {
            return transforms::VisibleRange::empty();
        }

        let spacing = self.candle_spacing();
        let center_x = screen_width / 2.0;

        // data_x_to_screen(i) = center_x + i * spacing + offset_x * spacing
        // Visible si 0 <= data_x_to_screen(i) <= screen_width.
        let first = ((-center_x - self.offset_x * spacing) / spacing)
            .ceil()
            .max(0.0) as usize;
        let last = ((center_x - self.offset_x * spacing) / spacing)
            .floor()
            .min(candle_count as f64 - 1.0) as usize;

        if first > last {
            return transforms::VisibleRange::empty();
        }

        transforms::VisibleRange {
            first: first.min(candle_count.saturating_sub(1)),
            last,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zoom_axes_are_independent() {
        let mut viewport = Viewport::new();
        viewport.set_zoom_x(2.0);
        viewport.set_zoom_y(5.0);
        assert!((viewport.zoom_x() - 2.0).abs() < f64::EPSILON);
        assert!((viewport.zoom_y() - 5.0).abs() < f64::EPSILON);

        viewport.set_zoom_x(3.0);
        assert!((viewport.zoom_x() - 3.0).abs() < f64::EPSILON);
        assert!((viewport.zoom_y() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zoom_clamps_to_limits() {
        let mut viewport = Viewport::new();
        viewport.set_zoom(100.0, 100.0);
        assert!((viewport.zoom_x() - 10.0).abs() < f64::EPSILON);
        assert!((viewport.zoom_y() - 10.0).abs() < f64::EPSILON);
        viewport.set_zoom(0.01, 0.01);
        assert!((viewport.zoom_x() - 0.1).abs() < f64::EPSILON);
        assert!((viewport.zoom_y() - 0.1).abs() < f64::EPSILON);
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
