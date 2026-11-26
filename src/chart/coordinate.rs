








use crate::domain::transforms;


pub const CANDLE_SPACING_BASE: f64 = transforms::CANDLE_SPACING_BASE;


pub const CANDLE_BODY_WIDTH_RATIO: f64 = transforms::CANDLE_BODY_WIDTH_RATIO;


pub fn candle_spacing(zoom_level: f64) -> f64 {
    transforms::candle_spacing(zoom_level)
}


pub fn candle_body_width(zoom_level: f64) -> f64 {
    transforms::candle_body_width(zoom_level)
}


pub fn data_x_to_screen(index: usize, screen_width: f64, zoom_level: f64, offset_x: f64) -> f64 {
    transforms::data_x_to_screen(index, screen_width, zoom_level, offset_x)
}




pub fn screen_x_to_data(
    screen_x: f64,
    screen_width: f64,
    zoom_level: f64,
    offset_x: f64,
    candle_count: usize,
) -> usize {
    transforms::screen_x_to_data(screen_x, screen_width, zoom_level, offset_x, candle_count)
}




pub fn price_to_screen(
    price: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    transforms::price_to_screen(
        price,
        screen_height,
        price_range,
        zoom_level,
        offset_y,
        center_price,
    )
}


pub fn screen_y_to_price(
    screen_y: f64,
    screen_height: f64,
    price_range: f64,
    zoom_level: f64,
    offset_y: f64,
    center_price: f64,
) -> f64 {
    transforms::screen_y_to_price(
        screen_y,
        screen_height,
        price_range,
        zoom_level,
        offset_y,
        center_price,
    )
}


pub fn validate_price_range(min: f64, max: f64) -> Option<f64> {
    transforms::validate_price_range(min, max)
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



#[derive(Debug, Clone, Copy)]
pub struct CoordinateConverter {
    pub screen_width: f64,
    pub screen_height: f64,
    pub zoom_level: f64,
    pub offset_x: f64,
    pub offset_y: f64,
    pub price_range: f64,
    pub center_price: f64,
    pub candle_count: usize,
}

impl CoordinateConverter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        screen_width: f64,
        screen_height: f64,
        zoom_level: f64,
        offset_x: f64,
        offset_y: f64,
        price_range: f64,
        center_price: f64,
        candle_count: usize,
    ) -> Self {
        Self {
            screen_width,
            screen_height,
            zoom_level,
            offset_x,
            offset_y,
            price_range,
            center_price,
            candle_count,
        }
    }


    pub fn data_x_to_screen(&self, index: usize) -> f64 {
        data_x_to_screen(index, self.screen_width, self.zoom_level, self.offset_x)
    }


    pub fn screen_x_to_data(&self, screen_x: f64) -> usize {
        screen_x_to_data(
            screen_x,
            self.screen_width,
            self.zoom_level,
            self.offset_x,
            self.candle_count,
        )
    }


    pub fn price_to_screen(&self, price: f64) -> f64 {
        price_to_screen(
            price,
            self.screen_height,
            self.price_range,
            self.zoom_level,
            self.offset_y,
            self.center_price,
        )
    }


    pub fn screen_y_to_price(&self, screen_y: f64) -> f64 {
        screen_y_to_price(
            screen_y,
            self.screen_height,
            self.price_range,
            self.zoom_level,
            self.offset_y,
            self.center_price,
        )
    }


    pub fn screen_to_data_point(&self, screen: ScreenPoint) -> DataPoint {
        let chart = crate::chart::coordinate::screen_to_chart_point(screen, 0.0, 0.0);
        chart_point_to_data_point(
            chart,
            self.screen_width,
            self.screen_height,
            self.zoom_level,
            self.offset_x,
            self.offset_y,
            self.price_range,
            self.center_price,
            self.candle_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_x_is_identity() {
        let converter =
            CoordinateConverter::new(1920.0, 1080.0, 2.0, 10.0, 5.0, 1000.0, 50000.0, 100);
        for i in 0..100usize {
            let sx = converter.data_x_to_screen(i);
            let recovered = converter.screen_x_to_data(sx);
            assert_eq!(recovered, i, "round-trip falló para índice {}", i);
        }
    }

    #[test]
    fn price_to_screen_and_back_is_identity() {
        let converter =
            CoordinateConverter::new(1920.0, 1080.0, 2.0, 10.0, 5.0, 1000.0, 50000.0, 100);
        for price in [49000.0, 50000.0, 51000.0] {
            let sy = converter.price_to_screen(price);
            let recovered = converter.screen_y_to_price(sy);
            assert!(
                (recovered - price).abs() < 0.01,
                "round-trip Y falló para precio {}",
                price
            );
        }
    }
}
