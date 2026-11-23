use crate::chart::layout::ChartLayout;
use crate::domain::transforms::{self, screen_to_chart_point};
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use crate::renderer::{ChartRenderer, ChartScene};
use eframe::egui;
use rust_decimal::prelude::FromPrimitive;

/// Renderizador de chart usando el Painter de egui (backend glow/OpenGL).
///
/// Este es el renderer que funciona actualmente en tu GPU Intel UHD 630.
pub struct GlowChartRenderer;

impl GlowChartRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GlowChartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn nice_tick_step(range: f64, target_ticks: usize) -> f64 {
    if range <= 0.0 {
        return 1.0;
    }
    let raw_step = range / target_ticks as f64;
    let magnitude = 10.0f64.powi(raw_step.log10().floor() as i32);
    let residual = raw_step / magnitude;
    if residual < 1.5 {
        1.0 * magnitude
    } else if residual < 3.0 {
        2.0 * magnitude
    } else if residual < 7.0 {
        5.0 * magnitude
    } else {
        10.0 * magnitude
    }
}

struct PriceAxisParams {
    center_price: f64,
    offset_y: f64,
    price_range: f64,
    zoom_y: f64,
}

struct VolumeAxisParams {
    max_volume: f64,
    zoom_y: f64,
}

fn draw_price_axis(
    painter: &egui::Painter,
    left: f64,
    top: f64,
    bottom: f64,
    params: PriceAxisParams,
    font_id: egui::FontId,
) {
    let screen_height = bottom - top;

    let half_range_visible = params.price_range / (2.0 * params.zoom_y);
    let min_visible =
        params.center_price + params.offset_y * params.price_range - half_range_visible;
    let max_visible =
        params.center_price + params.offset_y * params.price_range + half_range_visible;

    let tick_step = nice_tick_step(max_visible - min_visible, 6);

    let mut tick = (min_visible / tick_step).ceil() * tick_step;

    while tick <= max_visible {
        let relative_price = tick - params.center_price - params.offset_y * params.price_range;
        let pixels_per_unit = screen_height / params.price_range * params.zoom_y;
        let y = bottom - relative_price * pixels_per_unit;

        let price_decimal =
            rust_decimal::Decimal::from_f64(tick).unwrap_or(rust_decimal::Decimal::ZERO);
        let price = crate::domain::price::Price::new(price_decimal).unwrap_or_else(|_| {
            crate::domain::price::Price::new(rust_decimal::Decimal::ZERO).unwrap()
        });
        let label = format!("{:.2}", price.value());

        painter.text(
            egui::pos2((left - 8.0) as f32, y as f32),
            egui::Align2::RIGHT_CENTER,
            label,
            font_id.clone(),
            egui::Color32::GRAY,
        );

        tick += tick_step;
    }
}

fn draw_volume_axis(
    painter: &egui::Painter,
    left: f64,
    top: f64,
    bottom: f64,
    params: VolumeAxisParams,
    font_id: egui::FontId,
) {
    let screen_height = bottom - top;

    if params.max_volume <= 0.0 || !params.max_volume.is_finite() {
        return;
    }

    let tick_step = nice_tick_step(params.max_volume, 6);

    let mut tick = 0.0;
    while tick <= params.max_volume {
        let relative = tick / params.max_volume;
        let y = bottom - relative * screen_height;

        let label = if tick >= 1_000_000.0 {
            format!("{:.1}M", tick / 1_000_000.0)
        } else if tick >= 1_000.0 {
            format!("{:.1}K", tick / 1_000.0)
        } else {
            format!("{:.0}", tick)
        };

        painter.text(
            egui::pos2((left - 8.0) as f32, y as f32),
            egui::Align2::RIGHT_CENTER,
            label,
            font_id.clone(),
            egui::Color32::GRAY,
        );

        tick += tick_step;
    }
}

