use crate::domain::chart::time_scale::TimeScale;
use crate::domain::transforms::{self, ScreenPoint, screen_to_chart_point};
use crate::domain::viewport::{PriceScaleType, Viewport};
use crate::errors::RendererError;
use crate::renderer::{
    ChartGpuCallback, ChartRenderer, ChartScene, GpuVertex, RenderFrame, RenderLayer,
    build_geometry,
};
use eframe::egui;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);

pub struct EguiPainterChartRenderer {
    callback_id: u64,
    resource_owner: Arc<()>,
    cached_series_layer: RenderLayer,
    cached_series_shapes: Vec<egui::Shape>,
    cached_gpu_vertices: Arc<[GpuVertex]>,
    target_format: Option<eframe::wgpu::TextureFormat>,
    series_cache_valid: bool,
}

impl EguiPainterChartRenderer {
    pub fn new() -> Self {
        Self {
            callback_id: NEXT_CALLBACK_ID.fetch_add(1, Ordering::Relaxed),
            resource_owner: Arc::new(()),
            cached_series_layer: RenderLayer::default(),
            cached_series_shapes: Vec::new(),
            cached_gpu_vertices: Arc::from([]),
            target_format: None,
            series_cache_valid: false,
        }
    }

    pub fn with_target_format(target_format: Option<eframe::wgpu::TextureFormat>) -> Self {
        Self {
            target_format,
            ..Self::new()
        }
    }
}

impl Default for EguiPainterChartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn time_label_at_logical(
    data_points: &[crate::domain::chart::chart_data::ChartDataPoint],
    logical_index: f64,
) -> String {
    if data_points.is_empty() || !logical_index.is_finite() {
        return String::new();
    }

    let bounded_index = logical_index.clamp(0.0, (data_points.len() - 1) as f64);
    let left_index = bounded_index.floor() as usize;
    let right_index = bounded_index.ceil() as usize;
    let Some(left) = data_points.get(left_index) else {
        return String::new();
    };

    if left_index == right_index {
        return TimeScale::format_time_point(left.time_point)
            .trim_end_matches(" UTC")
            .to_owned();
    }

    let Some(right) = data_points.get(right_index) else {
        return String::new();
    };
    let fraction = bounded_index - left_index as f64;
    let start = left.time_point.open_time_ms() as i128;
    let end = right.time_point.open_time_ms() as i128;
    let timestamp = (start as f64 + (end - start) as f64 * fraction).round();

    if !timestamp.is_finite() || timestamp < i64::MIN as f64 || timestamp > i64::MAX as f64 {
        return String::new();
    }

    TimeScale::format_time_label(timestamp as i64)
        .trim_end_matches(" UTC")
        .to_owned()
}

fn nice_tick_step(range: f64, target_ticks: usize) -> f64 {
    if !range.is_finite() || range <= 0.0 || target_ticks == 0 {
        return 1.0;
    }

    let raw_step = range / target_ticks as f64;
    if !raw_step.is_finite() || raw_step <= 0.0 {
        return range;
    }

    let magnitude = 10.0f64.powi(raw_step.log10().floor() as i32);
    if !magnitude.is_finite() || magnitude <= 0.0 {
        return raw_step;
    }

    let residual = raw_step / magnitude;
    if residual < 1.5 {
        magnitude
    } else if residual < 3.0 {
        2.0 * magnitude
    } else if residual < 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    }
}

fn decimal_places_for_increment(increment: f64) -> usize {
    if !increment.is_finite() || increment <= 0.0 {
        return 2;
    }

    for decimals in 0_i32..=12 {
        let scale = 10.0_f64.powi(decimals);
        let scaled = increment * scale;
        if scaled.is_finite()
            && (scaled - scaled.round()).abs() <= f64::EPSILON * scaled.abs().max(1.0) * 8.0
        {
            return decimals as usize;
        }
    }
    12
}

fn format_price_tick(value: f64, step: f64, minimum_tick: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }

    let decimals = decimal_places_for_increment(step)
        .max(decimal_places_for_increment(minimum_tick))
        .clamp(2, 12);
    format!("{:.*}", decimals, value)
}

fn aligned_price_tick_step(range: f64, target_ticks: usize, minimum_tick: f64) -> f64 {
    let minimum_tick = if minimum_tick.is_finite() && minimum_tick > 0.0 {
        minimum_tick
    } else {
        0.01
    };
    let nice_step = nice_tick_step(range, target_ticks);
    if !nice_step.is_finite() || nice_step <= 0.0 {
        return minimum_tick;
    }

    let multiples = (nice_step / minimum_tick).ceil().max(1.0);
    let aligned = multiples * minimum_tick;
    if aligned.is_finite() && aligned > 0.0 {
        aligned
    } else {
        nice_step
    }
}

