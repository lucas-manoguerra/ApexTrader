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

/// Margen utilizado para limitar el desplazamiento horizontal de las velas.
const HORIZONTAL_PAN_MARGIN_RATIO: f64 = 0.05;

/// Margen predeterminado de cada extremo de la escala vertical (5%).
const DEFAULT_PRICE_SCALE_MARGIN: f64 = 0.05;

/// Margen máximo permitido por extremo, dejando al menos un 10% de plot útil.
const MAX_PRICE_SCALE_MARGIN: f64 = 0.45;

/// Criterio usado para calcular el rango de la autoescala vertical.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriceScaleMode {
    /// Incluye los extremos de las mechas de todas las velas visibles.
    #[default]
    HighLow,
    /// Calcula el rango usando únicamente el cierre de cada vela visible.
    Close,
}

impl PriceScaleMode {
    /// Etiqueta breve para los controles de la interfaz.
    pub const fn label(self) -> &'static str {
        match self {
            Self::HighLow => "Máximo/Mínimo (OHLC)",
            Self::Close => "Solo cierres",
        }
    }
}

/// Transformación utilizada por el eje vertical de precios.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriceScaleType {
    /// Distancias lineales en unidades de precio.
    #[default]
    Linear,
    /// Distancias lineales en el logaritmo natural del precio.
    Logarithmic,
    /// Variación porcentual respecto al precio base configurado.
    Percentage,
}