fn draw_time_axis(
    painter: &egui::Painter,
    left: f64,
    right: f64,
    bottom: f64,
    viewport: &Viewport,
    candles_count: usize,
    font_id: egui::FontId,
) {
    let screen_width = right - left;

    let visible = viewport.visible_range(candles_count, screen_width);
    if visible.is_empty() {
        return;
    }

    let visible_count = visible.last - visible.first + 1;
    let index_step = if visible_count > 1 {
        ((visible_count as f64) / 6.0).ceil() as usize
    } else {
        1
    };

    let mut idx = visible.first;
    while idx <= visible.last {
        let screen_x = viewport.data_x_to_screen(idx, screen_width) + left;
        let label = format!("{}", idx);

        painter.text(
            egui::pos2(screen_x as f32, (bottom + 6.0) as f32),
            egui::Align2::CENTER_TOP,
            label,
            font_id.clone(),
            egui::Color32::GRAY,
        );

        idx += index_step;
    }
}

impl ChartRenderer for GlowChartRenderer {
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
            "Renderizando {} puntos con glow en rect {:?} (zoom_x={:.2}, zoom_y={:.2})",
            scene.data_points.len(),
            rect,
            scene.viewport.zoom_x(),
            scene.viewport.zoom_y()
        );

        if scene.data_points.is_empty() {
            tracing::trace!("No hay puntos para renderizar");
            return Ok(());
        }

        let layout = ChartLayout::compute(rect.width() as f64, rect.height() as f64);
        if !layout.is_valid() {
            tracing::warn!("Dimensiones de dibujo inválidas, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        let ox = rect.left() as f64;
        let oy = rect.top() as f64;

        let left = layout.plot.left + ox;
        let right = layout.plot.right + ox;
        let top = layout.plot.top + oy;
        let bottom_precios = layout.plot.bottom + oy;
        let volumen_top = layout.volume_panel.top + oy;
        let volumen_bottom = layout.volume_panel.bottom + oy;

        let drawing_width = right - left;
        let visible = scene
            .viewport
            .visible_range(scene.candle_count, drawing_width);

        if visible.is_empty() {
            tracing::trace!("No hay puntos visibles en el viewport");
            return Ok(());
        }

        // La aplicación ya preparó estas métricas en una sola pasada
        // sobre el rango visible; el renderer no vuelve a recorrer las velas.
        let price_range = scene.price_range;
        let center_price = scene.center_price;
        let max_volume = scene.max_volume;

        tracing::trace!(
            "Rango de precios (visible): range={:.2}, center={:.2}, \
             max_volumen={:.2}, velas={}..={}",
            price_range,
            center_price,
            max_volume,
            visible.first,
            visible.last
        );

        let candle_width = transforms::candle_body_width(scene.viewport.zoom_x()) as f32;
        let candle_width = candle_width.clamp(4.0, 40.0);
        let _spacing = candle_width * 0.4;

        let price_to_y = |price: f64| -> f32 {
            let bottom = bottom_precios;
            let screen_height = bottom - top;
            let pixels_per_unit = screen_height / price_range * scene.viewport.zoom_y();
            let relative_price = price - center_price - scene.viewport.offset_y() * price_range;
            (bottom - relative_price * pixels_per_unit) as f32
        };

        let volume_to_height = |volume: f64| -> f32 {
            if max_volume <= 0.0 || !max_volume.is_finite() {
                return 0.0;
            }
            let screen_height = volumen_bottom - volumen_top;
            let relative = volume / max_volume;
            (relative * screen_height) as f32
        };

        for (i, point) in scene
            .data_points
            .iter()
            .enumerate()
            .skip(visible.first)
            .take(visible.last - visible.first + 1)
        {
            let screen_x = scene.viewport.data_x_to_screen(i, drawing_width) + left;

            let y_high = price_to_y(point.high());
            let y_low = price_to_y(point.low());
            let y_open = price_to_y(point.open());
            let y_close = price_to_y(point.close());

            let color = if point.close() >= point.open() {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            };

            painter.line_segment(
                [
                    egui::pos2(screen_x as f32, y_high),
                    egui::pos2(screen_x as f32, y_low),
                ],
                egui::Stroke::new(1.0, color),
            );

            let body_height = (y_close - y_open).abs().max(2.0);
            let body_y = y_open.min(y_close);
            let body_rect = egui::Rect::from_min_size(
                egui::pos2(screen_x as f32 - candle_width / 2.0, body_y),
                egui::vec2(candle_width, body_height),
            );

            painter.rect_filled(body_rect, 1.0, color);

            let volume_height = volume_to_height(point.volume());
            let volume_y = volumen_bottom - volume_height as f64;
            let volume_rect = egui::Rect::from_min_size(
                egui::pos2(screen_x as f32 - candle_width / 2.0, volume_y as f32),
                egui::vec2(candle_width, volume_height.max(1.0)),
            );
            painter.rect_filled(volume_rect, 0.0, color);
        }

        let font_id = egui::FontId::proportional(11.0);
        let left_f64 = left;
        let right_f64 = right;
        let top_f64 = top;

        draw_price_axis(
            painter,
            left_f64,
            top_f64,
            bottom_precios,
            PriceAxisParams {
                center_price,
                offset_y: scene.viewport.offset_y(),
                price_range,
                zoom_y: scene.viewport.zoom_y(),
            },
            font_id.clone(),
        );

        draw_volume_axis(
            painter,
            left_f64,
            volumen_top,
            volumen_bottom,
            VolumeAxisParams {
                max_volume,
                zoom_y: scene.viewport.zoom_y(),
            },
            font_id.clone(),
        );

        draw_time_axis(
            painter,
            left_f64,
            right_f64,
            bottom_precios,
            &scene.viewport,
            scene.candle_count,
            font_id,
        );

        if let Some(crosshair) = scene.crosshair_pos {
            // ScreenPoint se expresa relativo al origen del rect completo del
            // chart; ChartLayout también usa coordenadas locales al rect.
            let chart_point = screen_to_chart_point(crosshair, layout.plot.left, layout.plot.top);

            if chart_point.x >= 0.0
                && chart_point.x <= layout.plot_width()
                && chart_point.y >= 0.0
                && chart_point.y <= layout.plot_height()
            {
                let crosshair_screen_x = crosshair.x + ox;
                let crosshair_screen_y = crosshair.y + oy;
                let crosshair_color = egui::Color32::from_rgb(255, 165, 0);

                painter.line_segment(
                    [
                        egui::pos2(crosshair_screen_x as f32, top as f32),
                        egui::pos2(crosshair_screen_x as f32, bottom_precios as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                painter.line_segment(
                    [
                        egui::pos2(left as f32, crosshair_screen_y as f32),
                        egui::pos2(right as f32, crosshair_screen_y as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                let price_at_cursor = transforms::screen_y_to_price(
                    chart_point.y,
                    layout.plot_height(),
                    price_range,
                    scene.viewport.zoom_y(),
                    scene.viewport.offset_y(),
                    center_price,
                );
                let index_at_cursor = scene.viewport.screen_x_to_data(
                    chart_point.x,
                    layout.plot_width(),
                    scene.candle_count,
                );

                let price_label = format!("{:.2}", price_at_cursor);
                painter.text(
                    egui::pos2((left - 6.0) as f32, crosshair_screen_y as f32),
                    egui::Align2::RIGHT_CENTER,
                    price_label,
                    egui::FontId::proportional(11.0),
                    crosshair_color,
                );

                let index_label = format!("{}", index_at_cursor);
                painter.text(
                    egui::pos2(crosshair_screen_x as f32, (bottom_precios + 4.0) as f32),
                    egui::Align2::CENTER_TOP,
                    index_label,
                    egui::FontId::proportional(11.0),
                    crosshair_color,
                );
            }
        }

        tracing::debug!("Renderizado con glow completado correctamente");
        Ok(())
    }

    fn name(&self) -> &str {
        "glow (egui Painter)"
    }
}
