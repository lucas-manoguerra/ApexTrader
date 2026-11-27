use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use crate::domain::transforms;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

const HORIZONTAL_PAN_MARGIN_RATIO: f64 = 0.05;

const DEFAULT_PRICE_SCALE_MARGIN: f64 = 0.05;

const MAX_PRICE_SCALE_MARGIN: f64 = 0.45;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriceScaleMode {
    #[default]
    HighLow,

    Close,
}

impl PriceScaleMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::HighLow => "Máximo/Mínimo (OHLC)",
            Self::Close => "Solo cierres",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PriceScaleType {
    #[default]
    Linear,

    Logarithmic,

    Percentage,
}

impl PriceScaleType {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Linear => "Lineal",
            Self::Logarithmic => "Logarítmica",
            Self::Percentage => "Porcentual (%)",
        }
    }

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
    zoom_x: f64,
    zoom_y: f64,
    offset_x: f64,
    offset_y: f64,
    zoom_step: f64,
    min_zoom: f64,
    max_zoom: f64,
    price_scale_range: f64,
    price_scale_center: f64,
    price_scale_auto: bool,
    price_scale_mode: PriceScaleMode,
    price_scale_type: PriceScaleType,
    price_scale_percentage_base: Option<f64>,
    price_scale_top_margin: f64,
    price_scale_bottom_margin: f64,
    price_scale_locked: bool,
    price_scale_inverted: bool,
}

