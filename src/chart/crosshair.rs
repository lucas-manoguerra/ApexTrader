





use crate::chart::layout::ChartLayout;
use crate::domain::viewport::Viewport;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrosshairMode {

    #[default]
    Full,

    LabelsOnly,

    Hidden,
}





#[derive(Debug, Clone, Copy)]
pub struct CrosshairUpdateParams<'a> {

    pub layout: &'a ChartLayout,

    pub price_range: f64,

    pub center_price: f64,

    pub candle_count: usize,
}


#[derive(Debug, Clone, Copy)]
pub struct Crosshair {

    pub mode: CrosshairMode,

    pub x: f64,

    pub y: f64,

    pub index: usize,

    pub price: f64,

    pub visible: bool,
}

impl Crosshair {

    pub fn hidden() -> Self {
        Self {
            mode: CrosshairMode::Hidden,
            x: 0.0,
            y: 0.0,
            index: 0,
            price: 0.0,
            visible: false,
        }
    }







    pub fn update(
        &mut self,
        cursor_x: f64,
        cursor_y: f64,
        params: CrosshairUpdateParams<'_>,
        viewport: &Viewport,
    ) {
        let layout = params.layout;
        if !layout.is_valid() || !layout.plot.is_valid() {
            self.visible = false;
            return;
        }

        let plot_left = layout.plot.left;
        let plot_top = layout.plot.top;
        let plot_width = layout.plot.width();
        let plot_height = layout.plot.height();
        let price_range = params.price_range;
        let center_price = params.center_price;
        let candle_count = params.candle_count;

        if candle_count == 0 {
            self.visible = false;
            self.index = 0;
            return;
        }

        let rel_x = cursor_x - plot_left;
        let rel_y = cursor_y - plot_top;

        if rel_x < 0.0 || rel_x > plot_width || rel_y < 0.0 || rel_y > plot_height {
            self.visible = false;
            return;
        }

        self.visible = true;
        self.x = cursor_x;
        self.y = cursor_y;

        let center_x = plot_width / 2.0;
        let spacing = viewport.candle_spacing();
        if spacing > 0.0 {
            let index = ((rel_x - center_x) / spacing + viewport.offset_x()).round();
            self.index = index.clamp(0.0, candle_count as f64 - 1.0) as usize;
        } else {
            self.index = 0;
        }

        self.price = viewport
            .screen_y_to_price_value(rel_y, plot_height, price_range, center_price)
            .or_else(|| viewport.scale_value_to_price(center_price))
            .unwrap_or_default();
    }


    pub fn is_visible(&self) -> bool {
        self.visible && self.mode != CrosshairMode::Hidden
    }


    pub fn draw_lines(&self) -> bool {
        self.is_visible() && self.mode == CrosshairMode::Full
    }


    pub fn draw_labels(&self) -> bool {
        self.is_visible() && self.mode != CrosshairMode::Hidden
    }
}

impl Default for Crosshair {
    fn default() -> Self {
        Self::hidden()
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
    fn crosshair_default_is_hidden() {
        let ch = Crosshair::default();
        assert!(!ch.is_visible());
        assert!(!ch.draw_lines());
        assert!(!ch.draw_labels());
    }

    #[test]
    fn crosshair_outside_plot_is_invisible() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();

        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(0.0, 500.0, params, &viewport);
        assert!(!ch.visible);
    }

    #[test]
    fn crosshair_with_no_candles_is_invisible() {
        let mut ch = Crosshair {
            visible: true,
            ..Crosshair::default()
        };
        let layout = make_layout();
        let viewport = make_viewport();
        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;
        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 0,
        };

        ch.update(cx, cy, params, &viewport);

        assert!(!ch.visible);
        assert_eq!(ch.index, 0);
    }

    #[test]
    fn crosshair_inside_plot_is_visible() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();


        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;
        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(cx, cy, params, &viewport);

        assert!(ch.visible);
        assert_eq!(ch.x, cx);
        assert_eq!(ch.y, cy);
    }

    #[test]
    fn crosshair_index_at_center_is_zero() {
        let mut ch = Crosshair::default();
        let layout = make_layout();
        let viewport = make_viewport();

        let cx = layout.plot.left + layout.plot.width() / 2.0;
        let cy = layout.plot.top + layout.plot.height() / 2.0;
        let params = CrosshairUpdateParams {
            layout: &layout,
            price_range: 100.0,
            center_price: 50000.0,
            candle_count: 100,
        };
        ch.update(cx, cy, params, &viewport);


        assert_eq!(ch.index, 0);
    }

    #[test]
    fn crosshair_mode_labels_only_draws_labels_not_lines() {
        let ch = Crosshair {
            mode: CrosshairMode::LabelsOnly,
            visible: true,
            ..Default::default()
        };
        assert!(ch.draw_labels());
        assert!(!ch.draw_lines());
    }

    #[test]
    fn crosshair_hidden_never_draws() {
        let ch = Crosshair {
            mode: CrosshairMode::Hidden,
            visible: true,
            ..Default::default()
        };
        assert!(!ch.draw_labels());
        assert!(!ch.draw_lines());
    }

    #[test]
    fn candle_is_pure_domain() {
        let c = Candle::new(100.0, 105.0, 99.0, 103.0, 1000.0).expect("vela válida");
        assert_eq!(c.open(), 100.0);
    }
}
