use crate::domain::chart::time_scale::TimeScale;
use crate::domain::transforms::{self, ScreenPoint, screen_to_chart_point};
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use crate::renderer::{ChartRenderer, ChartScene, RenderColor, RenderFrame, RenderLayer};
use eframe::egui;
use rust_decimal::prelude::FromPrimitive;

/// Renderizador de chart usando el Painter de egui (backend glow/OpenGL).
///
/// Este es el renderer que funciona actualmente en tu GPU Intel UHD 630.
pub struct GlowChartRenderer {
    /// Shapes de velas y volumen calculadas en el último frame estructural.
    /// egui exige volver a emitirlas en cada frame; por eso se reproducen
    /// desde esta caché cuando solo cambia el crosshair.
    cached_series_layer: RenderLayer,
    cached_series_shapes: Vec<egui::Shape>,
    series_cache_valid: bool,
}

impl GlowChartRenderer {
    pub fn new() -> Self {
        Self {
            cached_series_layer: RenderLayer::default(),
            cached_series_shapes: Vec::new(),
            series_cache_valid: false,
        }
    }
}

impl Default for GlowChartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Obtiene la etiqueta temporal en una coordenada lógica fraccionaria.
///
/// Las posiciones entre velas se interpolan en tiempo real, incluso cuando
/// los intervalos entre timestamps son irregulares.
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
        return TimeScale::format_time_point(left.time_point);
    }

    let Some(right) = data_points.get(right_index) else {
        return String::new();
    };
    let fraction = bounded_index - left_index as f64;
    let start = left.time_point.open_time_ms as i128;
    let end = right.time_point.open_time_ms as i128;
    let timestamp = (start as f64 + (end - start) as f64 * fraction).round();

    if !timestamp.is_finite() || timestamp < i64::MIN as f64 || timestamp > i64::MAX as f64 {
        return String::new();
    }

    TimeScale::format_time_label(timestamp as i64)
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

struct TimeAxisParams<'a> {
    viewport: &'a Viewport,
    data_points: &'a [crate::domain::chart::chart_data::ChartDataPoint],
    logical_count: usize,
}

fn draw_time_axis(
    painter: &egui::Painter,
    left: f64,
    right: f64,
    bottom: f64,
    params: TimeAxisParams<'_>,
    font_id: egui::FontId,
) {
    let screen_width = right - left;
    let visible = params
        .viewport
        .visible_range(params.logical_count, screen_width);
    if visible.is_empty() {
        return;
    }

    let visible_count = visible.len();
    let index_step = if visible_count > 1 {
        ((visible_count as f64) / 6.0).ceil() as usize
    } else {
        1
    };

    let mut last_index = None;
    for point in params.data_points {
        let Some(index) = point.logical_index else {
            continue;
        };
        if index < visible.first
            || index > visible.last
            || !(index - visible.first).is_multiple_of(index_step)
            || last_index == Some(index)
        {
            continue;
        }

        let screen_x = params
            .viewport
            .logical_to_screen(index as f64, screen_width)
            + left;
        let label = TimeScale::format_time_point(point.time_point);
        painter.text(
            egui::pos2(screen_x as f32, (bottom + 6.0) as f32),
            egui::Align2::CENTER_TOP,
            label,
            font_id.clone(),
            egui::Color32::GRAY,
        );
        last_index = Some(index);
    }
}

/// Convierte la geometría compartida de la serie en primitivas específicas de egui.
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
            self.cached_series_layer = RenderLayer::default();
            self.cached_series_shapes.clear();
            self.series_cache_valid = true;
            tracing::trace!("No hay puntos para renderizar");
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

        if rebuild_series {
            self.cached_series_layer = frame.layers.first().cloned().unwrap_or_default();
            self.cached_series_shapes =
                series_layer_shapes(&self.cached_series_layer, frame.origin);
            self.series_cache_valid = true;
        }

        // egui limpia la lista de shapes en cada frame: reproducimos la caché
        // para mantener visible la serie sin recalcular cada vela.
        painter.extend(self.cached_series_shapes.iter().cloned());

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
            TimeAxisParams {
                viewport: &scene.viewport,
                data_points: scene.data_points,
                logical_count: scene.candle_count,
            },
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
                let crosshair_color = egui::Color32::from_rgba_unmultiplied(
                    RenderColor::ORANGE.red,
                    RenderColor::ORANGE.green,
                    RenderColor::ORANGE.blue,
                    RenderColor::ORANGE.alpha,
                );

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
                let logical_at_cursor = scene
                    .viewport
                    .screen_to_logical(chart_point.x, layout.plot_width());
                let time_label = time_label_at_logical(scene.data_points, logical_at_cursor);

                let price_label = format!("{:.2}", price_at_cursor);
                painter.text(
                    egui::pos2((left - 6.0) as f32, crosshair_screen_y as f32),
                    egui::Align2::RIGHT_CENTER,
                    price_label,
                    egui::FontId::proportional(11.0),
                    crosshair_color,
                );

                painter.text(
                    egui::pos2(crosshair_screen_x as f32, (bottom_precios + 4.0) as f32),
                    egui::Align2::CENTER_TOP,
                    time_label,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::InvalidationReason;

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

        assert_eq!(time_label_at_logical(&points, 0.5), "01/01 00:30 UTC");
        assert_eq!(time_label_at_logical(&points, 1.5), "01/01 01:30 UTC");
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

        assert_eq!(time_label_at_logical(&points, -10.0), "01/01 00:00 UTC");
        assert_eq!(time_label_at_logical(&points, 10.0), "01/01 00:00 UTC");
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
            max_volume: 1_000.0,
            candle_count: candles.len(),
            screen_width: 800.0,
            screen_height: 500.0,
            crosshair_pos: None,
            invalidation_mask: crate::renderer::InvalidationMask {
                level: InvalidationReason::Full,
            },
        };
        let mut renderer = GlowChartRenderer::new();
        let context = egui::Context::default();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 500.0));

        let mut output = context.run_ui(egui::RawInput::default(), |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::unique("glow-renderer-candle-cache"),
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
        let mut renderer = GlowChartRenderer {
            cached_series_layer: RenderLayer::default(),
            cached_series_shapes: vec![egui::Shape::Noop],
            series_cache_valid: true,
        };
        let context = egui::Context::default();
        let scene = ChartScene::empty();
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 500.0));

        let mut output = context.run_ui(egui::RawInput::default(), |ctx| {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::unique("glow-renderer-empty-cache"),
            ));
            renderer
                .draw(&painter, rect, &scene)
                .expect("escena vacía válida");
        });

        output.textures_delta.clear();

        assert!(renderer.cached_series_shapes.is_empty());
        assert!(renderer.series_cache_valid);
    }
}