impl Viewport {
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
            price_scale_inverted: false,
        }
    }

    pub fn zoom_x(&self) -> f64 {
        self.zoom_x
    }

    pub fn zoom_y(&self) -> f64 {
        self.zoom_y
    }

    pub fn price_scale_auto(&self) -> bool {
        self.price_scale_auto
    }

    pub fn price_scale_mode(&self) -> PriceScaleMode {
        self.price_scale_mode
    }

    pub fn set_price_scale_mode(&mut self, mode: PriceScaleMode) {
        self.price_scale_mode = mode;
    }

    pub fn price_scale_type(&self) -> PriceScaleType {
        self.price_scale_type
    }

    pub fn price_scale_percentage_base(&self) -> Option<f64> {
        self.price_scale_percentage_base
    }

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

    pub fn price_to_scale_value(&self, price: f64) -> Option<f64> {
        self.price_scale_type
            .to_scale_value(price, self.price_scale_percentage_base)
    }

    pub fn scale_value_to_price(&self, value: f64) -> Option<f64> {
        self.price_scale_type
            .from_scale_value(value, self.price_scale_percentage_base)
    }

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
            if self.price_scale_inverted {
                screen_height - screen_y
            } else {
                screen_y
            },
            screen_height,
            price_range,
            self.zoom_y,
            self.offset_y,
            center_scale_value,
        );
        self.scale_value_to_price(scale_value)
    }

    pub fn price_scale_margins(&self) -> (f64, f64) {
        (self.price_scale_top_margin, self.price_scale_bottom_margin)
    }

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

    pub fn reset_price_scale_margins(&mut self) {
        let _ =
            self.set_price_scale_margins(DEFAULT_PRICE_SCALE_MARGIN, DEFAULT_PRICE_SCALE_MARGIN);
    }

    fn auto_price_scale_zoom(&self) -> f64 {
        (1.0 - self.price_scale_top_margin - self.price_scale_bottom_margin)
            .clamp(self.min_zoom, self.max_zoom)
    }

    fn auto_price_scale_center(&self, midpoint: f64, range: f64, zoom: f64) -> f64 {
        midpoint
            + (self.price_scale_top_margin - self.price_scale_bottom_margin) * range / (2.0 * zoom)
    }

    pub fn set_price_scale_auto(&mut self, enabled: bool) {
        self.price_scale_auto = enabled;
        if enabled {
            self.zoom_y = self.auto_price_scale_zoom();
            self.offset_y = 0.0;
        }
    }

    pub fn price_scale_locked(&self) -> bool {
        self.price_scale_locked
    }

    pub fn set_price_scale_locked(&mut self, locked: bool) {
        self.price_scale_locked = locked;
    }

    pub fn price_scale_inverted(&self) -> bool {
        self.price_scale_inverted
    }

    pub fn set_price_scale_inverted(&mut self, inverted: bool) {
        self.price_scale_inverted = inverted;
    }

    pub fn price_scale_reference(&self) -> (f64, f64) {
        (self.price_scale_range, self.price_scale_center)
    }

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

    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }

    pub fn offset_y(&self) -> f64 {
        self.offset_y
    }

    pub fn zoom_step(&self) -> f64 {
        self.zoom_step
    }

    pub fn min_zoom(&self) -> f64 {
        self.min_zoom
    }

    pub fn max_zoom(&self) -> f64 {
        self.max_zoom
    }

    pub fn candle_spacing(&self) -> f64 {
        transforms::candle_spacing(self.zoom_x)
    }

    pub fn set_min_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.min_zoom = value;
        if self.min_zoom > self.max_zoom {
            self.max_zoom = self.min_zoom;
        }

        self.zoom_x = self.zoom_x.clamp(self.min_zoom, self.max_zoom);
        self.zoom_y = self.zoom_y.clamp(self.min_zoom, self.max_zoom);
    }

    pub fn set_max_zoom(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.max_zoom = value;
        if self.max_zoom < self.min_zoom {
            self.min_zoom = self.max_zoom;
        }

        self.zoom_x = self.zoom_x.clamp(self.min_zoom, self.max_zoom);
        self.zoom_y = self.zoom_y.clamp(self.min_zoom, self.max_zoom);
    }

    pub fn set_zoom(&mut self, x: f64, y: f64) {
        self.set_zoom_x(x);
        self.set_zoom_y(y);
    }

    pub fn set_zoom_x(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.zoom_x = value.clamp(self.min_zoom, self.max_zoom);
    }

    pub fn set_zoom_y(&mut self, value: f64) {
        if !value.is_finite() || value <= 0.0 {
            return;
        }
        self.zoom_y = value.clamp(self.min_zoom, self.max_zoom);
    }

    pub fn set_offset_x(&mut self, value: f64) {
        if value.is_finite() {
            self.offset_x = value;
        }
    }

    pub fn set_offset_y(&mut self, value: f64) {
        if value.is_finite() {
            self.offset_y = value;
        }
    }

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

        let min_offset_x = (margin - center_x) / candle_spacing;

        let max_offset_x =
            (screen_width - margin - center_x - (candle_count as f64 - 1.0) * candle_spacing)
                / candle_spacing;

        if min_offset_x <= max_offset_x {
            self.offset_x = self.offset_x.clamp(min_offset_x, max_offset_x);
        }
    }

    pub fn clamp_zoom_to_candles(&mut self, candle_count: usize, screen_width: f64) {
        if candle_count == 0 || !screen_width.is_finite() || screen_width <= 0.0 {
            return;
        }

        let max_zoom_by_candle = 5.0;

        let min_allowed = self.min_zoom;
        let max_allowed = self.max_zoom.min(max_zoom_by_candle);

        if min_allowed <= max_allowed {
            self.zoom_x = self.zoom_x.clamp(min_allowed, max_allowed);
        }
    }

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

        let cursor_from_center_x = center_x_px - screen_width / 2.0;
        let cursor_from_center_y = center_y_px - screen_height / 2.0;

        self.offset_x += (cursor_from_center_x / transforms::CANDLE_SPACING_BASE)
            * (1.0 / new_zoom_x - 1.0 / self.zoom_x);
        let vertical_direction = if self.price_scale_inverted { -1.0 } else { 1.0 };
        self.offset_y += vertical_direction
            * (cursor_from_center_y / screen_height)
            * (1.0 / new_zoom_y - 1.0 / self.zoom_y);
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

    pub fn pan(&mut self, dx: f64, dy: f64, screen_height: f64) {
        if !dx.is_finite() || !dy.is_finite() || !screen_height.is_finite() || screen_height <= 0.0
        {
            return;
        }

        self.offset_x += dx / self.candle_spacing();
        let vertical_direction = if self.price_scale_inverted { 1.0 } else { -1.0 };
        self.offset_y += vertical_direction * dy / (screen_height * self.zoom_y);

        tracing::trace!(
            "Pan: offset=({:.2}, {:.2}), zoom=({:.2}, {:.2})",
            self.offset_x,
            self.offset_y,
            self.zoom_x,
            self.zoom_y
        );
    }

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
        self.price_scale_inverted = false;
        tracing::debug!("Viewport restablecido a valores por defecto");
    }

    pub fn data_x_to_screen(&self, index: usize, screen_width: f64) -> f64 {
        self.logical_to_screen(index as f64, screen_width)
    }

    pub fn logical_to_screen(&self, logical_index: f64, screen_width: f64) -> f64 {
        if !logical_index.is_finite() || !screen_width.is_finite() || screen_width <= 0.0 {
            return screen_width / 2.0;
        }
        screen_width / 2.0 + (logical_index + self.offset_x) * self.candle_spacing()
    }

    pub fn screen_to_logical(&self, screen_x: f64, screen_width: f64) -> f64 {
        if !screen_x.is_finite() || !screen_width.is_finite() || screen_width <= 0.0 {
            return 0.0;
        }
        (screen_x - screen_width / 2.0) / self.candle_spacing() - self.offset_x
    }

    pub fn fit_content(&mut self, logical_count: usize, screen_width: f64) -> bool {
        if logical_count == 0 {
            return false;
        }

        self.fit_logical_range(0.0, (logical_count - 1) as f64, screen_width)
    }

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

    pub fn screen_x_to_data(&self, screen_x: f64, screen_width: f64, candle_count: usize) -> usize {
        if candle_count == 0 {
            return 0;
        }
        self.screen_to_logical(screen_x, screen_width)
            .round()
            .clamp(0.0, candle_count.saturating_sub(1) as f64) as usize
    }

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
        let direction = if self.price_scale_inverted { -1.0 } else { 1.0 };
        center_y - relative_price * pixels_per_unit * direction
    }

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
        let direction = if self.price_scale_inverted { -1.0 } else { 1.0 };
        let relative_price = (center_y - screen_y) / (pixels_per_unit * direction);
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

    #[test]
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