#[derive(Debug, Clone, Copy)]
struct PriceAxisParams {
    center_price: f64,
    offset_y: f64,
    price_range: f64,
    zoom_y: f64,
    minimum_tick: f64,
    scale_type: PriceScaleType,
    inverted: bool,
    percentage_base: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct VolumeAxisParams {
    max_volume: f64,
}

struct TimeAxisParams<'a> {
    viewport: &'a Viewport,
    data_points: &'a [crate::domain::chart::chart_data::ChartDataPoint],
    logical_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct PriceTick {
    value: f64,
    scale_value: f64,
    y: f64,
    label_step: f64,
}

#[derive(Debug, Clone)]
struct TimeAxisTick {
    screen_x: f64,
    label: String,
}

fn price_axis_label(
    value: f64,
    step: f64,
    minimum_tick: f64,
    scale_type: PriceScaleType,
) -> String {
    match scale_type {
        PriceScaleType::Percentage => format!("{value:+.2}%"),
        PriceScaleType::Linear | PriceScaleType::Logarithmic => {
            format_price_tick(value, step, minimum_tick)
        }
    }
}

fn price_axis_ticks(top: f64, bottom: f64, params: PriceAxisParams) -> Vec<PriceTick> {
    let height = bottom - top;
    if !height.is_finite()
        || height <= 0.0
        || !params.price_range.is_finite()
        || params.price_range <= 0.0
        || !params.center_price.is_finite()
        || !params.offset_y.is_finite()
        || !params.zoom_y.is_finite()
        || params.zoom_y <= 0.0
    {
        return Vec::new();
    }

    let half_range_visible = params.price_range / (2.0 * params.zoom_y);
    let visible_center = params.center_price + params.offset_y * params.price_range;
    let min_visible = visible_center - half_range_visible;
    let max_visible = visible_center + half_range_visible;
    let target_ticks = ((height / 56.0).floor() as usize).clamp(4, 8);
    let pixels_per_unit = height / params.price_range * params.zoom_y;
    let direction = if params.inverted { -1.0 } else { 1.0 };
    let y_for_scale_value =
        |value: f64| top + height / 2.0 - (value - visible_center) * pixels_per_unit * direction;
    let mut ticks = Vec::with_capacity(target_ticks + 2);

    match params.scale_type {
        PriceScaleType::Linear => {
            let tick_step = aligned_price_tick_step(
                max_visible - min_visible,
                target_ticks,
                params.minimum_tick,
            );
            if !tick_step.is_finite() || tick_step <= 0.0 {
                return Vec::new();
            }

            let mut value = (min_visible / tick_step).ceil() * tick_step;
            for _ in 0..(target_ticks + 4) {
                if !value.is_finite() || value > max_visible + tick_step * 1e-9 {
                    break;
                }
                ticks.push(PriceTick {
                    value,
                    scale_value: value,
                    y: y_for_scale_value(value),
                    label_step: tick_step,
                });
                value += tick_step;
            }
        }
        PriceScaleType::Percentage => {
            let tick_step = nice_tick_step(max_visible - min_visible, target_ticks);
            if !tick_step.is_finite() || tick_step <= 0.0 {
                return Vec::new();
            }

            let mut value = (min_visible / tick_step).ceil() * tick_step;
            for _ in 0..(target_ticks + 4) {
                if !value.is_finite() || value > max_visible + tick_step * 1e-9 {
                    break;
                }
                ticks.push(PriceTick {
                    value,
                    scale_value: value,
                    y: y_for_scale_value(value),
                    label_step: tick_step,
                });
                value += tick_step;
            }
        }
        PriceScaleType::Logarithmic => {
            let Some(min_price) = params
                .scale_type
                .from_scale_value(min_visible, params.percentage_base)
            else {
                return Vec::new();
            };
            let Some(max_price) = params
                .scale_type
                .from_scale_value(max_visible, params.percentage_base)
            else {
                return Vec::new();
            };
            if min_price <= 0.0 || max_price <= min_price {
                return Vec::new();
            }

            let first_decade = min_price.log10().floor() as i32 - 1;
            let last_decade = max_price.log10().ceil() as i32 + 1;
            let decade_count = (last_decade - first_decade + 1).max(1) as usize;
            let decade_stride = decade_count.div_ceil(target_ticks.max(1)).max(1);
            for exponent in (first_decade..=last_decade).step_by(decade_stride) {
                let decade = 10.0_f64.powi(exponent);
                if !decade.is_finite() || decade <= 0.0 {
                    continue;
                }
                for multiplier in [1.0, 2.0, 5.0] {
                    let price = decade * multiplier;
                    let Some(scale_value) = params
                        .scale_type
                        .to_scale_value(price, params.percentage_base)
                    else {
                        continue;
                    };
                    if scale_value < min_visible - 1e-10 || scale_value > max_visible + 1e-10 {
                        continue;
                    }
                    ticks.push(PriceTick {
                        value: price,
                        scale_value,
                        y: y_for_scale_value(scale_value),
                        label_step: params.minimum_tick,
                    });
                }
            }
            ticks.sort_by(|left, right| left.scale_value.total_cmp(&right.scale_value));
        }
    }

    ticks
}

fn time_axis_ticks(left: f64, right: f64, params: &TimeAxisParams<'_>) -> Vec<TimeAxisTick> {
    let width = right - left;
    if !width.is_finite() || width <= 0.0 || params.logical_count == 0 {
        return Vec::new();
    }

    let visible = params.viewport.visible_range(params.logical_count, width);
    if visible.is_empty() {
        return Vec::new();
    }

    let target_labels = ((width / 118.0).floor() as usize).clamp(2, 9);
    let logical_span = visible.last.saturating_sub(visible.first).max(1);
    let bar_step = nice_tick_step(logical_span as f64, target_labels.saturating_sub(1))
        .round()
        .clamp(1.0, usize::MAX as f64) as usize;

    let base_interval_ms = params
        .data_points
        .iter()
        .find(|point| point.time_point.interval_ms() > 0)
        .map(|point| point.time_point.interval_ms())
        .unwrap_or(1);
    let tick_interval_ms = base_interval_ms
        .saturating_mul(bar_step.min(i64::MAX as usize) as i64)
        .max(1);

    let mut ticks = Vec::with_capacity(target_labels + 1);
    let mut previous_bucket: Option<i64> = None;
    let mut previous_tick_timestamp: Option<i64> = None;
    let mut last_screen_x: Option<f64> = None;

    for point in params.data_points {
        let Some(index) = point.logical_index else {
            continue;
        };
        if index > visible.last {
            break;
        }

        let timestamp = point.time_point.open_time_ms();
        let bucket = timestamp.div_euclid(tick_interval_ms);
        if index < visible.first {
            previous_bucket = Some(bucket);
            continue;
        }
        if previous_bucket == Some(bucket) {
            continue;
        }
        previous_bucket = Some(bucket);

        let screen_x = params.viewport.logical_to_screen(index as f64, width) + left;
        if screen_x < left + 4.0 || screen_x > right - 4.0 {
            continue;
        }

        let label =
            TimeScale::format_time_axis_label(timestamp, tick_interval_ms, previous_tick_timestamp);

        let minimum_spacing = (label.chars().count() as f64 * 7.5 + 24.0).max(72.0);
        if last_screen_x.is_some_and(|last| screen_x - last < minimum_spacing) {
            continue;
        }

        ticks.push(TimeAxisTick { screen_x, label });
        last_screen_x = Some(screen_x);
        previous_tick_timestamp = Some(timestamp);
    }

    ticks
}

fn screen_rect(rect: crate::chart::layout::Rect, origin: ScreenPoint) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2((rect.left + origin.x) as f32, (rect.top + origin.y) as f32),
        egui::pos2(
            (rect.right + origin.x) as f32,
            (rect.bottom + origin.y) as f32,
        ),
    )
}

