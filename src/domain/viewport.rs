/// Viewport del gráfico: estado de zoom y pan.
///
/// `Viewport` **posee** el estado de la cámara del chart.
/// Es responsable de calcular cómo se transforman las coordenadas de datos
/// (precio, tiempo) a coordenadas de pantalla y viceversa.
///
/// Todos los campos son privados. Se modifican únicamente a través de
/// métodos públicos que validan las invariantes del dominio:
///
/// - `zoom_level` debe estar siempre en `[min_zoom, max_zoom]`.
/// - `min_zoom` debe ser siempre `<= max_zoom`.
/// - `zoom_level`, `offset_x`, `offset_y` deben ser siempre finitos.
/// - `delta == 0.0` en `zoom_at` no modifica el estado.
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Nivel de zoom. 1.0 = sin zoom (1 unidad de precio = 1 píxel base).
    /// Valores mayores = más cerca (más detalle).
    /// Valores menores = más alejado (más velas visibles).
    zoom_level: f64,

    /// Desplazamiento horizontal en coordenadas de datos.
    /// Positivo = se mueve a la derecha (velas más recientes a la izquierda).
    offset_x: f64,

    /// Desplazamiento vertical en coordenadas de datos.
    /// Positivo = se mueve hacia arriba (precios más altos en el centro).
    offset_y: f64,

    /// Factor de zoom por cada "tick" de scroll del mouse.
    zoom_step: f64,

    /// Nivel mínimo de zoom permitido.
    min_zoom: f64,

    /// Nivel máximo de zoom permitido.
    max_zoom: f64,
}

impl Viewport {
    /// Crea un viewport con valores por defecto.
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom_step: 0.01,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Devuelve el nivel de zoom actual.
    pub fn zoom_level(&self) -> f64 {
        self.zoom_level
    }

