





use crate::chart::layout::ChartLayout;
use crate::domain::transforms::{self, ScreenPoint};
use crate::domain::viewport::Viewport;


#[derive(Debug, Clone, PartialEq)]
pub enum HitKind {

    Candle { index: usize },

    Series { series_id: HitSeriesId },

    Drawing { drawing_id: HitDrawingId },

    Marker { marker_id: HitMarkerId },

    PriceLine { price: f64 },

    AxisLabel { axis: HitAxis, label: String },

    Overlay { overlay_id: HitOverlayId },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitSeriesId(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitDrawingId(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitMarkerId(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitOverlayId(pub u64);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitAxis {

    Price,

    Time,
}


#[derive(Debug, Clone)]
pub struct HitResult {

    pub kind: HitKind,

    pub screen_point: ScreenPoint,

    pub data_point: transforms::DataPoint,


    pub distance: f64,
}



#[derive(Debug, Clone, Copy)]
pub struct HitTestContext<'a> {

    pub viewport: &'a Viewport,

    pub layout: &'a ChartLayout,

    pub screen_x: f64,

    pub screen_y: f64,

    pub price_range: f64,

    pub center_price: f64,

    pub candle_count: usize,
}

impl<'a> HitTestContext<'a> {

    pub fn new(
        viewport: &'a Viewport,
        layout: &'a ChartLayout,
        screen_x: f64,
        screen_y: f64,
        price_range: f64,
        center_price: f64,
        candle_count: usize,
    ) -> Self {
        Self {
            viewport,
            layout,
            screen_x,
            screen_y,
            price_range,
            center_price,
            candle_count,
        }
    }


    pub fn is_inside_plot(&self) -> bool {
        if !self.layout.is_valid() || !self.layout.plot.is_valid() {
            return false;
        }
        let plot = self.layout.plot;
        self.screen_x >= plot.left
            && self.screen_x <= plot.right
            && self.screen_y >= plot.top
            && self.screen_y <= plot.bottom
    }



    pub fn cursor_to_data(&self) -> Option<transforms::DataPoint> {
        if !self.is_inside_plot() {
            return None;
        }

        let plot = self.layout.plot;
        let rel_x = self.screen_x - plot.left;
        let rel_y = self.screen_y - plot.top;

        let center_x = plot.width() / 2.0;
        let spacing = self.viewport.candle_spacing();

        let index = if spacing > 0.0 {
            ((rel_x - center_x) / spacing + self.viewport.offset_x()).round()
        } else {
            0.0
        };

        let price = transforms::screen_y_to_price(
            rel_y,
            plot.height(),
            self.price_range,
            self.viewport.zoom_y(),
            self.viewport.offset_y(),
            self.center_price,
        );

        Some(transforms::DataPoint {
            index: index.clamp(0.0, self.candle_count as f64 - 1.0),
            price,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;

    fn make_viewport() -> Viewport {
        let mut v = Viewport::new();
        v.set_zoom(2.0, 2.0);
        v
    }

    fn make_layout() -> ChartLayout {
        ChartLayout::compute(1920.0, 1080.0)
    }

    #[test]
    fn hit_context_inside_plot() {
        let viewport = make_viewport();
        let layout = make_layout();
        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;

        let ctx = HitTestContext::new(&viewport, &layout, cx, cy, 100.0, 50000.0, 100);

        assert!(ctx.is_inside_plot());
    }

    #[test]
    fn hit_context_outside_plot() {
        let viewport = make_viewport();
        let layout = make_layout();

        let ctx = HitTestContext::new(&viewport, &layout, -10.0, 500.0, 100.0, 50000.0, 100);

        assert!(!ctx.is_inside_plot());
    }

    #[test]
    fn hit_context_cursor_to_data_center() {
        let viewport = make_viewport();
        let layout = make_layout();
        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;

        let ctx = HitTestContext::new(&viewport, &layout, cx, cy, 100.0, 50000.0, 100);

        let dp = ctx.cursor_to_data().expect("cursor dentro del plot");

        assert!((dp.index - 0.0).abs() < f64::EPSILON);
        assert!((dp.price - 50000.0).abs() < 1.0);
    }

    #[test]
    fn hit_result_default_fields() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).expect("vela válida");
        let _ = candle;
        let result = HitResult {
            kind: HitKind::Candle { index: 5 },
            screen_point: ScreenPoint { x: 100.0, y: 200.0 },
            data_point: transforms::DataPoint {
                index: 5.0,
                price: 50250.0,
            },
            distance: 0.0,
        };
        assert_eq!(result.distance, 0.0);
        assert!((result.data_point.index - 5.0).abs() < f64::EPSILON);
    }
}
