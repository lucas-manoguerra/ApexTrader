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
/// - **ViewportSpace**: estado de la cámara (zoom_level, min_zoom, max_zoom).
/// - **ScreenSpace**: píxeles de pantalla (0..=screen_width, 0..=screen_height).
///
/// Las transformaciones entre espacios son explícitas y unidireccionales.
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
use crate::domain::transforms;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

/// Margen porcentual usado en auto_scale y clamping (5% cada lado).
const MARGIN_RATIO: f64 = 0.05;

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Nivel de zoom. 1.0 = sin zoom (1 unidad de precio = 1 píxel base).
    /// Valores mayores = más cerca (más detalle).
    /// Valores menores = más alejado (más velas visibles).
    zoom_level: f64,

    /// Desplazamiento horizontal en coordenadas de datos (índices de vela).
    /// Positivo = se mueve a la derecha (velas más recientes a la izquierda).
    offset_x: f64,

    /// Desplazamiento vertical en coordenadas de datos (precio).
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

    /// Devuelve el espaciamiento entre centros de velas en píxeles de pantalla.
    ///
    /// A mayor zoom, mayor espaciamiento (las velas están más separadas).
    /// Consiste con el ancho de vela usado en el renderer.
    pub fn candle_spacing(&self) -> f64 {
        transforms::candle_spacing(self.zoom_level)
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

        self.offset_x = self.offset_x.clamp(min_offset_x, max_offset_x);
    }

    /// Limita el zoom para que una sola vela no llene toda la pantalla
    /// y para que al menos dos velas quepan simultáneamente.
    ///
    /// `candle_count` es el número total de velas cargadas.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    pub fn clamp_zoom_to_candles(&mut self, candle_count: usize, screen_width: f64) {
        if candle_count == 0 || !screen_width.is_finite() || screen_width <= 0.0 {
            return;
        }

        // candle_spacing = CANDLE_SPACING_BASE * zoom_level
        // Máximo zoom: candle_spacing >= 4.0 => zoom_level <= 5.0 / CANDLE_SPACING_BASE * CANDLE_SPACING_BASE
        //                simplified: zoom_level <= 5.0 (una vela de 20px no ocupa más de 100px)
        // Mínimo zoom: al menos 2 velas quepan => 2 * candle_spacing <= screen_width
        //                => 2 * CANDLE_SPACING_BASE * zoom_level <= screen_width
        //                => zoom_level <= screen_width / (2 * CANDLE_SPACING_BASE)
        // Nota: para el mínimo zoom queremos que AL MENOS 2 velas quepan,
        // así que zoom_level NO puede ser tan pequeño que 2 velas ocupen más que la pantalla.
        // Es decir: 2 * CANDLE_SPACING_BASE * zoom_level <= screen_width
        // => zoom_level <= screen_width / (2 * CANDLE_SPACING_BASE)
        // Pero eso es un LÍMITE SUPERIOR, no inferior.
        //
        // El límite inferior para zoom viene de min_zoom del usuario.
        // Para evitar que una vela llene toda la pantalla, limitamos zoom_level <= 5.0.
        let max_zoom_by_candle = 5.0;

        let min_allowed = self.min_zoom;
        let max_allowed = self.max_zoom.min(max_zoom_by_candle);

        if min_allowed <= max_allowed {
            self.zoom_level = self.zoom_level.clamp(min_allowed, max_allowed);
        }
    }

    /// Aplica un zoom alrededor de un punto dado (coordenadas de pantalla).
    ///
    /// `delta` es el cambio de scroll: positivo = zoom in, negativo = zoom out.
    /// `center_x` y `center_y` son el punto alrededor del cual hacer zoom
    /// como fracciones 0..=1 del área de pantalla (0.0 = izquierda/arriba, 1.0 = derecha/abajo).
    /// `screen_width` y `screen_height` son las dimensiones del área de dibujo en píxeles.
    ///
    /// Si `delta == 0.0`, no se modifica el estado.
    /// Si alguno de los parámetros no es finito, no se modifica el estado.
    pub fn zoom_at(
        &mut self,
        delta: f64,
        center_x: f64,
        center_y: f64,
        screen_width: f64,
        screen_height: f64,
    ) {
        // Validar finitud de parámetros.
        if !delta.is_finite()
            || !center_x.is_finite()
            || !center_y.is_finite()
            || !screen_width.is_finite()
            || !screen_height.is_finite()
        {
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

        // Convertir coordenadas normalizadas 0..=1 a píxeles de pantalla.
        let center_x_px = center_x * screen_width;
        let center_y_px = center_y * screen_height;

        // Ajustar offsets para que el zoom sea centrado en el punto dado.
        // Cuando hacemos zoom, el punto center debe quedarse fijo en pantalla.
        let zoom_ratio = new_zoom / self.zoom_level;

        // Convertir offset_x (índices de vela) a píxeles, aplicar zoom, volver a índices.
        let offset_x_px = self.offset_x * self.candle_spacing();
        let offset_y_px = self.offset_y * screen_height / self.zoom_level;

        let new_offset_x_px = center_x_px + (offset_x_px - center_x_px) * zoom_ratio;
        let new_offset_y_px = center_y_px + (offset_y_px - center_y_px) * zoom_ratio;

        self.offset_x = new_offset_x_px / self.candle_spacing_at(new_zoom);
        self.offset_y = new_offset_y_px * new_zoom / screen_height;

        self.zoom_level = new_zoom;

        tracing::debug!(
            "Zoom: level={:.2}, offset=({:.2}, {:.2})",
            self.zoom_level,
            self.offset_x,
            self.offset_y
        );
    }

    /// Calcula el candle_spacing para un nivel de zoom dado.
    fn candle_spacing_at(&self, zoom_level: f64) -> f64 {
        transforms::candle_spacing(zoom_level)
    }

    /// Aplica un desplazamiento (pan) al viewport.
    ///
    /// `dx` y `dy` son el cambio en píxeles de pantalla.
    /// `screen_height` es la altura real del área de dibujo en píxeles.
    ///
    /// Si alguno de los parámetros no es finito, o la altura no es positiva,
    /// no se modifica el estado.
    pub fn pan(&mut self, dx: f64, dy: f64, screen_height: f64) {
        if !dx.is_finite()
            || !dy.is_finite()
            || !screen_height.is_finite()
            || screen_height <= 0.0
        {
            return;
        }

        // El pan se aplica inversamente: si arrastramos a la derecha,
        // el contenido se mueve a la derecha, así que offset_x aumenta.
        // Convertir píxeles a índices de vela usando candle_spacing.
        self.offset_x += dx / self.candle_spacing();
        // offset_y está en unidades de precio, no de pantalla. Para simplicidad
        // y consistencia con price_to_screen, usamos el mismo factor que el eje Y.
        self.offset_y += dy * self.zoom_level / screen_height;

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

        let margin = MARGIN_RATIO;
        let available_height = screen_height * (1.0 - 2.0 * margin);
        let available_width = screen_width * (1.0 - 2.0 * margin);

        // Verticalmente: el rango de precios ocupa screen_height * zoom_level píxeles.
        // Para que quepa en available_height: zoom_level <= available_height / screen_height.
        let zoom_vertical_max = available_height / screen_height;

        // Horizontalmente: ancho_total = candle_count * candle_spacing = candle_count *
        // CANDLE_SPACING_BASE * zoom_level. Para que quepa en available_width:
        // zoom_level <= available_width / (candle_count * CANDLE_SPACING_BASE)
        let zoom_horizontal_max =
            available_width / (candle_count as f64 * transforms::CANDLE_SPACING_BASE);

        // Tomar el mínimo para asegurar que todo quepa en ambas dimensiones.
        let max_allowed = zoom_vertical_max
            .min(zoom_horizontal_max)
            .min(self.max_zoom);

        // Si ni siquiera el zoom mínimo permite que quepan todas las velas
        // horizontalmente, usamos min_zoom y aceptamos scroll horizontal.
        let new_zoom = if self.min_zoom > max_allowed {
            self.min_zoom
        } else {
            self.zoom_level.clamp(self.min_zoom, max_allowed)
        };

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
        transforms::data_x_to_screen(index, screen_width, self.zoom_level, self.offset_x)
    }

    /// Convierte una coordenada de pantalla X a un índice de vela (coordenada de datos X).
    ///
    /// `screen_x` es la posición en píxeles de pantalla.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve el índice de vela más cercano (basado en 0).
    pub fn screen_x_to_data(&self, screen_x: f64, screen_width: f64) -> usize {
        transforms::screen_x_to_data(screen_x, screen_width, self.zoom_level, self.offset_x)
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
