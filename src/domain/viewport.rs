/// Viewport del gráfico: estado de zoom y pan.
///
/// `Viewport` **posee** el estado de la cámara del chart.
/// Es responsable de calcular cómo se transforman las coordenadas de datos
/// (precio, tiempo) a coordenadas de pantalla y viceversa.
///
/// Ningún otro módulo puede modificar el viewport directamente;
/// solo a través de sus métodos públicos.
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// Nivel de zoom. 1.0 = sin zoom (1 unidad de precio = 1 píxel base).
    /// Valores mayores = más cerca (más detalle).
    /// Valores menores = más alejado (más velas visibles).
    pub zoom_level: f64,

    /// Desplazamiento horizontal en coordenadas de datos.
    /// Positivo = se mueve a la derecha (velas más recientes a la izquierda).
    pub offset_x: f64,

    /// Desplazamiento vertical en coordenadas de datos.
    /// Positivo = se mueve hacia arriba (precios más altos en el centro).
    pub offset_y: f64,

    /// Factor de zoom por cada "tick" de scroll del mouse.
    pub zoom_step: f64,

    /// Nivel mínimo de zoom permitido.
    pub min_zoom: f64,

    /// Nivel máximo de zoom permitido.
    pub max_zoom: f64,
}

impl Viewport {
    /// Crea un viewport con valores por defecto.
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            zoom_step: 0.1,
            min_zoom: 0.1,
            max_zoom: 10.0,
        }
    }

    /// Aplica un zoom alrededor de un punto dado (coordenadas de pantalla).
    ///
    /// `delta` es el cambio de scroll: positivo = zoom in, negativo = zoom out.
    /// `center_x` y `center_y` son el punto alrededor del cual hacer zoom
    /// (posiciones en coordenadas de pantalla normalizadas 0..=1).
    pub fn zoom_at(&mut self, delta: f64, center_x: f64, center_y: f64) {
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
    pub fn pan(&mut self, dx: f64, dy: f64) {
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
