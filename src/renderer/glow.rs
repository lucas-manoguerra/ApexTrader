use crate::chart::layout::ChartLayout;
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::transforms::{self, ScreenPoint, screen_to_chart_point};
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use crate::renderer::ChartRenderer;
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

/// Parámetros para dibujar el eje de volumen.
struct VolumeAxisParams {
    /// Volumen máximo visible (para escalar las barras).
    max_volume: f64,
    /// Nivel de zoom del eje Y del volumen (1.0 = escala base).
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

    // Rango de precios visible en el área de dibujo.
    let half_range_visible = params.price_range / (2.0 * params.zoom_y);
    let min_visible =
        params.center_price + params.offset_y * params.price_range - half_range_visible;
    let max_visible =
        params.center_price + params.offset_y * params.price_range + half_range_visible;

    let tick_step = nice_tick_step(max_visible - min_visible, 6);

    // Primer tick >= min_visible.
    let mut tick = (min_visible / tick_step).ceil() * tick_step;

    while tick <= max_visible {
        let relative_price = tick - params.center_price - params.offset_y * params.price_range;
        let pixels_per_unit = screen_height / params.price_range * params.zoom_y;
        let y = bottom - relative_price * pixels_per_unit;

        // Formatear precio con 2 decimales fijos.
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

    // Ticks dinámicos para el volumen (máximo 6 ticks).
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
    viewport: &crate::domain::viewport::Viewport,
    candles_count: usize,
    font_id: egui::FontId,
) {
    let screen_width = right - left;

    // Usa el mismo VisibleRange que el renderer de velas (AT-004):
    // una única fuente de verdad para qué velas están visibles.
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
        points: &[ChartDataPoint],
        viewport: &Viewport,
        crosshair_pos: Option<ScreenPoint>,
    ) -> Result<(), RendererError> {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            tracing::warn!("Rectángulo de dibujo vacío, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        tracing::debug!(
            "Renderizando {} puntos con glow en rect {:?} (zoom_x={:.2}, zoom_y={:.2})",
            points.len(),
            rect,
            viewport.zoom_x(),
            viewport.zoom_y()
        );

        if points.is_empty() {
            tracing::trace!("No hay puntos para renderizar");
            return Ok(());
        }

        // Layout único y autoritativo (AT-002): input y renderer consumen
        // el mismo ChartLayout, eliminando cálculos de margen hardcodeados.
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

        // Rango visible primero: solo las velas que están en el viewport.
        // Esto evita el escaneo O(N_total) de todo el histórico por frame
        // (AT-005): min/max/volumen se calculan solo sobre lo visible.
        let drawing_width = right - left;
        let visible = viewport.visible_range(points.len(), drawing_width);

        if visible.is_empty() {
            tracing::trace!("No hay puntos visibles en el viewport");
            return Ok(());
        }

        // Calculamos rangos de precio y volumen SOLO sobre puntos visibles:
        // O(N_visible) en lugar de O(N_total).
        let visible_points = &points[visible.first..=visible.last];
        let min_price = visible_points
            .iter()
            .map(|p| p.low())
            .fold(f64::INFINITY, f64::min);
        let max_price = visible_points
            .iter()
            .map(|p| p.high())
            .fold(f64::NEG_INFINITY, f64::max);
        let mut price_range = max_price - min_price;
        if price_range == 0.0 {
            price_range = 1.0;
        }

        // Volumen máximo para escalar las barras del panel de volumen (solo visibles).
        let max_volume = visible_points
            .iter()
            .map(|p| p.volume())
            .fold(0.0f64, f64::max);

        // Centro de precios para el eje Y (punto alrededor del cual se hace zoom).
        let center_price = (min_price + max_price) / 2.0;

        tracing::trace!(
            "Rango de precios (visible): min={:.2}, max={:.2}, range={:.2}, center={:.2}, \
             max_volumen={:.2}, velas={}..={}",
            min_price,
            max_price,
            price_range,
            center_price,
            max_volume,
            visible.first,
            visible.last
        );

        // Espaciamiento entre velas ajustado por zoom.
        let candle_width = transforms::candle_body_width(viewport.zoom_x()) as f32;
        let candle_width = candle_width.clamp(4.0, 40.0);
        let _spacing = candle_width * 0.4;

        // Helpers: convierte precio f64 → píxel f32 en el eje Y invertido.
        // Aplica zoom_y y offset_y del viewport, dentro del área de precios.
        let price_to_y = |price: f64| -> f32 {
            let bottom = bottom_precios;
            let screen_height = bottom - top;
            let pixels_per_unit = screen_height / price_range * viewport.zoom_y();
            let relative_price = price - center_price - viewport.offset_y() * price_range;
            (bottom - relative_price * pixels_per_unit) as f32
        };

        // Helper: convierte volumen f64 → altura f32 dentro del panel de volumen.
        let volume_to_height = |volume: f64| -> f32 {
            if max_volume <= 0.0 || !max_volume.is_finite() {
                return 0.0;
            }
            let screen_height = volumen_bottom - volumen_top;
            let relative = volume / max_volume;
            (relative * screen_height) as f32
        };

        // Dibujamos solo los puntos visibles: O(N_visible) por frame en lugar
        // de O(N_total).
        for (i, point) in points
            .iter()
            .enumerate()
            .skip(visible.first)
            .take(visible.last - visible.first + 1)
        {
            // Posición X ajustada por zoom y pan del viewport.
            let screen_x = viewport.data_x_to_screen(i, drawing_width) + left;

            let y_high = price_to_y(point.high());
            let y_low = price_to_y(point.low());
            let y_open = price_to_y(point.open());
            let y_close = price_to_y(point.close());

            // Color del cuerpo: verde si subió, rojo si bajó.
            let color = if point.close() >= point.open() {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            };

            // Mécho (wick): línea vertical del mínimo al máximo.
            painter.line_segment(
                [
                    egui::pos2(screen_x as f32, y_high),
                    egui::pos2(screen_x as f32, y_low),
                ],
                egui::Stroke::new(1.0, color),
            );

            // Cuerpo: rectángulo del precio de apertura al de cierre.
            let body_height = (y_close - y_open).abs().max(2.0);
            let body_y = y_open.min(y_close);
            let body_rect = egui::Rect::from_min_size(
                egui::pos2(screen_x as f32 - candle_width / 2.0, body_y),
                egui::vec2(candle_width, body_height),
            );

            painter.rect_filled(body_rect, 1.0, color);

            // Volumen: barra debajo del chart de precios.
            let volume_height = volume_to_height(point.volume());
            let volume_y = volumen_bottom - volume_height as f64;
            let volume_rect = egui::Rect::from_min_size(
                egui::pos2(screen_x as f32 - candle_width / 2.0, volume_y as f32),
                egui::vec2(candle_width, volume_height.max(1.0)),
            );
            painter.rect_filled(volume_rect, 0.0, color);
        }

        // Eje de precios (Y): ticks dinámicos ajustados al zoom.
        let font_id = egui::FontId::proportional(11.0);
        let n = points.len();
        let left_f64 = left;
        let right_f64 = right;
        let top_f64 = top;
        // Eje de precios (Y): ticks dinámicos ajustados al zoom.
        draw_price_axis(
            painter,
            left_f64,
            top_f64,
            bottom_precios,
            PriceAxisParams {
                center_price,
                offset_y: viewport.offset_y(),
                price_range,
                zoom_y: viewport.zoom_y(),
            },
            font_id.clone(),
        );

        // Eje de volumen (Y): ticks dinámicos del panel de volumen.
        draw_volume_axis(
            painter,
            left_f64,
            volumen_top,
            volumen_bottom,
            VolumeAxisParams {
                max_volume,
                zoom_y: viewport.zoom_y(),
            },
            font_id.clone(),
        );

        // Eje de tiempo (X): ticks dinámicos de índice de vela ajustados al zoom.
        draw_time_axis(
            painter,
            left_f64,
            right_f64,
            bottom_precios,
            viewport,
            n,
            font_id,
        );

        // --- Crosshair: líneas vertical y horizontal en la posición del cursor ---
        if let Some(crosshair) = crosshair_pos {
            let left_f64 = left;
            let right_f64 = right;
            let top_f64 = top;
            let bottom_f64 = bottom_precios;

            // Solo dibujar si el cursor está dentro del área de dibujo útil de precios.
            // crosshair está en ScreenPoint (relativo al rect completo), mismo espacio
            // que left/right/top/bottom_precios, así que la comparación es consistente.
            if crosshair.x >= left_f64
                && crosshair.x <= right_f64
                && crosshair.y >= top_f64
                && crosshair.y <= bottom_f64
            {
                let crosshair_color = egui::Color32::from_rgb(255, 165, 0);

                // Línea vertical (sobre el área de precios).
                painter.line_segment(
                    [
                        egui::pos2(crosshair.x as f32, top_f64 as f32),
                        egui::pos2(crosshair.x as f32, bottom_f64 as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                // Línea horizontal (sobre el área de precios).
                painter.line_segment(
                    [
                        egui::pos2(left as f32, crosshair.y as f32),
                        egui::pos2(right as f32, crosshair.y as f32),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                // Calcular precio y índice bajo el cursor.
                let screen_width_drawing = right_f64 - left_f64;
                let screen_height_drawing = bottom_f64 - top_f64;

                // Convertir ScreenPoint (rect completo) a ChartPoint (área útil),
                // restando los márgenes de ejes (left/top).
                let chart_point = screen_to_chart_point(crosshair, left_f64, top_f64);

                // Precio y índice bajo el cursor, usando transformaciones con f64
                // sin reconstruir objetos de dominio por frame (evita expect() en el
                // camino de renderizado).
                let price_at_cursor = transforms::screen_y_to_price(
                    chart_point.y,
                    screen_height_drawing,
                    price_range,
                    viewport.zoom_y(),
                    viewport.offset_y(),
                    center_price,
                );
                let index_at_cursor =
                    viewport.screen_x_to_data(chart_point.x, screen_width_drawing, points.len());

                // Etiqueta de precio (lado izquierdo, alineada con la línea horizontal).
                let price_label = format!("{:.2}", price_at_cursor);
                painter.text(
                    egui::pos2((left - 6.0) as f32, crosshair.y as f32),
                    egui::Align2::RIGHT_CENTER,
                    price_label,
                    egui::FontId::proportional(11.0),
                    crosshair_color,
                );

                // Etiqueta de índice de vela (fondo, alineada con la línea vertical).
                let index_label = format!("{}", index_at_cursor);
                painter.text(
                    egui::pos2(crosshair.x as f32, (bottom_f64 + 4.0) as f32),
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