    /// Devuelve el desplazamiento horizontal actual.
    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }

    /// Devuelve el desplazamiento vertical actual.
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

    /// Establece el nivel mínimo de zoom.
    ///
    /// Si el nuevo valor es mayor que `max_zoom`, se ajusta `max_zoom`
    /// para mantener `min_zoom <= max_zoom`.
    pub fn set_min_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.min_zoom = value;
        if self.min_zoom > self.max_zoom {
            self.max_zoom = self.min_zoom;
        }
        // Asegurar que zoom_level esté dentro del nuevo rango.
        self.zoom_level = self.zoom_level.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el nivel máximo de zoom permitido.
    ///
    /// Si el nuevo valor es menor que `min_zoom`, se ajusta `min_zoom`
    /// para mantener `min_zoom <= max_zoom`.
    pub fn set_max_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.max_zoom = value;
        if self.max_zoom < self.min_zoom {
            self.min_zoom = self.max_zoom;
        }
        // Asegurar que zoom_level esté dentro del nuevo rango.
        self.zoom_level = self.zoom_level.clamp(self.min_zoom, self.max_zoom);
    }

    /// Establece el nivel de zoom directamente.
    ///
    /// El valor se clampea a `[min_zoom, max_zoom]`. Si no es finito, se ignora.
    pub fn set_zoom_level(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.zoom_level = value.clamp(self.min_zoom, self.max_zoom);
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

    /// Aplica un zoom alrededor de un punto dado (coordenadas de pantalla).
    ///
    /// `delta` es el cambio de scroll: positivo = zoom in, negativo = zoom out.
    /// `center_x` y `center_y` son el punto alrededor del cual hacer zoom
    /// (posiciones en coordenadas de pantalla normalizadas 0..=1).
    ///
    /// Si `delta == 0.0`, no se modifica el estado.
    /// Si alguno de los parámetros no es finito, no se modifica el estado.
    pub fn zoom_at(&mut self, delta: f64, center_x: f64, center_y: f64) {
        // Validar finitud de parámetros.
        if !delta.is_finite() || !center_x.is_finite() || !center_y.is_finite() {
            return;
        }

        // Ignorar eventos sin movimiento.
        if delta == 0.0 {
            return;
        }

        let zoom_factor = if delta > 0.0 {
            1.0 + self.zoom_step
        } else {
            1.0 - self.zoom_step
        };

        let new_zoom = (self.zoom_level * zoom_factor).clamp(self.min_zoom, self.max_zoom);

        // Ajustar offsets para que el zoom sea centrado en el punto dado.
        // Cuando hacemos zoom, el punto center debe quedarse fijo en pantalla.
        let zoom_ratio = new_zoom / self.zoom_level;

        self.offset_x = center_x + (self.offset_x - center_x) * zoom_ratio;
        self.offset_y = center_y + (self.offset_y - center_y) * zoom_ratio;

        self.zoom_level = new_zoom;

        tracing::debug!(
            "Zoom: level={:.2}, offset=({:.2}, {:.2})",
            self.zoom_level,
            self.offset_x,
            self.offset_y
        );
    }

    /// Aplica un desplazamiento (pan) al viewport.
    ///
    /// `dx` y `dy` son el cambio en coordenadas de pantalla.
    ///
    /// Si alguno de los parámetros no es finito, no se modifica el estado.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }

        // El pan se aplica inversamente: si arrastramos a la derecha,
        // el contenido se mueve a la derecha, así que offset_x aumenta.
        self.offset_x += dx / self.zoom_level;
        self.offset_y += dy / self.zoom_level;

        tracing::trace!(
            "Pan: offset=({:.2}, {:.2}), zoom={:.2}",
            self.offset_x,
            self.offset_y,
            self.zoom_level
        );
    }

    /// Ajusta automáticamente el zoom y los offsets para que todas las velas
    /// quepan en el área de dibujo con un margen del 5% en cada lado.
    ///
    /// `price_range` es el rango de precios (mínimo y máximo) de las velas a mostrar.
    /// `candle_count` es la cantidad total de velas.
    /// `screen_width` y `screen_height` son las dimensiones del área de dibujo en píxeles.
    ///
    /// Después de llamar, `zoom_level`, `offset_x` y `offset_y` quedan configurados
    /// para que todas las velas sean visibles. El llamante puede seguir ajustando
    /// manualmente con `zoom_at` o `pan`.
    pub fn auto_scale(
        &mut self,
        price_range: PriceRange,
        candle_count: usize,
        screen_width: f64,
        screen_height: f64,
    ) {
        let min_price = price_range.min().value();
        let max_price = price_range.max().value();

        let margin = 0.05;
        let available_height = screen_height * (1.0 - 2.0 * margin);
        let available_width = screen_width * (1.0 - 2.0 * margin);

        // En price_to_screen: pixels_per_unit = screen_height / range_f64 * zoom_level.
        // El rango de precios ocupa screen_height * zoom_level píxeles verticalmente.
        // Para que quepa en available_height: zoom_level <= available_height / screen_height.
        let zoom_vertical_max = available_height / screen_height;

        // Horizontalmente: ancho_total = candle_count * candle_spacing =
        // candle_count * candle_spacing_base / zoom_level.
        // Para que quepa en available_width: zoom_level >= candle_count * candle_spacing_base / available_width.
        let candle_spacing_base = 20.0;
        let zoom_horizontal_min = (candle_count as f64 * candle_spacing_base) / available_width;

        // Tomar zoom_horizontal_min como base, pero no exceder zoom_vertical_max ni max_zoom.
        let max_allowed = zoom_vertical_max.min(self.max_zoom);
        let new_zoom = zoom_horizontal_min.clamp(self.min_zoom, max_allowed);

        // Centrar horizontalmente: la primera vela (index 0) y la última (index N-1)
        // deben quedar simétricas alrededor de center_x.
        // data_x_to_screen(0) = center_x + offset_x * candle_spacing
        // Queremos que quede en center_x - (N-1)/2 * candle_spacing.
        // Entonces offset_x = -(N-1) / 2.
        self.offset_x = -(candle_count as f64 - 1.0) / 2.0;

        // offset_y se deja en 0: el llamante pasará center_price como el punto medio
        // del rango de precios al llamar price_to_screen, lo cual centrará verticalmente.
        self.offset_y = 0.0;

        self.zoom_level = new_zoom;

        tracing::debug!(
            "Autoescala: zoom={:.4}, offset_x={:.4}, offset_y={:.4}, \
             rango_precio={:.2}..{:.2}, velas={}, pantalla={:.0}x{:.0}",
            self.zoom_level,
            self.offset_x,
            self.offset_y,
            min_price,
            max_price,
            candle_count,
            screen_width,
            screen_height
        );
    }

    /// Restablece el viewport a valores por defecto.
    pub fn reset(&mut self) {
        self.zoom_level = 1.0;
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        tracing::debug!("Viewport restablecido a valores por defecto");
    }

    /// Convierte un índice de vela (coordenada de datos X) a coordenada de pantalla.
    ///
    /// `index` es el índice de la vela en el array (basado en 0).
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve la coordenada X en píxeles de pantalla.
    pub fn data_x_to_screen(&self, index: usize, screen_width: f64) -> f64 {
        let candle_spacing = 20.0 / self.zoom_level; // píxeles por vela
        let center_x = screen_width / 2.0;
        center_x + (index as f64 * candle_spacing) + self.offset_x * candle_spacing
    }

    /// Convierte una coordenada de pantalla X a un índice de vela (coordenada de datos X).
    ///
    /// `screen_x` es la posición en píxeles de pantalla.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve el índice de vela más cercano (basado en 0).
    pub fn screen_x_to_data(&self, screen_x: f64, screen_width: f64) -> usize {
        let candle_spacing = 20.0 / self.zoom_level;
        let center_x = screen_width / 2.0;
        let index = (screen_x - center_x - self.offset_x * candle_spacing) / candle_spacing;
        index.round() as usize
    }

    /// Convierte un precio (coordenada de datos Y) a coordenada de pantalla.
    ///
    /// `price` es el precio a convertir.
    /// `screen_height` es el alto del área de dibujo en píxeles.
    /// `price_range` es el rango de precios visible (mínimo y máximo).
    /// `center_price` es el precio central alrededor del cual se hace zoom.
    ///
    /// Devuelve la coordenada Y en píxeles de pantalla (eje Y invertido: 0 es arriba).
    pub fn price_to_screen(
        &self,
        price: Price,
        screen_height: f64,
        price_range: PriceRange,
        center_price: Price,
    ) -> f64 {
        let center_y = screen_height / 2.0;
        let range_value = price_range.max().value() - price_range.min().value();
        let range_value_f64 = range_value
            .to_f64()
            .expect("el rango de precios debe ser convertible a f64 para renderizado");
        let pixels_per_unit = screen_height / range_value_f64 * self.zoom_level;
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
        let pixels_per_unit = screen_height / range_value_f64 * self.zoom_level;
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
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