fn draw_chart_surfaces(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
) {
    let white = egui::Color32::from_rgb(255, 255, 255);
    let panel = egui::Color32::from_rgb(249, 250, 252);
    let volume = egui::Color32::from_rgb(252, 253, 255);
    let divider = egui::Color32::from_rgb(220, 225, 232);

    painter.rect_filled(
        screen_rect(layout.full, origin),
        egui::CornerRadius::ZERO,
        white,
    );
    painter.rect_filled(
        screen_rect(layout.volume_panel, origin),
        egui::CornerRadius::ZERO,
        volume,
    );

    let right_axis_background = egui::Rect::from_min_max(
        egui::pos2(
            (layout.price_axis_right.left + origin.x) as f32,
            (layout.plot.top + origin.y) as f32,
        ),
        egui::pos2(
            (layout.full.right + origin.x) as f32,
            (layout.volume_panel.bottom + origin.y) as f32,
        ),
    );
    painter.rect_filled(right_axis_background, egui::CornerRadius::ZERO, panel);
    painter.rect_filled(
        screen_rect(layout.time_axis, origin),
        egui::CornerRadius::ZERO,
        panel,
    );

    let plot_left = (layout.plot.left + origin.x) as f32;
    let plot_right = (layout.plot.right + origin.x) as f32;
    let plot_top = (layout.plot.top + origin.y) as f32;
    let plot_bottom = (layout.plot.bottom + origin.y) as f32;
    let volume_bottom = (layout.volume_panel.bottom + origin.y) as f32;
    let full_right = (layout.full.right + origin.x) as f32;

    painter.line_segment(
        [
            egui::pos2(plot_right, plot_top),
            egui::pos2(plot_right, volume_bottom),
        ],
        egui::Stroke::new(1.0, divider),
    );
    painter.line_segment(
        [
            egui::pos2(plot_left, plot_bottom),
            egui::pos2(full_right, plot_bottom),
        ],
        egui::Stroke::new(1.0, divider),
    );
    painter.line_segment(
        [
            egui::pos2(plot_left, volume_bottom),
            egui::pos2(full_right, volume_bottom),
        ],
        egui::Stroke::new(1.0, divider),
    );
}

fn draw_grid(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
    price_ticks: &[PriceTick],
    time_ticks: &[TimeAxisTick],
) {
    let grid = egui::Color32::from_rgb(232, 236, 242);
    let stroke = egui::Stroke::new(1.0, grid);
    let left = (layout.plot.left + origin.x) as f32;
    let right = (layout.plot.right + origin.x) as f32;
    let top = (layout.plot.top + origin.y) as f32;
    let volume_bottom = (layout.volume_panel.bottom + origin.y) as f32;

    for tick in price_ticks {
        if tick.y >= layout.plot.top + origin.y && tick.y <= layout.plot.bottom + origin.y {
            let y = tick.y as f32;
            painter.line_segment([egui::pos2(left, y), egui::pos2(right, y)], stroke);
        }
    }

    for tick in time_ticks {
        let x = tick.screen_x as f32;
        painter.line_segment([egui::pos2(x, top), egui::pos2(x, volume_bottom)], stroke);
    }
}

fn draw_price_axis(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
    ticks: &[PriceTick],
    minimum_tick: f64,
    scale_type: PriceScaleType,
    font_id: egui::FontId,
) {
    let axis_left = (layout.price_axis_right.left + origin.x) as f32;
    let plot_right = (layout.plot.right + origin.x) as f32;
    let axis_text = egui::Color32::from_rgb(92, 101, 114);
    let tick_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(191, 198, 208));

    for tick in ticks {
        if tick.y < layout.plot.top + origin.y || tick.y > layout.plot.bottom + origin.y {
            continue;
        }
        let y = tick.y as f32;
        painter.line_segment(
            [egui::pos2(plot_right, y), egui::pos2(plot_right + 5.0, y)],
            tick_stroke,
        );
        painter.text(
            egui::pos2(axis_left + 9.0, y),
            egui::Align2::LEFT_CENTER,
            price_axis_label(tick.value, tick.label_step, minimum_tick, scale_type),
            font_id.clone(),
            axis_text,
        );
    }
}

