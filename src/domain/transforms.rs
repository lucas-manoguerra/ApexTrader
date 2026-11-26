

















pub const CANDLE_SPACING_BASE: f64 = 20.0;


pub const CANDLE_BODY_WIDTH_RATIO: f64 = 0.6;


pub const MARGIN_RATIO: f64 = 0.05;




pub fn candle_spacing(zoom_level: f64) -> f64 {
    CANDLE_SPACING_BASE * zoom_level
}






pub fn candle_body_width(zoom_level: f64) -> f64 {
    candle_spacing(zoom_level) * CANDLE_BODY_WIDTH_RATIO
}


pub fn data_x_to_screen(index: usize, screen_width: f64, zoom_level: f64, offset_x: f64) -> f64 {
    let spacing = candle_spacing(zoom_level);
    let center_x = screen_width / 2.0;
    center_x + (index as f64 * spacing) + offset_x * spacing
}




pub fn screen_x_to_data(
    screen_x: f64,
    screen_width: f64,
    zoom_level: f64,
    offset_x: f64,
    candle_count: usize,
) -> usize {
    if candle_count == 0
        || !screen_x.is_finite()
        || !screen_width.is_finite()
        || screen_width <= 0.0
    {
        return 0;
    }

    let spacing = candle_spacing(zoom_level);
    if spacing <= 0.0 || !spacing.is_finite() {
        return 0;
    }

    let center_x = screen_width / 2.0;
    let index = (screen_x - center_x - offset_x * spacing) / spacing;
    let index = index.round();

    if index < 0.0 {
        return 0;
    }
    (index as usize).min(candle_count.saturating_sub(1))
}







pub fn price_to_screen(
    price: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    let center_y = screen_height / 2.0;
    let pixels_per_unit = screen_height / price_range * zoom_level;
    let relative_price = price - center_price - offset_y * price_range;
    center_y - relative_price * pixels_per_unit
}


pub fn screen_y_to_price(
    screen_y: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    let center_y = screen_height / 2.0;
    let pixels_per_unit = screen_height / price_range * zoom_level;
    let relative_price = (center_y - screen_y) / pixels_per_unit;
    center_price + offset_y * price_range + relative_price
}






pub fn validate_price_range(min: f64, max: f64) -> Option<f64> {
    if !min.is_finite() || !max.is_finite() {
        return None;
    }
    let range = max - min;
    if !range.is_finite() || range == 0.0 {
        return None;
    }
    Some(range)
}




#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}



#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartPoint {
    pub x: f64,
    pub y: f64,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub index: f64,
    pub price: f64,
}



pub fn screen_to_chart_point(screen: ScreenPoint, left: f64, top: f64) -> ChartPoint {
    ChartPoint {
        x: screen.x - left,
        y: screen.y - top,
    }
}



#[allow(clippy::too_many_arguments)]
pub fn chart_point_to_data_point(
    chart: ChartPoint,
    screen_width: f64,
    screen_height: f64,
    zoom_level: f64,
    offset_x: f64,
    offset_y: f64,
    price_range: f64,
    center_price: f64,
    candle_count: usize,
) -> DataPoint {
    let index = screen_x_to_data(chart.x, screen_width, zoom_level, offset_x, candle_count) as f64;
    let price = screen_y_to_price(
        chart.y,
        screen_height,
        price_range,
        zoom_level,
        offset_y,
        center_price,
    );
    DataPoint { index, price }
}









#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    pub first: usize,
    pub last: usize,
}

impl VisibleRange {




    pub fn empty() -> Self {
        Self { first: 1, last: 0 }
    }


    pub fn new(first: usize, last: usize) -> Self {
        Self { first, last }
    }


    pub fn is_empty(&self) -> bool {
        self.first > self.last
    }


    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else {
            self.last - self.first + 1
        }
    }
}

impl Default for VisibleRange {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_empty_is_empty() {
        let r = VisibleRange::empty();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert_eq!((r.first, r.last), (1, 0));
    }

    #[test]
    fn visible_range_default_is_empty() {
        let r = VisibleRange::default();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn visible_range_new_reports_length() {
        let r = VisibleRange::new(10, 20);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 11);
    }

    #[test]
    fn visible_range_single_candle() {
        let r = VisibleRange::new(5, 5);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn visible_range_zero_zero_is_not_empty() {
        let r = VisibleRange::new(0, 0);
        assert!(!r.is_empty());
        assert_eq!(r.len(), 1);
    }
}