#[cfg(test)]
mod price_scale_inversion_tests {
    use super::*;

    #[test]
    fn inverted_scale_reverses_screen_price_mapping() {
        let mut viewport = Viewport::new();
        assert!(viewport.update_auto_price_scale(90.0, 110.0));
        let (range, center) = viewport.price_scale_reference();

        let normal_top = viewport
            .screen_y_to_price_value(0.0, 200.0, range, center)
            .expect("precio superior válido");
        let normal_bottom = viewport
            .screen_y_to_price_value(200.0, 200.0, range, center)
            .expect("precio inferior válido");

        viewport.set_price_scale_inverted(true);

        let inverted_top = viewport
            .screen_y_to_price_value(0.0, 200.0, range, center)
            .expect("precio superior invertido válido");
        let inverted_bottom = viewport
            .screen_y_to_price_value(200.0, 200.0, range, center)
            .expect("precio inferior invertido válido");

        assert!(normal_top > normal_bottom);
        assert!(inverted_top < inverted_bottom);
        assert!((normal_top - inverted_bottom).abs() < 1e-10);
        assert!((normal_bottom - inverted_top).abs() < 1e-10);
    }

    #[test]
    fn reset_restores_normal_price_scale_orientation() {
        let mut viewport = Viewport::new();
        viewport.set_price_scale_inverted(true);

        viewport.reset();

        assert!(!viewport.price_scale_inverted());
    }
}