fn draw_volume_axis(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
    params: VolumeAxisParams,
    font_id: egui::FontId,
) {
    if !params.max_volume.is_finite() || params.max_volume <= 0.0 {
        return;
    }

    let top = layout.volume_panel.top;
    let bottom = layout.volume_panel.bottom;
    let height = bottom - top;
    let target_ticks = ((height / 22.0).floor() as usize).clamp(2, 4);
    let tick_step = nice_tick_step(params.max_volume, target_ticks);
    if !tick_step.is_finite() || tick_step <= 0.0 {
        return;
    }

    let axis_left = (layout.price_axis_right.left + origin.x) as f32;
    let axis_text = egui::Color32::from_rgb(112, 120, 132);
    let mut tick = 0.0;
    for _ in 0..(target_ticks + 2) {
        if tick > params.max_volume + tick_step * 1e-9 {
            break;
        }
        let fraction = tick / params.max_volume;
        let y = bottom - fraction * height;
        let y = (y + origin.y) as f32;
        let label = if tick >= 1_000_000.0 {
            format!("{:.1}M", tick / 1_000_000.0)
        } else if tick >= 1_000.0 {
            format!("{:.1}K", tick / 1_000.0)
        } else {
            format!("{:.0}", tick)
        };
        painter.text(
            egui::pos2(axis_left + 9.0, y),
            egui::Align2::LEFT_CENTER,
            label,
            font_id.clone(),
            axis_text,
        );
        tick += tick_step;
    }
}

fn draw_time_axis(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
    ticks: &[TimeAxisTick],
    font_id: egui::FontId,
) {
    let axis_text = egui::Color32::from_rgb(92, 101, 114);
    let axis_top = (layout.time_axis.top + origin.y) as f32;
    for tick in ticks {
        let x = tick.screen_x as f32;
        painter.line_segment(
            [egui::pos2(x, axis_top), egui::pos2(x, axis_top + 5.0)],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(191, 198, 208)),
        );
        painter.text(
            egui::pos2(x, axis_top + 7.0),
            egui::Align2::CENTER_TOP,
            tick.label.clone(),
            font_id.clone(),
            axis_text,
        );
    }
}

fn draw_current_price_marker(
    painter: &egui::Painter,
    layout: &crate::chart::layout::ChartLayout,
    origin: ScreenPoint,
    params: PriceAxisParams,
    price: f64,
    color: egui::Color32,
) {
    if !price.is_finite() || params.price_range <= 0.0 || params.zoom_y <= 0.0 {
        return;
    }

    let top = layout.plot.top;
    let height = layout.plot_height();
    let Some(scale_value) = params
        .scale_type
        .to_scale_value(price, params.percentage_base)
    else {
        return;
    };
    let relative_price = scale_value - params.center_price - params.offset_y * params.price_range;
    let pixels_per_unit = height / params.price_range * params.zoom_y;
    let direction = if params.inverted { -1.0 } else { 1.0 };
    let y = top + height / 2.0 - relative_price * pixels_per_unit * direction;
    if !y.is_finite() || y < top || y > layout.plot.bottom {
        return;
    }

    let screen_y = (y + origin.y) as f32;
    let left = (layout.plot.left + origin.x) as f32;
    let right = (layout.plot.right + origin.x) as f32;
    let axis_left = (layout.price_axis_right.left + origin.x) as f32;
    let axis_right = (layout.price_axis_right.right + origin.x) as f32;
    let marker_y = screen_y;

    let line_color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 150);
    painter.line_segment(
        [egui::pos2(left, screen_y), egui::pos2(right, screen_y)],
        egui::Stroke::new(1.0, line_color),
    );

    let marker_rect = egui::Rect::from_min_max(
        egui::pos2(axis_left + 1.0, marker_y - 9.0),
        egui::pos2(axis_right - 1.0, marker_y + 9.0),
    );
    painter.rect_filled(marker_rect, egui::CornerRadius::same(2), color);
    painter.text(
        marker_rect.center(),
        egui::Align2::CENTER_CENTER,
        price_axis_label(
            if params.scale_type == PriceScaleType::Percentage {
                scale_value
            } else {
                price
            },
            params.minimum_tick,
            params.minimum_tick,
            params.scale_type,
        ),
        egui::FontId::proportional(11.0),
        egui::Color32::WHITE,
    );
}

fn series_layer_shapes(layer: &RenderLayer, origin: ScreenPoint) -> Vec<egui::Shape> {
    let mut shapes = Vec::with_capacity(layer.candles.len().saturating_mul(3));

    for (candle, volume) in layer.candles.iter().zip(&layer.volume_bars) {
        let color = egui::Color32::from_rgba_unmultiplied(
            candle.color.red,
            candle.color.green,
            candle.color.blue,
            candle.color.alpha,
        );
        let center_x = candle.center_x + origin.x as f32;
        let origin_y = origin.y as f32;

        shapes.push(egui::Shape::line_segment(
            [
                egui::pos2(center_x, candle.high_y + origin_y),
                egui::pos2(center_x, candle.low_y + origin_y),
            ],
            egui::Stroke::new(1.0, color),
        ));

        let body_height = (candle.close_y - candle.open_y).abs().max(2.0);
        let body_y = candle.open_y.min(candle.close_y) + origin_y;
        let body_rect = egui::Rect::from_min_size(
            egui::pos2(center_x - candle.body_width / 2.0, body_y),
            egui::vec2(candle.body_width, body_height),
        );
        shapes.push(egui::Shape::rect_filled(
            body_rect,
            egui::CornerRadius::same(1),
            color,
        ));

        let volume_color = egui::Color32::from_rgba_unmultiplied(
            volume.color.red,
            volume.color.green,
            volume.color.blue,
            volume.color.alpha,
        );
        let volume_rect = egui::Rect::from_min_size(
            egui::pos2(
                volume.center_x + origin.x as f32 - candle.body_width / 2.0,
                volume.base_y + origin_y - volume.height,
            ),
            egui::vec2(candle.body_width, volume.height),
        );
        shapes.push(egui::Shape::rect_filled(
            volume_rect,
            egui::CornerRadius::ZERO,
            volume_color,
        ));
    }

    shapes
}

