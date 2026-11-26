use crate::chart::layout::{ChartLayout, Rect};
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl RenderColor {
    pub const GREEN: Self = Self::rgba(0, 255, 0, 255);
    pub const RED: Self = Self::rgba(255, 0, 0, 255);
    pub const ORANGE: Self = Self::rgba(255, 165, 0, 255);

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderLayer {

    pub candles: Vec<CandleRender>,

    pub volume_bars: Vec<VolumeBarRender>,

    pub price_axis_labels: Vec<PriceAxisLabel>,

    pub time_axis_labels: Vec<TimeAxisLabel>,

    pub crosshair: Option<CrosshairRender>,
}

#[derive(Debug, Clone, Copy)]
pub struct CandleRender {

    pub index: usize,

    pub open_y: f32,

    pub high_y: f32,

    pub low_y: f32,

    pub close_y: f32,

    pub center_x: f32,

    pub body_width: f32,

    pub color: RenderColor,

    pub volume: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct VolumeBarRender {

    pub index: usize,

    pub center_x: f32,

    pub height: f32,

    pub base_y: f32,

    pub color: RenderColor,
}

#[derive(Debug, Clone)]
pub struct PriceAxisLabel {

    pub text: String,

    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct TimeAxisLabel {

    pub text: String,

    pub x: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CrosshairRender {

    pub x: f32,

    pub y: f32,

    pub price: f64,

    pub index: usize,

    pub color: RenderColor,
}

#[derive(Debug, Clone)]
pub struct RenderFrame {

    pub rect: Rect,

    pub origin: ScreenPoint,

    pub layout: ChartLayout,

    pub layers: Vec<RenderLayer>,

    pub crosshair_pos: Option<ScreenPoint>,
}

impl RenderFrame {

    pub fn from_scene(
        scene: &ChartScene<'_>,
        width: f64,
        height: f64,
        origin: ScreenPoint,
        include_series: bool,
    ) -> Self {
        let layout = ChartLayout::compute(width, height);
        let layers = if include_series && layout.is_valid() {
            vec![build_series_layer(scene, &layout)]
        } else {
            Vec::new()
        };

        Self {
            rect: layout.full,
            origin,
            layout,
            layers,
            crosshair_pos: scene.crosshair_pos,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderGeometry {

    pub candle_vertices: Vec<[f32; 4]>,

    pub candle_indices: Vec<u16>,

    pub wick_vertices: Vec<[f32; 4]>,

    pub volume_vertices: Vec<[f32; 4]>,
}

#[derive(Debug, Clone)]
pub struct ChartScene<'a> {

    pub data_points: &'a [ChartDataPoint],

    pub viewport: Viewport,

    pub price_range: f64,

    pub center_price: f64,

    pub price_tick_size: f64,

    pub max_volume: f64,

    pub candle_count: usize,

    pub screen_width: f64,

    pub screen_height: f64,

    pub crosshair_pos: Option<ScreenPoint>,

    pub invalidation_mask: InvalidationMask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvalidationReason {

    #[default]
    None,

    CursorOnly,

    Overlay,

    SeriesData,

    ScaleLayout,

    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InvalidationMask {

    pub level: InvalidationReason,
}

impl InvalidationMask {

    pub fn none() -> Self {
        Self {
            level: InvalidationReason::None,
        }
    }

    pub fn is_none(&self) -> bool {
        self.level == InvalidationReason::None
    }

    pub fn merge(self, other: Self) -> Self {
        let level = if self.level as u8 > other.level as u8 {
            self.level
        } else {
            other.level
        };
        Self { level }
    }
}

impl<'a> ChartScene<'a> {

    pub fn empty() -> Self {
        Self {
            data_points: &[],
            viewport: Viewport::new(),
            price_range: 0.0,
            center_price: 0.0,
            price_tick_size: 0.01,
            max_volume: 0.0,
            candle_count: 0,
            screen_width: 0.0,
            screen_height: 0.0,
            crosshair_pos: None,
            invalidation_mask: InvalidationMask::none(),
        }
    }
}

impl<'a> Default for ChartScene<'a> {
    fn default() -> Self {
        Self::empty()
    }
}

fn build_series_layer(scene: &ChartScene<'_>, layout: &ChartLayout) -> RenderLayer {
    if scene.data_points.is_empty()
        || !scene.price_range.is_finite()
        || scene.price_range <= 0.0
        || !scene.center_price.is_finite()
    {
        return RenderLayer::default();
    }

    let plot_width = layout.plot_width();
    let visible = scene.viewport.visible_range(scene.candle_count, plot_width);
    if visible.is_empty() {
        return RenderLayer::default();
    }

    let candle_width = crate::domain::transforms::candle_body_width(scene.viewport.zoom_x())
        .clamp(4.0, 40.0) as f32;
    let price_pixels_per_unit = layout.plot_height() / scene.price_range * scene.viewport.zoom_y();
    let volume_panel_height = layout.volume_panel.height();

    let price_to_y = |price: f64| -> f32 {
        let Some(scale_value) = scene.viewport.price_to_scale_value(price) else {
            return f32::NAN;
        };
        let relative_price =
            scale_value - scene.center_price - scene.viewport.offset_y() * scene.price_range;
        (layout.plot.top + layout.plot.height() / 2.0 - relative_price * price_pixels_per_unit)
            as f32
    };

    let mut layer = RenderLayer {
        candles: Vec::with_capacity(visible.len()),
        volume_bars: Vec::with_capacity(visible.len()),
        ..RenderLayer::default()
    };

    for (data_index, point) in scene
        .data_points
        .iter()
        .enumerate()
        .skip(visible.first)
        .take(visible.len())
    {
        let logical_index = point.logical_index.unwrap_or(data_index);
        let center_x = (scene
            .viewport
            .logical_to_screen(logical_index as f64, plot_width)
            + layout.plot.left) as f32;

        let color = if point.close() >= point.open() {
            RenderColor::GREEN
        } else {
            RenderColor::RED
        };
        let volume_fraction = if scene.max_volume.is_finite() && scene.max_volume > 0.0 {
            (point.volume() / scene.max_volume).max(0.0)
        } else {
            0.0
        };
        let volume_height = (volume_fraction * volume_panel_height).max(1.0) as f32;

        layer.candles.push(CandleRender {
            index: logical_index,
            open_y: price_to_y(point.open()),
            high_y: price_to_y(point.high()),
            low_y: price_to_y(point.low()),
            close_y: price_to_y(point.close()),
            center_x,
            body_width: candle_width,
            color,
            volume: point.volume(),
        });
        layer.volume_bars.push(VolumeBarRender {
            index: logical_index,
            center_x,
            height: volume_height,
            base_y: layout.volume_panel.bottom as f32,
            color,
        });
    }

    layer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;
    use crate::domain::chart::time_point::TimePoint;

    fn sample_scene<'a>(points: &'a [ChartDataPoint]) -> ChartScene<'a> {
        ChartScene {
            data_points: points,
            viewport: Viewport::new(),
            price_range: 10.0,
            center_price: 102.0,
            price_tick_size: 0.01,
            max_volume: 1_000.0,
            candle_count: points.len(),
            screen_width: 744.0,
            screen_height: 405.0,
            crosshair_pos: None,
            invalidation_mask: InvalidationMask {
                level: InvalidationReason::Full,
            },
        }
    }

    #[test]
    fn chart_scene_borrows_data_without_copying_the_history() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).expect("vela válida");
        let time_point = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let points = [ChartDataPoint::new(candle, time_point)];
        let original_ptr = points.as_ptr();

        let scene = sample_scene(&points);

        assert_eq!(scene.data_points.as_ptr(), original_ptr);
        assert_eq!(scene.data_points.len(), 1);
        assert_eq!(scene.data_points[0].close(), 103.0);
    }

    #[test]
    fn empty_scene_uses_an_empty_borrowed_slice() {
        let scene = ChartScene::empty();

        assert!(scene.data_points.is_empty());
        assert_eq!(scene.candle_count, 0);
        assert_eq!(scene.max_volume, 0.0);
    }

    #[test]
    fn render_frame_builds_backend_neutral_candle_and_volume_geometry() {
        let time_a = TimePoint::new(0, 60_000, 60_000).expect("tiempo válido");
        let time_b = TimePoint::new(60_000, 120_000, 60_000).expect("tiempo válido");
        let points = [
            ChartDataPoint::new(
                Candle::new(100.0, 105.0, 98.0, 103.0, 500.0).expect("vela alcista válida"),
                time_a,
            ),
            ChartDataPoint::new(
                Candle::new(103.0, 106.0, 99.0, 101.0, 1_000.0).expect("vela bajista válida"),
                time_b,
            ),
        ];
        let scene = sample_scene(&points);
        let origin = ScreenPoint { x: 30.0, y: 40.0 };

        let frame = RenderFrame::from_scene(&scene, 800.0, 500.0, origin, true);

        assert!(frame.layout.is_valid());
        assert_eq!(frame.rect.width(), 800.0);
        assert_eq!(frame.origin, origin);
        assert_eq!(frame.layers.len(), 1);

        let layer = &frame.layers[0];
        assert_eq!(layer.candles.len(), 2);
        assert_eq!(layer.volume_bars.len(), 2);
        assert_eq!(layer.candles[0].index, 0);
        assert_eq!(layer.candles[0].color, RenderColor::GREEN);
        assert_eq!(layer.candles[1].index, 1);
        assert_eq!(layer.candles[1].color, RenderColor::RED);
        assert_eq!(layer.volume_bars[0].index, layer.candles[0].index);
        assert_eq!(layer.volume_bars[1].index, layer.candles[1].index);
        assert!(layer.candles.iter().all(|candle| candle.body_width > 0.0));
        assert!(layer.volume_bars.iter().all(|bar| bar.height > 0.0));
    }

    #[test]
    fn render_frame_can_skip_series_geometry_for_reusable_frames() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).expect("vela válida");
        let time = TimePoint::new(0, 60_000, 60_000).expect("tiempo válido");
        let points = [ChartDataPoint::new(candle, time)];
        let scene = sample_scene(&points);

        let frame =
            RenderFrame::from_scene(&scene, 800.0, 500.0, ScreenPoint { x: 0.0, y: 0.0 }, false);

        assert!(frame.layers.is_empty());
        assert!(frame.layout.is_valid());
    }

    #[test]
    fn render_frame_has_no_geometry_for_invalid_layout() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).expect("vela válida");
        let time = TimePoint::new(0, 60_000, 60_000).expect("tiempo válido");
        let points = [ChartDataPoint::new(candle, time)];
        let scene = sample_scene(&points);

        let frame =
            RenderFrame::from_scene(&scene, 0.0, 500.0, ScreenPoint { x: 0.0, y: 0.0 }, true);

        assert!(!frame.layout.is_valid());
        assert!(frame.layers.is_empty());
    }
}