impl PriceScaleType {
    /// Etiqueta breve para los controles de la interfaz.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Linear => "Lineal",
            Self::Logarithmic => "Logarítmica",
            Self::Percentage => "Porcentual (%)",
        }
    }

    /// Convierte un precio de mercado al espacio numérico del eje.
    pub fn to_scale_value(self, price: f64, percentage_base: Option<f64>) -> Option<f64> {
        if !price.is_finite() {
            return None;
        }

        let value = match self {
            Self::Linear => price,
            Self::Logarithmic if price > 0.0 => price.ln(),
            Self::Logarithmic => return None,
            Self::Percentage => {
                let base = percentage_base.filter(|base| base.is_finite() && *base > 0.0)?;
                (price / base - 1.0) * 100.0
            }
        };
        value.is_finite().then_some(value)
    }

    /// Convierte un valor del eje vertical al precio de mercado correspondiente.
    pub fn from_scale_value(self, value: f64, percentage_base: Option<f64>) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }

        let price = match self {
            Self::Linear => value,
            Self::Logarithmic => value.exp(),
            Self::Percentage => {
                let base = percentage_base.filter(|base| base.is_finite() && *base > 0.0)?;
                base * (1.0 + value / 100.0)
            }
        };
        (price.is_finite() && (self != Self::Logarithmic || price > 0.0)).then_some(price)
    }
}

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

    /// Rango y centro persistentes expresados en las unidades del tipo de escala.
    /// En modo manual no cambian cuando llegan nuevas velas.
    price_scale_range: f64,
    price_scale_center: f64,
    /// Autoajuste vertical según los extremos de las velas visibles.
    price_scale_auto: bool,
    /// Criterio de precios que determina el rango de la autoescala.
    price_scale_mode: PriceScaleMode,
    /// Transformación numérica aplicada al eje de precios.
    price_scale_type: PriceScaleType,
    /// Precio de referencia para el modo porcentual.
    price_scale_percentage_base: Option<f64>,
    /// Márgenes verticales relativos al alto del plot (0.05 = 5%).
    price_scale_top_margin: f64,
    price_scale_bottom_margin: f64,
    /// Bloquea interacciones que alteran el eje de precios.
    price_scale_locked: bool,
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
            price_scale_range: 1.0,
            price_scale_center: 0.0,
            price_scale_auto: true,
            price_scale_mode: PriceScaleMode::HighLow,
            price_scale_type: PriceScaleType::Linear,
            price_scale_percentage_base: None,
            price_scale_top_margin: DEFAULT_PRICE_SCALE_MARGIN,
            price_scale_bottom_margin: DEFAULT_PRICE_SCALE_MARGIN,
            price_scale_locked: false,
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

    /// Indica si el eje de precios sigue automáticamente los extremos visibles.
    pub fn price_scale_auto(&self) -> bool {
        self.price_scale_auto
    }

    /// Devuelve el criterio usado por la autoescala vertical.
    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.price_scale_mode
    }

    /// Cambia el criterio de precios que determina el rango de autoescala.
    ///
    /// El siguiente cálculo de autoescala aplicará el nuevo criterio. En modo
    /// manual se conserva la referencia actual hasta volver a activar Auto.
    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) {
        self.price_scale_mode = mode;
    }

    /// Devuelve la transformación usada por el eje vertical.
    pub fn price_scale_type(&self) -> PriceScaleType {
        self.price_scale_type
    }

    /// Devuelve el precio base utilizado por la escala porcentual.
    pub fn price_scale_percentage_base(&self) -> Option<f64> {
        self.price_scale_percentage_base
    }

    /// Actualiza el precio base porcentual. Los valores deben ser positivos y finitos.
    ///
    /// Si el modo porcentual está activo, la referencia se recalculará con los
    /// datos visibles en el siguiente frame.
    pub fn set_price_scale_percentage_base(&mut self, base: f64) -> bool {
        if !base.is_finite() || base <= 0.0 {
            return false;
        }
        if self.price_scale_percentage_base == Some(base) {
            return true;
        }

        self.price_scale_percentage_base = Some(base);
        if self.price_scale_type == PriceScaleType::Percentage {
            self.price_scale_auto = true;
            self.zoom_y = self.auto_price_scale_zoom();
            self.offset_y = 0.0;
        }
        true
    }

    /// Cambia la transformación del eje vertical.
    ///
    /// El cambio vuelve a Auto para recalcular el rango en las nuevas unidades.
    /// El modo porcentual requiere que exista un precio base válido.
    pub fn set_price_scale_type(&mut self, scale_type: PriceScaleType) -> bool {
        if scale_type == self.price_scale_type {
            return true;
        }
        if scale_type == PriceScaleType::Percentage && self.price_scale_percentage_base.is_none() {
            return false;
        }

        self.price_scale_type = scale_type;
        self.price_scale_auto = true;
        self.zoom_y = self.auto_price_scale_zoom();
        self.offset_y = 0.0;
        true
    }

    /// Convierte un precio de mercado al espacio numérico del eje.
    pub fn price_to_scale_value(&self, price: f64) -> Option<f64> {
        self.price_scale_type
            .to_scale_value(price, self.price_scale_percentage_base)
    }

    /// Convierte un valor del eje al precio de mercado correspondiente.
    pub fn scale_value_to_price(&self, value: f64) -> Option<f64> {
        self.price_scale_type
            .from_scale_value(value, self.price_scale_percentage_base)
    }

    /// Convierte una coordenada vertical a precio usando la referencia de escala actual.
    pub fn screen_y_to_price_value(
        &self,
        screen_y: f64,
        screen_height: f64,
        price_range: f64,
        center_scale_value: f64,
    ) -> Option<f64> {
        if !screen_height.is_finite()
            || screen_height <= 0.0
            || !price_range.is_finite()
            || price_range <= 0.0
            || !center_scale_value.is_finite()
        {
            return None;
        }

        let scale_value = transforms::screen_y_to_price(
            screen_y,
            screen_height,
            price_range,
            self.zoom_y,
            self.offset_y,
            center_scale_value,
        );
        self.scale_value_to_price(scale_value)
    }

    /// Márgenes superior e inferior de autoescala como fracción del alto del plot.
    ///
    /// Por ejemplo, (0.05, 0.10) reserva un 5% arriba y un 10% abajo.
    pub fn price_scale_margins(&self) -> (f64, f64) {
        (self.price_scale_top_margin, self.price_scale_bottom_margin)
    }

    /// Configura los márgenes verticales de autoescala.
    ///
    /// Cada margen debe estar entre 0% y 45%, y su suma no puede superar 90%.
    /// En modo Auto actualiza el zoom previsto; la referencia del precio se
    /// recalcula con los datos visibles en el siguiente frame. En modo Manual
    /// solo guarda la configuración para la próxima activación de Auto.
    ///
    /// Devuelve false y conserva el estado si los valores no son válidos.
    pub fn set_price_scale_margins(&mut self, top: f64, bottom: f64) -> bool {
        if !top.is_finite()
            || !bottom.is_finite()
            || !(0.0..=MAX_PRICE_SCALE_MARGIN).contains(&top)
            || !(0.0..=MAX_PRICE_SCALE_MARGIN).contains(&bottom)
            || top + bottom > 1.0 - 2.0 * DEFAULT_PRICE_SCALE_MARGIN
        {
            return false;
        }

        self.price_scale_top_margin = top;
        self.price_scale_bottom_margin = bottom;
        if self.price_scale_auto {
            self.zoom_y = self.auto_price_scale_zoom();
            self.offset_y = 0.0;
        }
        true
    }

    /// Restablece los márgenes verticales predeterminados (5% arriba y abajo).
    pub fn reset_price_scale_margins(&mut self) {
        let _ =
            self.set_price_scale_margins(DEFAULT_PRICE_SCALE_MARGIN, DEFAULT_PRICE_SCALE_MARGIN);
    }

    /// Zoom vertical disponible después de reservar los márgenes configurados.
    fn auto_price_scale_zoom(&self) -> f64 {
        (1.0 - self.price_scale_top_margin - self.price_scale_bottom_margin)
            .clamp(self.min_zoom, self.max_zoom)
    }

    /// Desplaza el centro de referencia para respetar márgenes asimétricos.
    fn auto_price_scale_center(&self, midpoint: f64, range: f64, zoom: f64) -> f64 {
        midpoint
            + (self.price_scale_top_margin - self.price_scale_bottom_margin) * range / (2.0 * zoom)
    }

    /// Activa o desactiva la autoescala vertical.
    ///
    /// Al activarla se restablecen zoom y desplazamiento vertical; el rango
    /// de referencia se actualiza con los datos visibles en el siguiente frame.
    pub fn set_price_scale_auto(&mut self, enabled: bool) {
        self.price_scale_auto = enabled;
        if enabled {
            self.zoom_y = self.auto_price_scale_zoom();
            self.offset_y = 0.0;
        }
    }

    /// Indica si el usuario bloqueó las interacciones de la escala de precios.
    pub fn price_scale_locked(&self) -> bool {
        self.price_scale_locked
    }

    /// Bloquea o desbloquea el desplazamiento y zoom vertical del chart.
    pub fn set_price_scale_locked(&mut self, locked: bool) {
        self.price_scale_locked = locked;
    }

    /// Rango y centro de referencia en las unidades transformadas del eje vertical.
    pub fn price_scale_reference(&self) -> (f64, f64) {
        (self.price_scale_range, self.price_scale_center)
    }

    /// Actualiza la referencia vertical a partir de los extremos visibles.
    ///
    /// Solo actúa en modo automático. En modo manual el rango queda congelado,
    /// incluso cuando se agregan nuevas velas o cambia el historial realtime.
    /// Devuelve true si la escala visual cambió.
    pub fn update_auto_price_scale(&mut self, min_price: f64, max_price: f64) -> bool {
        if !self.price_scale_auto
            || !min_price.is_finite()
            || !max_price.is_finite()
            || max_price < min_price
        {
            return false;
        }

        let Some(min_scale_value) = self.price_to_scale_value(min_price) else {
            return false;
        };
        let Some(max_scale_value) = self.price_to_scale_value(max_price) else {
            return false;
        };
        let midpoint = min_scale_value + (max_scale_value - min_scale_value) / 2.0;
        let raw_range = max_scale_value - min_scale_value;
        if !midpoint.is_finite() || !raw_range.is_finite() {
            return false;
        }

        // Una serie plana necesita un rango no degenerado en el espacio del eje.
        let range = if raw_range > f64::EPSILON {
            raw_range
        } else {
            (midpoint.abs() * 0.002).max(1e-6)
        };
        let zoom_y = self.auto_price_scale_zoom();
        let center = self.auto_price_scale_center(midpoint, range, zoom_y);

        let changed = self.price_scale_range != range
            || self.price_scale_center != center
            || self.zoom_y != zoom_y
            || self.offset_y != 0.0;
        if changed {
            self.price_scale_range = range;
            self.price_scale_center = center;
            self.zoom_y = zoom_y;
            self.offset_y = 0.0;
        }
        changed
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
        let margin = screen_width * HORIZONTAL_PAN_MARGIN_RATIO;

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
            || (delta_x == 0.0 && delta_y == 0.0)
            || !screen_width.is_finite()
            || !screen_height.is_finite()
            || screen_width <= 0.0
            || screen_height <= 0.0
            || !center_x.is_finite()
            || !center_y.is_finite()
        {
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

        self.zoom_to_at(
            self.zoom_x * factor_x,
            self.zoom_y * factor_y,
            center_x,
            center_y,
            screen_width,
            screen_height,
        );
    }

    /// Establece ambos niveles de zoom preservando el punto bajo el cursor.
    ///
    /// center_x y center_y son fracciones normalizadas del área de plot.
    /// Centraliza la transformación de offsets para que los distintos tipos
    /// de gesto utilicen exactamente la misma fórmula.
    ///
    /// Devuelve true si el viewport cambió.
    pub fn zoom_to_at(
        &mut self,
        zoom_x: f64,
        zoom_y: f64,
        center_x: f64,
        center_y: f64,
        screen_width: f64,
        screen_height: f64,
    ) -> bool {
        if !zoom_x.is_finite()
            || !zoom_y.is_finite()
            || zoom_x <= 0.0
            || zoom_y <= 0.0
            || !center_x.is_finite()
            || !center_y.is_finite()
            || !screen_width.is_finite()
            || !screen_height.is_finite()
            || screen_width <= 0.0
            || screen_height <= 0.0
        {
            return false;
        }

        let new_zoom_x = zoom_x.clamp(self.min_zoom, self.max_zoom);
        let new_zoom_y = zoom_y.clamp(self.min_zoom, self.max_zoom);
        if new_zoom_x == self.zoom_x && new_zoom_y == self.zoom_y {
            return false;
        }

        let center_x_px = center_x.clamp(0.0, 1.0) * screen_width;
        let center_y_px = center_y.clamp(0.0, 1.0) * screen_height;
        // Los offsets se expresan respecto del centro del plot. Para que
        // el dato bajo el cursor no se mueva, hay que usar el desplazamiento
        // del cursor respecto del centro, no la coordenada desde el borde.
        let cursor_from_center_x = center_x_px - screen_width / 2.0;
        let cursor_from_center_y = center_y_px - screen_height / 2.0;

        self.offset_x += (cursor_from_center_x / transforms::CANDLE_SPACING_BASE)
            * (1.0 / new_zoom_x - 1.0 / self.zoom_x);
        self.offset_y +=
            (cursor_from_center_y / screen_height) * (1.0 / new_zoom_y - 1.0 / self.zoom_y);
        self.zoom_x = new_zoom_x;
        self.zoom_y = new_zoom_y;

        tracing::debug!(
            "Zoom: x={:.2}, y={:.2}, offset=({:.2}, {:.2})",
            self.zoom_x,
            self.zoom_y,
            self.offset_x,
            self.offset_y
        );
        true
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

    /// Ajusta automáticamente el zoom y el offset vertical para que el rango
    /// de precios quepa en el área de dibujo con los márgenes configurados.
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

        let (Some(min_scale_value), Some(max_scale_value)) = (
            self.price_to_scale_value(min_price_f64),
            self.price_to_scale_value(max_price_f64),
        ) else {
            return;
        };
        let Some(price_range_value) =
            transforms::validate_price_range(min_scale_value, max_scale_value)
        else {
            return;
        };

        let new_zoom_y = self.auto_price_scale_zoom();
        let midpoint = min_scale_value + price_range_value / 2.0;

        self.price_scale_range = price_range_value;
        self.price_scale_center =
            self.auto_price_scale_center(midpoint, price_range_value, new_zoom_y);
        self.price_scale_auto = true;
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
        self.price_scale_top_margin = DEFAULT_PRICE_SCALE_MARGIN;
        self.price_scale_bottom_margin = DEFAULT_PRICE_SCALE_MARGIN;
        self.zoom_y = self.auto_price_scale_zoom();
        self.offset_x = 0.0;
        self.offset_y = 0.0;
        self.price_scale_range = 1.0;
        self.price_scale_center = 0.0;
        self.price_scale_auto = true;
        self.price_scale_mode = PriceScaleMode::HighLow;
        self.price_scale_type = PriceScaleType::Linear;
        self.price_scale_percentage_base = None;
        self.price_scale_locked = false;
        tracing::debug!("Viewport restablecido a valores por defecto");
    }

    /// Convierte un índice de vela (coordenada de datos X) a coordenada de pantalla.
    ///
    /// `index` es el índice de la vela en el array (basado en 0).
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve la coordenada X en píxeles de pantalla. Usa `zoom_x` (eje X).
    pub fn data_x_to_screen(&self, index: usize, screen_width: f64) -> f64 {
        self.logical_to_screen(index as f64, screen_width)
    }

    /// Convierte una coordenada lógica (posiblemente fraccionaria) a píxeles.
    pub fn logical_to_screen(&self, logical_index: f64, screen_width: f64) -> f64 {
        if !logical_index.is_finite() || !screen_width.is_finite() || screen_width <= 0.0 {
            return screen_width / 2.0;
        }
        screen_width / 2.0 + (logical_index + self.offset_x) * self.candle_spacing()
    }

    /// Convierte una coordenada de pantalla a una coordenada lógica fraccionaria.
    pub fn screen_to_logical(&self, screen_x: f64, screen_width: f64) -> f64 {
        if !screen_x.is_finite() || !screen_width.is_finite() || screen_width <= 0.0 {
            return 0.0;
        }
        (screen_x - screen_width / 2.0) / self.candle_spacing() - self.offset_x
    }

    /// Ajusta el eje temporal para mostrar todo el historial con un margen
    /// horizontal del 5%. El eje de precios no se modifica.
    pub fn fit_content(&mut self, logical_count: usize, screen_width: f64) -> bool {
        if logical_count == 0 {
            return false;
        }

        self.fit_logical_range(0.0, (logical_count - 1) as f64, screen_width)
    }

    /// Centra una posición lógica (también fraccionaria) en el área visible.
    ///
    /// Devuelve `true` si el offset horizontal cambió.
    pub fn center_on_logical(&mut self, logical_index: f64) -> bool {
        if !logical_index.is_finite() {
            return false;
        }

        let next_offset = -logical_index;
        if self.offset_x == next_offset {
            return false;
        }
        self.offset_x = next_offset;
        true
    }

    /// Ajusta el zoom horizontal para mostrar un intervalo lógico completo
    /// con un margen del 5% a cada lado.
    ///
    /// Los extremos pueden ser fraccionarios, por ejemplo al navegar a un
    /// intervalo temporal cuyos límites caen entre dos velas.
    /// El eje vertical no se modifica.
    pub fn fit_logical_range(
        &mut self,
        first_logical: f64,
        last_logical: f64,
        screen_width: f64,
    ) -> bool {
        if !first_logical.is_finite()
            || !last_logical.is_finite()
            || first_logical < 0.0
            || last_logical < first_logical
            || !screen_width.is_finite()
            || screen_width <= 0.0
        {
            return false;
        }

        let logical_span = last_logical - first_logical;
        let desired_spacing = if logical_span <= f64::EPSILON {
            transforms::CANDLE_SPACING_BASE
        } else {
            screen_width * 0.90 / logical_span
        };
        let desired_zoom = desired_spacing / transforms::CANDLE_SPACING_BASE;
        let center = first_logical + logical_span / 2.0;
        let previous = (self.zoom_x, self.offset_x);

        self.set_zoom_x(desired_zoom);
        self.set_offset_x(-center);

        previous != (self.zoom_x, self.offset_x)
    }

    /// Convierte una coordenada de pantalla X a un índice de vela (coordenada de datos X).
    ///
    /// `screen_x` es la posición en píxeles de pantalla.
    /// `screen_width` es el ancho del área de dibujo en píxeles.
    ///
    /// Devuelve el índice de vela más cercano (basado en 0), clampado a
    /// `[0, candle_count - 1]`. Usa `zoom_x` (eje X).
    pub fn screen_x_to_data(&self, screen_x: f64, screen_width: f64, candle_count: usize) -> usize {
        if candle_count == 0 {
            return 0;
        }
        self.screen_to_logical(screen_x, screen_width)
            .round()
            .clamp(0.0, candle_count.saturating_sub(1) as f64) as usize
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

        let min_price = price_range
            .min()
            .value()
            .to_f64()
            .expect("precio mínimo convertible");
        let max_price = price_range
            .max()
            .value()
            .to_f64()
            .expect("precio máximo convertible");
        let (Some(min_value), Some(max_value)) = (
            self.price_to_scale_value(min_price),
            self.price_to_scale_value(max_price),
        ) else {
            return center_y;
        };
        let Some(range_value) = transforms::validate_price_range(min_value, max_value) else {
            return center_y;
        };
        let price_value = price.value().to_f64().expect("precio convertible");
        let center_value = center_price
            .value()
            .to_f64()
            .expect("precio central convertible");
        let (Some(price_scale_value), Some(center_scale_value)) = (
            self.price_to_scale_value(price_value),
            self.price_to_scale_value(center_value),
        ) else {
            return center_y;
        };

        let pixels_per_unit = screen_height / range_value * self.zoom_y;
        let relative_price = price_scale_value - center_scale_value - self.offset_y * range_value;
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
        let min_price = price_range
            .min()
            .value()
            .to_f64()
            .expect("precio mínimo convertible");
        let max_price = price_range
            .max()
            .value()
            .to_f64()
            .expect("precio máximo convertible");
        let (Some(min_value), Some(max_value)) = (
            self.price_to_scale_value(min_price),
            self.price_to_scale_value(max_price),
        ) else {
            return center_price;
        };
        let Some(range_value) = transforms::validate_price_range(min_value, max_value) else {
            return center_price;
        };
        let center_price_value = center_price
            .value()
            .to_f64()
            .expect("precio central convertible");
        let Some(center_scale_value) = self.price_to_scale_value(center_price_value) else {
            return center_price;
        };
        let pixels_per_unit = screen_height / range_value * self.zoom_y;
        let relative_price = (center_y - screen_y) / pixels_per_unit;
        let scale_value = center_scale_value + self.offset_y * range_value + relative_price;
        let Some(price_value) = self.scale_value_to_price(scale_value) else {
            return center_price;
        };

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

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_price_scale_tracks_the_visible_price_range() {
        let mut viewport = Viewport::new();

        assert!(viewport.update_auto_price_scale(90.0, 110.0));
        assert_eq!(viewport.price_scale_reference(), (20.0, 100.0));
        assert!((viewport.zoom_y() - 0.9).abs() < 1e-12);
        assert_eq!(viewport.offset_y(), 0.0);

        assert!(viewport.update_auto_price_scale(80.0, 120.0));
        assert_eq!(viewport.price_scale_reference(), (40.0, 100.0));
    }

    #[test]
    fn price_scale_mode_defaults_to_high_low_and_can_select_close() {
        let mut viewport = Viewport::new();

        assert_eq!(viewport.price_scale_mode(), PriceScaleMode::HighLow);

        viewport.set_price_scale_mode(PriceScaleMode::Close);
        assert_eq!(viewport.price_scale_mode(), PriceScaleMode::Close);

        viewport.reset();
        assert_eq!(viewport.price_scale_mode(), PriceScaleMode::HighLow);
    }

    #[test]
    fn price_scale_type_defaults_to_linear_and_resets_to_linear() {
        let mut viewport = Viewport::new();
        assert_eq!(viewport.price_scale_type(), PriceScaleType::Linear);

        assert!(viewport.set_price_scale_percentage_base(100.0));
        assert!(viewport.set_price_scale_type(PriceScaleType::Percentage));
        assert_eq!(viewport.price_scale_type(), PriceScaleType::Percentage);

        viewport.reset();
        assert_eq!(viewport.price_scale_type(), PriceScaleType::Linear);
        assert_eq!(viewport.price_scale_percentage_base(), None);
    }

    #[test]
    fn logarithmic_scale_round_trips_positive_prices_and_rejects_non_positive_prices() {
        let scale = PriceScaleType::Logarithmic;
        for price in [0.01, 1.0, 50.0, 50_000.0, 1_000_000.0] {
            let value = scale.to_scale_value(price, None).expect("precio positivo");
            let restored = scale.from_scale_value(value, None).expect("inversa válida");
            assert!((restored - price).abs() <= price.abs() * 1e-12);
        }

        assert_eq!(scale.to_scale_value(0.0, None), None);
        assert_eq!(scale.to_scale_value(-1.0, None), None);
    }

    #[test]
    fn percentage_scale_uses_configured_base_and_round_trips_prices() {
        let scale = PriceScaleType::Percentage;
        assert!((scale.to_scale_value(100.0, Some(100.0)).unwrap()).abs() < 1e-12);
        assert!((scale.to_scale_value(110.0, Some(100.0)).unwrap() - 10.0).abs() < 1e-12);
        assert!((scale.from_scale_value(10.0, Some(100.0)).unwrap() - 110.0).abs() < 1e-12);
        assert_eq!(scale.to_scale_value(110.0, None), None);
        assert_eq!(scale.from_scale_value(10.0, Some(0.0)), None);
    }

    #[test]
    fn auto_price_scale_stores_values_in_the_selected_transformed_space() {
        let mut logarithmic = Viewport::new();
        assert!(logarithmic.set_price_scale_type(PriceScaleType::Logarithmic));
        assert!(logarithmic.update_auto_price_scale(10.0, 100.0));
        let (log_range, log_center) = logarithmic.price_scale_reference();
        assert!((log_range - 10.0_f64.ln()).abs() < 1e-12);
        assert!((log_center - 10.0_f64.ln() - 10.0_f64.ln() / 2.0).abs() < 1e-12);

        let mut percentage = Viewport::new();
        assert!(percentage.set_price_scale_percentage_base(100.0));
        assert!(percentage.set_price_scale_type(PriceScaleType::Percentage));
        assert!(percentage.update_auto_price_scale(90.0, 110.0));
        let (percentage_range, percentage_center) = percentage.price_scale_reference();
        assert!((percentage_range - 20.0).abs() < 1e-12);
        assert!(percentage_center.abs() < 1e-12);
    }

    #[test]
    fn logarithmic_auto_scale_rejects_ranges_with_non_positive_prices() {
        let mut viewport = Viewport::new();
        assert!(viewport.set_price_scale_type(PriceScaleType::Logarithmic));
        let reference = viewport.price_scale_reference();

        assert!(!viewport.update_auto_price_scale(-10.0, 100.0));
        assert_eq!(viewport.price_scale_reference(), reference);
    }

    #[test]
    fn enabling_auto_price_scale_restores_the_configured_vertical_margins() {
        let mut viewport = Viewport::new();
        viewport.set_zoom(2.5, 3.5);
        viewport.set_offset_y(0.75);
        assert!(viewport.set_price_scale_margins(0.10, 0.05));

        viewport.set_price_scale_auto(true);

        assert!(viewport.price_scale_auto());
        assert!((viewport.zoom_y() - 0.85).abs() < f64::EPSILON);
        assert_eq!(viewport.offset_y(), 0.0);
    }

    #[test]
    fn asymmetric_price_scale_margins_place_visible_extremes_at_configured_edges() {
        let mut viewport = Viewport::new();
        assert!(viewport.set_price_scale_margins(0.10, 0.05));
        assert!(viewport.update_auto_price_scale(90.0, 110.0));

        let (range, center) = viewport.price_scale_reference();
        let top_y = transforms::price_to_screen(
            110.0,
            1_000.0,
            range,
            viewport.zoom_y(),
            viewport.offset_y(),
            center,
        );
        let bottom_y = transforms::price_to_screen(
            90.0,
            1_000.0,
            range,
            viewport.zoom_y(),
            viewport.offset_y(),
            center,
        );

        assert!((top_y - 100.0).abs() < 1e-9);
        assert!((bottom_y - 950.0).abs() < 1e-9);
    }

    #[test]
    fn invalid_price_scale_margins_are_rejected_without_changing_the_configuration() {
        let mut viewport = Viewport::new();
        let defaults = viewport.price_scale_margins();

        for (top, bottom) in [
            (-0.01, 0.05),
            (0.05, -0.01),
            (0.46, 0.05),
            (0.05, 0.46),
            (0.45, 0.46),
            (f64::NAN, 0.05),
            (0.05, f64::INFINITY),
        ] {
            assert!(!viewport.set_price_scale_margins(top, bottom));
            assert_eq!(viewport.price_scale_margins(), defaults);
        }
    }

    #[test]
    fn manual_price_scale_keeps_its_reference_when_margins_change() {
        let mut viewport = Viewport::new();
        assert!(viewport.update_auto_price_scale(90.0, 110.0));
        viewport.set_price_scale_auto(false);
        let reference = viewport.price_scale_reference();
        let zoom = viewport.zoom_y();

        assert!(viewport.set_price_scale_margins(0.15, 0.10));

        assert_eq!(viewport.price_scale_reference(), reference);
        assert_eq!(viewport.zoom_y(), zoom);
        assert_eq!(viewport.price_scale_margins(), (0.15, 0.10));
    }

    #[test]
    fn reset_restores_default_price_scale_margins() {
        let mut viewport = Viewport::new();
        assert!(viewport.set_price_scale_margins(0.20, 0.10));

        viewport.reset();

        assert_eq!(viewport.price_scale_margins(), (0.05, 0.05));
        assert!((viewport.zoom_y() - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn manual_price_scale_keeps_its_reference_when_new_data_arrives() {
        let mut viewport = Viewport::new();
        viewport.update_auto_price_scale(90.0, 110.0);
        viewport.set_price_scale_auto(false);

        assert!(!viewport.update_auto_price_scale(70.0, 140.0));
        assert_eq!(viewport.price_scale_reference(), (20.0, 100.0));
        assert!(!viewport.price_scale_auto());
    }

    #[test]
    fn flat_price_ranges_receive_a_safe_nonzero_scale() {
        let mut viewport = Viewport::new();

        assert!(viewport.update_auto_price_scale(50_000.0, 50_000.0));
        let (range, center) = viewport.price_scale_reference();
        assert!(range.is_finite() && range > 0.0);
        assert_eq!(center, 50_000.0);
    }

    #[test]
    fn price_scale_lock_is_independent_from_auto_mode() {
        let mut viewport = Viewport::new();
        viewport.set_price_scale_locked(true);
        assert!(viewport.price_scale_locked());
        assert!(viewport.price_scale_auto());

        viewport.set_price_scale_auto(false);
        assert!(viewport.price_scale_locked());
        assert!(!viewport.price_scale_auto());

        viewport.set_price_scale_locked(false);
        assert!(!viewport.price_scale_locked());
    }

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
    #[test]
    fn zoom_to_at_preserves_the_data_point_under_the_cursor() {
        let mut viewport = Viewport::new();
        viewport.set_zoom(1.5, 1.25);
        viewport.set_offset_x(8.0);
        viewport.set_offset_y(-0.35);

        let width = 1200.0;
        let height = 700.0;
        let x_px = width * 0.37;
        let y_px = height * 0.63;
        let data_x_before = (x_px - width / 2.0) / viewport.candle_spacing() - viewport.offset_x();
        let price_before = transforms::screen_y_to_price(
            y_px,
            height,
            1000.0,
            viewport.zoom_y(),
            viewport.offset_y(),
            50000.0,
        );

        assert!(viewport.zoom_to_at(2.2, 1.8, 0.37, 0.63, width, height));

        let data_x_after = (x_px - width / 2.0) / viewport.candle_spacing() - viewport.offset_x();
        let price_after = transforms::screen_y_to_price(
            y_px,
            height,
            1000.0,
            viewport.zoom_y(),
            viewport.offset_y(),
            50000.0,
        );

        assert!((data_x_before - data_x_after).abs() < 1e-9);
        assert!((price_before - price_after).abs() < 1e-9);
    }

    #[test]
    fn zoom_to_at_rejects_invalid_dimensions_and_zoom() {
        let mut viewport = Viewport::new();
        let before = viewport;
        assert!(!viewport.zoom_to_at(2.0, 2.0, 0.5, 0.5, 0.0, 100.0));
        assert!(!viewport.zoom_to_at(f64::NAN, 2.0, 0.5, 0.5, 100.0, 100.0));
        assert_eq!(viewport, before);
    }
    #[test]
    fn logical_screen_transforms_are_inverse() {
        let mut viewport = Viewport::new();
        viewport.set_zoom_x(2.4);
        viewport.set_offset_x(-3.25);
        let width = 1_280.0;
        for logical in [0.0, 0.5, 4.0, 19.75, 100.0] {
            let screen = viewport.logical_to_screen(logical, width);
            let restored = viewport.screen_to_logical(screen, width);
            assert!((restored - logical).abs() < 1e-10);
        }
    }

    #[test]
    fn fit_content_centers_the_full_logical_range() {
        let mut viewport = Viewport::new();
        assert!(viewport.fit_content(100, 1_000.0));
        let first = viewport.logical_to_screen(0.0, 1_000.0);
        let last = viewport.logical_to_screen(99.0, 1_000.0);
        assert!((first - 50.0).abs() < 1e-8);
        assert!((last - 950.0).abs() < 1e-8);
    }

    #[test]
    fn fit_logical_range_places_both_endpoints_inside_plot_margins() {
        let mut viewport = Viewport::new();
        assert!(viewport.fit_logical_range(10.5, 20.5, 1_000.0));

        let first = viewport.logical_to_screen(10.5, 1_000.0);
        let last = viewport.logical_to_screen(20.5, 1_000.0);
        assert!((first - 50.0).abs() < 1e-8);
        assert!((last - 950.0).abs() < 1e-8);
    }

    #[test]
    fn fit_logical_range_changes_only_the_horizontal_axis() {
        let mut viewport = Viewport::new();
        viewport.set_zoom_y(3.25);
        viewport.set_offset_y(-0.75);

        assert!(viewport.fit_logical_range(5.0, 15.0, 1_000.0));

        assert_eq!(viewport.zoom_y(), 3.25);
        assert_eq!(viewport.offset_y(), -0.75);
    }

    #[test]
    fn center_on_logical_places_target_at_plot_center() {
        let mut viewport = Viewport::new();
        viewport.set_zoom_x(2.0);
        assert!(viewport.center_on_logical(42.5));
        assert!((viewport.logical_to_screen(42.5, 800.0) - 400.0).abs() < 1e-8);
        assert!(!viewport.center_on_logical(42.5));

        let before = viewport;
        assert!(!viewport.center_on_logical(f64::INFINITY));
        assert_eq!(viewport, before);
    }

    #[test]
    fn fit_logical_range_rejects_invalid_ranges_without_mutating_viewport() {
        let mut viewport = Viewport::new();
        let before = viewport;
        assert!(!viewport.fit_logical_range(2.0, 1.0, 1_000.0));
        assert!(!viewport.fit_logical_range(-1.0, 1.0, 1_000.0));
        assert!(!viewport.fit_logical_range(0.0, f64::NAN, 1_000.0));
        assert!(!viewport.fit_logical_range(0.0, 1.0, 0.0));
        assert_eq!(viewport, before);
    }

    #[test]
    fn fit_content_ignores_empty_or_invalid_dimensions() {
        let mut viewport = Viewport::new();
        let before = viewport;
        assert!(!viewport.fit_content(0, 1_000.0));
        assert!(!viewport.fit_content(10, f64::NAN));
        assert_eq!(viewport, before);
    }
}