fn should_rebuild_series(cache_valid: bool, reason: crate::renderer::InvalidationReason) -> bool {
    !cache_valid
        || !matches!(
            reason,
            crate::renderer::InvalidationReason::None
                | crate::renderer::InvalidationReason::CursorOnly
                | crate::renderer::InvalidationReason::Overlay
        )
}

impl ChartRenderer for EguiPainterChartRenderer {
    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        scene: &ChartScene<'_>,
    ) -> Result<(), RendererError> {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            tracing::warn!("Rectángulo de dibujo vacío, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        tracing::debug!(
            "Renderizando {} puntos con backend WGPU en rect {:?} (zoom_x={:.2}, zoom_y={:.2})",
            scene.data_points.len(),
            rect,
            scene.viewport.zoom_x(),
            scene.viewport.zoom_y()
        );

        if scene.data_points.is_empty() {
            self.cached_series_layer = RenderLayer::default();
            self.cached_series_shapes.clear();
            self.cached_gpu_vertices = Arc::from([]);
            self.series_cache_valid = false;
            return Ok(());
        }

        let origin = ScreenPoint {
            x: rect.left() as f64,
            y: rect.top() as f64,
        };
        let rebuild_series =
            should_rebuild_series(self.series_cache_valid, scene.invalidation_mask.level);
        let frame = RenderFrame::from_scene(
            scene,
            rect.width() as f64,
            rect.height() as f64,
            origin,
            rebuild_series,
        );
        let layout = &frame.layout;
        if !layout.is_valid() {
            tracing::warn!("Dimensiones de dibujo inválidas, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        let ox = frame.origin.x;
        let oy = frame.origin.y;
        let left = layout.plot.left + ox;
        let right = layout.plot.right + ox;
        let top = layout.plot.top + oy;
        let bottom = layout.plot.bottom + oy;
        let volume_bottom = layout.volume_panel.bottom + oy;
        let drawing_width = layout.plot_width();

        let visible = scene
            .viewport
            .visible_range(scene.candle_count, drawing_width);
        if visible.is_empty() {
            tracing::trace!("No hay puntos visibles en el viewport");
            return Ok(());
        }

        let price_params = PriceAxisParams {
            center_price: scene.center_price,
            offset_y: scene.viewport.offset_y(),
            price_range: scene.price_range,
            zoom_y: scene.viewport.zoom_y(),
            minimum_tick: scene.price_tick_size,
            scale_type: scene.viewport.price_scale_type(),
            inverted: scene.viewport.price_scale_inverted(),
            percentage_base: scene.viewport.price_scale_percentage_base(),
        };
        let price_ticks = price_axis_ticks(top, bottom, price_params);
        let time_params = TimeAxisParams {
            viewport: &scene.viewport,
            data_points: scene.data_points,
            logical_count: scene.candle_count,
        };
        let time_ticks = time_axis_ticks(left, right, &time_params);

        draw_chart_surfaces(painter, layout, frame.origin);
        draw_grid(painter, layout, frame.origin, &price_ticks, &time_ticks);

        if rebuild_series {
            self.cached_series_layer = frame.layers.first().cloned().unwrap_or_default();
            if self.target_format.is_some() {
                let geometry = build_geometry(&self.cached_series_layer);
                self.cached_gpu_vertices = ChartGpuCallback::vertices_from_geometry(
                    &geometry,
                    [rect.width(), rect.height()],
                );
                self.cached_series_shapes.clear();
            } else {
                self.cached_series_shapes =
                    series_layer_shapes(&self.cached_series_layer, frame.origin);
                self.cached_gpu_vertices = Arc::from([]);
            }
            self.series_cache_valid = true;
        }

        let series_clip = egui::Rect::from_min_max(
            egui::pos2(left as f32, top as f32),
            egui::pos2(right as f32, volume_bottom as f32),
        );
        let series_painter = painter.with_clip_rect(series_clip);
        if let Some(target_format) = self.target_format {
            if !self.cached_gpu_vertices.is_empty() {
                series_painter.add(ChartGpuCallback::shape(
                    self.callback_id,
                    rect,
                    self.cached_gpu_vertices.clone(),
                    target_format,
                    Arc::downgrade(&self.resource_owner),
                ));
            }
        } else {
            series_painter.extend(self.cached_series_shapes.iter().cloned());
        }

        let font_id = egui::FontId::proportional(11.0);
        let last_point = scene.data_points.last();
        let current_price = last_point.map(|point| {
            let color = if point.close() >= point.open() {
                egui::Color32::from_rgb(8, 153, 129)
            } else {
                egui::Color32::from_rgb(242, 54, 69)
            };
            (point.close(), color)
        });

        draw_price_axis(
            painter,
            layout,
            frame.origin,
            &price_ticks,
            price_params.minimum_tick,
            price_params.scale_type,
            font_id.clone(),
        );
        if let Some((price, color)) = current_price {
            draw_current_price_marker(painter, layout, frame.origin, price_params, price, color);
        }

        draw_volume_axis(
            painter,
            layout,
            frame.origin,
            VolumeAxisParams {
                max_volume: scene.max_volume,
            },
            font_id.clone(),
        );
        draw_time_axis(painter, layout, frame.origin, &time_ticks, font_id.clone());

        if let Some(crosshair) = scene.crosshair_pos {
            let chart_point = screen_to_chart_point(crosshair, layout.plot.left, layout.plot.top);
            if chart_point.x >= 0.0
                && chart_point.x <= layout.plot_width()
                && chart_point.y >= 0.0
                && chart_point.y <= layout.plot_height()
            {
                let crosshair_screen_x = crosshair.x + ox;
                let crosshair_screen_y = crosshair.y + oy;
                let crosshair_color = egui::Color32::from_rgb(104, 116, 132);
                let crosshair_clip = egui::Rect::from_min_max(
                    egui::pos2(left as f32, top as f32),
                    egui::pos2(right as f32, volume_bottom as f32),
                );
                let crosshair_painter = painter.with_clip_rect(crosshair_clip);
                crosshair_painter.line_segment(
                    [
                        egui::pos2(crosshair_screen_x as f32, top as f32),
                        egui::pos2(crosshair_screen_x as f32, volume_bottom as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );
                crosshair_painter.line_segment(
                    [
                        egui::pos2(left as f32, crosshair_screen_y as f32),
                        egui::pos2(right as f32, crosshair_screen_y as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                let price_scale_value = transforms::screen_y_to_price(
                    chart_point.y,
                    layout.plot_height(),
                    scene.price_range,
                    scene.viewport.zoom_y(),
                    scene.viewport.offset_y(),
                    scene.center_price,
                );
                let price_at_cursor = scene
                    .viewport
                    .scale_value_to_price(price_scale_value)
                    .unwrap_or_default();
                let logical_at_cursor = scene
                    .viewport
                    .screen_to_logical(chart_point.x, layout.plot_width());
                let time_label = time_label_at_logical(scene.data_points, logical_at_cursor);

                let price_axis_left = (layout.price_axis_right.left + ox) as f32;
                let price_axis_right = (layout.price_axis_right.right + ox) as f32;
                let price_marker_rect = egui::Rect::from_min_max(
                    egui::pos2(price_axis_left + 1.0, crosshair_screen_y as f32 - 9.0),
                    egui::pos2(price_axis_right - 1.0, crosshair_screen_y as f32 + 9.0),
                );
                painter.rect_filled(
                    price_marker_rect,
                    egui::CornerRadius::same(2),
                    egui::Color32::from_rgb(72, 84, 101),
                );
                painter.text(
                    price_marker_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    price_axis_label(
                        if price_params.scale_type == PriceScaleType::Percentage {
                            price_params
                                .scale_type
                                .to_scale_value(price_at_cursor, price_params.percentage_base)
                                .unwrap_or_default()
                        } else {
                            price_at_cursor
                        },
                        scene.price_tick_size,
                        scene.price_tick_size,
                        price_params.scale_type,
                    ),
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );

                let time_axis_left = (layout.time_axis.left + ox) as f32;
                let time_axis_right = (layout.time_axis.right + ox) as f32;
                let time_axis_width = time_axis_right - time_axis_left;
                let time_center = if time_axis_width >= 112.0 {
                    (crosshair_screen_x as f32).clamp(time_axis_left + 56.0, time_axis_right - 56.0)
                } else {
                    (time_axis_left + time_axis_right) * 0.5
                };
                let time_marker_rect = egui::Rect::from_min_max(
                    egui::pos2(time_center - 55.0, (layout.time_axis.top + oy + 3.0) as f32),
                    egui::pos2(
                        time_center + 55.0,
                        (layout.time_axis.bottom + oy - 3.0) as f32,
                    ),
                );
                painter.rect_filled(
                    time_marker_rect,
                    egui::CornerRadius::same(2),
                    egui::Color32::from_rgb(72, 84, 101),
                );
                painter.text(
                    time_marker_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    time_label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }
        }

        tracing::debug!("Renderizado con glow completado correctamente");
        Ok(())
    }

    fn name(&self) -> &str {
        if self.target_format.is_some() {
            "WGPU chart pipeline + egui UI"
        } else {
            "egui Painter fallback"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::InvalidationReason;

    #[test]
    fn price_ticks_share_the_center_origin_used_by_candles_and_crosshair() {
        let params = PriceAxisParams {
            center_price: 100.0,
            offset_y: 0.0,
            price_range: 20.0,
            zoom_y: 1.0,
            minimum_tick: 0.01,
            scale_type: PriceScaleType::Linear,
            inverted: false,
            percentage_base: None,
        };
        let ticks = price_axis_ticks(100.0, 300.0, params);
        let center = ticks
            .iter()
            .find(|tick| (tick.scale_value - 100.0).abs() < 1e-9)
            .expect("debe existir un tick en el precio central");
        assert!((center.y - 200.0).abs() < 1e-9);
    }

    #[test]
    fn logarithmic_price_ticks_are_geometrically_spaced_and_label_market_prices() {
        let params = PriceAxisParams {
            center_price: 10.0_f64.ln(),
            offset_y: 0.0,
            price_range: 100.0_f64.ln(),
            zoom_y: 1.0,
            minimum_tick: 0.01,
            scale_type: PriceScaleType::Logarithmic,
            inverted: false,
            percentage_base: None,
        };
        let ticks = price_axis_ticks(0.0, 400.0, params);

        assert!(ticks.len() >= 4);
        assert!(ticks.iter().any(|tick| (tick.value - 10.0).abs() < 1e-9));
        assert!(
            ticks
                .iter()
                .all(|tick| { (tick.scale_value - tick.value.ln()).abs() < 1e-10 })
        );
    }

    #[test]
    fn percentage_price_ticks_are_labeled_as_relative_changes() {
        let params = PriceAxisParams {
            center_price: 0.0,
            offset_y: 0.0,
            price_range: 20.0,
            zoom_y: 1.0,
            minimum_tick: 0.01,
            scale_type: PriceScaleType::Percentage,
            inverted: false,
            percentage_base: Some(100.0),
        };
        let ticks = price_axis_ticks(0.0, 400.0, params);

        assert!(ticks.len() >= 4);
        assert!(ticks.iter().any(|tick| tick.value.abs() < 1e-9));
        assert_eq!(
            price_axis_label(5.0, 1.0, 0.01, PriceScaleType::Percentage),
            "+5.00%"
        );
    }

    #[test]
    fn price_tick_precision_follows_the_tick_step() {
        assert_eq!(format_price_tick(50_123.0, 1.0, 0.01), "50123.00");
        assert_eq!(format_price_tick(5_000.25, 0.5, 0.25), "5000.25");
        assert_eq!(format_price_tick(1.08765, 0.0001, 0.00001), "1.08765");
        assert_eq!(
            format_price_tick(1.000000001, 0.000000001, 0.000000001),
            "1.000000001"
        );
        assert_eq!(
            format_price_tick(1.000000000001, 0.000000000001, 0.000000000001),
            "1.000000000001"
        );
    }

    fn minute_points(
        start_ms: i64,
        interval_ms: i64,
        count: usize,
    ) -> Vec<crate::domain::chart::chart_data::ChartDataPoint> {
        use crate::domain::candle::Candle;
        use crate::domain::chart::chart_data::ChartDataPoint;
        use crate::domain::chart::time_point::TimePoint;

        let candle = Candle::new(100.0, 101.0, 99.0, 100.5, 1_000.0).expect("vela válida");
        (0..count)
            .map(|index| {
                let open_time_ms = start_ms + index as i64 * interval_ms;
                let time_point =
                    TimePoint::new(open_time_ms, open_time_ms + interval_ms, interval_ms)
                        .expect("intervalo válido");
                let mut point = ChartDataPoint::new(candle, time_point);
                point.logical_index = Some(index);
                point
            })
            .collect()
    }

    #[test]
    fn time_axis_ticks_align_to_clock_intervals_instead_of_visible_index_origin() {
        let points = minute_points(9 * 3_600_000, 60_000, 60);
        let mut viewport = Viewport::new();
        viewport.set_offset_x(-29.5);
        let params = TimeAxisParams {
            viewport: &viewport,
            data_points: &points,
            logical_count: points.len(),
        };

        let ticks = time_axis_ticks(0.0, 800.0, &params);
        assert!(ticks.len() >= 3, "debe haber varias marcas temporales");
        for tick in ticks {
            let (hour, minute) = tick.label.split_once(':').expect("etiqueta horaria");
            assert!(hour.parse::<u32>().is_ok());
            let minute = minute.parse::<u32>().expect("minuto numérico");
            assert_eq!(
                minute % 10,
                0,
                "los ticks deben alinearse a múltiplos de diez minutos: {}",
                tick.label
            );
        }
    }

    #[test]
    fn time_axis_marks_calendar_day_changes_with_a_date_label() {
        const DAY_MS: i64 = 86_400_000;
        let points = minute_points(DAY_MS - 10 * 60_000, 60_000, 30);
        let mut viewport = Viewport::new();
        viewport.set_offset_x(-14.5);
        let params = TimeAxisParams {
            viewport: &viewport,
            data_points: &points,
            logical_count: points.len(),
        };

        let ticks = time_axis_ticks(0.0, 800.0, &params);
        assert!(
            ticks.iter().any(|tick| tick.label == "02/01"),
            "el primer tick del nuevo día debe mostrar la fecha: {ticks:?}"
        );
    }

    #[test]
    fn time_axis_ticks_remain_ordered_and_spaced_across_time_gaps() {
        let mut points = minute_points(9 * 3_600_000, 60_000, 20);
        let gap_start = 10 * 3_600_000;
        let gap_points = minute_points(gap_start, 60_000, 20);
        for (offset, mut point) in gap_points.into_iter().enumerate() {
            point.logical_index = Some(20 + offset);
            points.push(point);
        }

        let mut viewport = Viewport::new();
        viewport.set_offset_x(-19.5);
        let params = TimeAxisParams {
            viewport: &viewport,
            data_points: &points,
            logical_count: points.len(),
        };
        let ticks = time_axis_ticks(0.0, 800.0, &params);

        assert!(ticks.windows(2).all(|pair| {
            pair[0].screen_x < pair[1].screen_x && pair[1].screen_x - pair[0].screen_x >= 72.0
        }));
    }

    #[test]
    fn price_ticks_align_to_the_instrument_minimum_increment() {
        let params = PriceAxisParams {
            center_price: 100.0,
            offset_y: 0.0,
            price_range: 0.8,
            zoom_y: 1.0,
            minimum_tick: 0.25,
            scale_type: PriceScaleType::Linear,
            inverted: false,
            percentage_base: None,
        };
        let ticks = price_axis_ticks(0.0, 200.0, params);

        assert!(ticks.len() >= 3);
        assert!(ticks.iter().all(|tick| {
            let increments = tick.value / params.minimum_tick;
            (increments - increments.round()).abs() < 1e-8
        }));
    }

    #[test]
    fn crosshair_time_label_interpolates_irregular_time_intervals() {
        use crate::domain::candle::Candle;
        use crate::domain::chart::chart_data::ChartDataPoint;
        use crate::domain::chart::time_point::TimePoint;

        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();
        let points = [
            ChartDataPoint::new(candle, TimePoint::new(0, 60_000, 60_000).unwrap()),
            ChartDataPoint::new(
                candle,
                TimePoint::new(3_600_000, 3_660_000, 60_000).unwrap(),
            ),
            ChartDataPoint::new(
                candle,
                TimePoint::new(7_200_000, 7_260_000, 60_000).unwrap(),
            ),
        ];

        assert_eq!(time_label_at_logical(&points, 0.5), "01/01 00:30");
        assert_eq!(time_label_at_logical(&points, 1.5), "01/01 01:30");
    }

    #[test]
    fn crosshair_time_label_clamps_to_history_and_handles_invalid_input() {
        use crate::domain::candle::Candle;
        use crate::domain::chart::chart_data::ChartDataPoint;
        use crate::domain::chart::time_point::TimePoint;

        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0).unwrap();
        let points = [ChartDataPoint::new(
            candle,
            TimePoint::new(0, 60_000, 60_000).unwrap(),
        )];

        assert_eq!(time_label_at_logical(&points, -10.0), "01/01 00:00");
        assert_eq!(time_label_at_logical(&points, 10.0), "01/01 00:00");
        assert_eq!(time_label_at_logical(&points, f64::NAN), "");
        assert_eq!(time_label_at_logical(&[], 0.0), "");
    }

    #[test]
    fn series_cache_is_reused_for_unchanged_and_cursor_only_frames() {
        assert!(should_rebuild_series(false, InvalidationReason::None));
        assert!(!should_rebuild_series(true, InvalidationReason::None));
        assert!(!should_rebuild_series(true, InvalidationReason::CursorOnly));
        assert!(
            !should_rebuild_series(true, InvalidationReason::Overlay),
            "un overlay no modifica la geometría de velas ni volumen"
        );
    }

    #[test]
    fn series_cache_is_invalidated_by_structural_changes() {
        for reason in [
            InvalidationReason::SeriesData,
            InvalidationReason::ScaleLayout,
            InvalidationReason::Full,
        ] {
            assert!(
                should_rebuild_series(true, reason),
                "la invalidación {reason:?} debe regenerar la serie"
            );
        }
    }

    #[test]
    fn invalid_cache_is_built_even_without_an_invalidation_reason() {
        assert!(should_rebuild_series(false, InvalidationReason::None));
        assert!(should_rebuild_series(false, InvalidationReason::CursorOnly));
        assert!(should_rebuild_series(false, InvalidationReason::Overlay));
    }

    #[test]
    fn each_visible_candle_generates_wick_body_and_volume_shapes() {
        use crate::domain::candle::Candle;
        use crate::domain::chart::chart_data::ChartDataPoint;
        use crate::domain::chart::time_point::TimePoint;
        use crate::domain::viewport::Viewport;

        let time = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let candles = [
            ChartDataPoint::new(
                Candle::new(100.0, 105.0, 98.0, 103.0, 500.0).expect("vela alcista válida"),
                time,
            ),
            ChartDataPoint::new(
                Candle::new(103.0, 106.0, 99.0, 101.0, 1_000.0).expect("vela bajista válida"),
                time,
            ),
        ];
        let scene = ChartScene {
            data_points: &candles,
            viewport: Viewport::new(),
            price_range: 10.0,
            center_price: 102.0,
            price_tick_size: 0.01,
            max_volume: 1_000.0,
            candle_count: candles.len(),
            screen_width: 800.0,
            screen_height: 500.0,
            crosshair_pos: None,
            invalidation_mask: crate::renderer::InvalidationMask {
                level: InvalidationReason::Full,
            },
        };
        let mut renderer = EguiPainterChartRenderer::new();
        let context = egui::Context::default();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 500.0));

        let mut output = context.run_ui(egui::RawInput::default(), |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::unique("egui-painter-renderer-candle-cache"),
            ));
            renderer
                .draw(&painter, rect, &scene)
                .expect("escena válida");
        });

        output.textures_delta.clear();

        assert_eq!(
            renderer.cached_series_shapes.len(),
            candles.len() * 3,
            "cada vela visible debe generar wick, cuerpo y barra de volumen"
        );
        assert!(renderer.series_cache_valid);
    }

    #[test]
    fn empty_scene_clears_previously_cached_shapes() {
        let mut renderer = EguiPainterChartRenderer {
            callback_id: 0,
            resource_owner: Arc::new(()),
            cached_series_layer: RenderLayer::default(),
            cached_series_shapes: vec![egui::Shape::Noop],
            cached_gpu_vertices: Arc::from([]),
            target_format: None,
            series_cache_valid: true,
        };
        let context = egui::Context::default();
        let scene = ChartScene::empty();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 500.0));

        let mut output = context.run_ui(egui::RawInput::default(), |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::unique("egui-painter-renderer-empty-cache"),
            ));
            renderer
                .draw(&painter, rect, &scene)
                .expect("escena vacía válida");
        });

        output.textures_delta.clear();

        assert!(renderer.cached_series_shapes.is_empty());
        assert!(!renderer.series_cache_valid);
        assert!(should_rebuild_series(
            renderer.series_cache_valid,
            InvalidationReason::CursorOnly
        ));
    }
}
