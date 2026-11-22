use crate::domain::candle::Candle;
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
    zoom_level: f64,
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
    let half_range_visible = params.price_range / (2.0 * params.zoom_level);
    let min_visible =
        params.center_price + params.offset_y * params.price_range - half_range_visible;
    let max_visible =
        params.center_price + params.offset_y * params.price_range + half_range_visible;

    let tick_step = nice_tick_step(max_visible - min_visible, 6);

    // Primer tick >= min_visible.
    let mut tick = (min_visible / tick_step).ceil() * tick_step;

    while tick <= max_visible {
        let relative_price = tick - params.center_price - params.offset_y * params.price_range;
        let pixels_per_unit = screen_height / params.price_range * params.zoom_level;
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
    let candle_spacing = transforms::candle_spacing(viewport.zoom_level());

    let first_visible = (-viewport.offset_x()).max(0.0) as i32;
    let last_visible = ((-viewport.offset_x()) + screen_width / candle_spacing)
        .min(candles_count as f64 - 1.0) as i32;

    if first_visible > last_visible {
        return;
    }

    let visible_range = last_visible - first_visible;
    let index_step = if visible_range > 0 {
        ((visible_range as f64) / 6.0).ceil() as i32
    } else {
        1
    };

    let mut idx = first_visible;
    while idx <= last_visible {
        let screen_x = viewport.data_x_to_screen(idx as usize, screen_width) + left;
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
        candles: &[Candle],
        viewport: &Viewport,
        crosshair_pos: Option<ScreenPoint>,
    ) -> Result<(), RendererError> {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            tracing::warn!("Rectángulo de dibujo vacío, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        tracing::debug!(
            "Renderizando {} velas con glow en rect {:?} (zoom={:.2})",
            candles.len(),
            rect,
            viewport.zoom_level()
        );

        if candles.is_empty() {
            tracing::trace!("No hay velas para renderizar");
            return Ok(());
        }

        let left = rect.left() + 40.0; // espacio para el eje de precios
        let right = rect.right() - 20.0;
        let top = rect.top() + 40.0; // espacio para título
        let bottom = rect.bottom() - 40.0; // espacio para el eje de tiempo

        // Calculamos rangos de precio para escalar (usamos f64 para precisión).
        let min_price = candles
            .iter()
            .map(|c| c.low())
            .fold(f64::INFINITY, f64::min);
        let max_price = candles
            .iter()
            .map(|c| c.high())
            .fold(f64::NEG_INFINITY, f64::max);
        let mut price_range = max_price - min_price;
        if price_range == 0.0 {
            price_range = 1.0;
        }

        // Centro de precios para el eje Y (punto alrededor del cual se hace zoom).
        let center_price = (min_price + max_price) / 2.0;

        tracing::trace!(
            "Rango de precios: min={:.2}, max={:.2}, range={:.2}, center={:.2}",
            min_price,
            max_price,
            price_range,
            center_price
        );

        let drawing_width = (right - left) as f64;
        let visible = viewport.visible_range(candles.len(), drawing_width);

        // Espaciamiento entre velas ajustado por zoom.
        let candle_width = transforms::candle_body_width(viewport.zoom_level()) as f32;
        let candle_width = candle_width.clamp(4.0, 40.0);
        let _spacing = candle_width * 0.4;

        // Helpers: convierte precio f64 → píxel f32 en el eje Y invertido.
        // Aplica zoom_level y offset_y del viewport.
        let price_to_y = |price: f64| -> f32 {
            let bottom = bottom as f64;
            let top = top as f64;
            let screen_height = bottom - top;
            let pixels_per_unit = screen_height / price_range * viewport.zoom_level();
            let relative_price = price - center_price - viewport.offset_y() * price_range;
            (bottom - relative_price * pixels_per_unit) as f32
        };

        // Dibujamos solo las velas visibles: O(N_visible) por frame en lugar
        // de O(N_total).
        for (i, candle) in candles
            .iter()
            .enumerate()
            .skip(visible.first)
            .take(visible.last - visible.first + 1)
        {
            // Posición X ajustada por zoom y pan del viewport.
            let screen_x = viewport.data_x_to_screen(i, drawing_width) + left as f64;

            let y_high = price_to_y(candle.high());
            let y_low = price_to_y(candle.low());
            let y_open = price_to_y(candle.open());
            let y_close = price_to_y(candle.close());

            // Color del cuerpo: verde si subió, rojo si bajó.
            let color = if candle.close() >= candle.open() {
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
        }

        // Eje de precios (Y): ticks dinámicos ajustados al zoom.
        let font_id = egui::FontId::proportional(11.0);
        let n = candles.len();
        let left_f64 = left as f64;
        let right_f64 = right as f64;
        let top_f64 = top as f64;
        let bottom_f64 = bottom as f64;
        draw_price_axis(
            painter,
            left_f64,
            top_f64,
            bottom_f64,
            PriceAxisParams {
                center_price,
                offset_y: viewport.offset_y(),
                price_range,
                zoom_level: viewport.zoom_level(),
            },
            font_id.clone(),
        );

        // Eje de tiempo (X): ticks dinámicos de índice de vela ajustados al zoom.
        draw_time_axis(
            painter, left_f64, right_f64, bottom_f64, viewport, n, font_id,
        );

        // --- Crosshair: líneas vertical y horizontal en la posición del cursor ---
        if let Some(crosshair) = crosshair_pos {
            let left_f64 = left as f64;
            let right_f64 = right as f64;
            let top_f64 = top as f64;
            let bottom_f64 = bottom as f64;

            // Solo dibujar si el cursor está dentro del área de dibujo útil.
            // crosshair está en ScreenPoint (relativo al rect completo), mismo espacio
            // que left/right/top/bottom, así que la comparación es consistente.
            if crosshair.x >= left_f64
                && crosshair.x <= right_f64
                && crosshair.y >= top_f64
                && crosshair.y <= bottom_f64
            {
                let crosshair_color = egui::Color32::from_rgb(255, 165, 0);

                // Línea vertical.
                painter.line_segment(
                    [
                        egui::pos2(crosshair.x as f32, top),
                        egui::pos2(crosshair.x as f32, bottom),
                    ],
                    egui::Stroke::new(1.0, crosshair_color),
                );

                // Línea horizontal.
                painter.line_segment(
                    [
                        egui::pos2(left, crosshair.y as f32),
                        egui::pos2(right, crosshair.y as f32),
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
                    viewport.zoom_level(),
                    viewport.offset_y(),
                    center_price,
                );
                let index_at_cursor =
                    viewport.screen_x_to_data(chart_point.x, screen_width_drawing, candles.len());

                // Etiqueta de precio (lado izquierdo, alineada con la línea horizontal).
                let price_label = format!("{:.2}", price_at_cursor);
                painter.text(
                    egui::pos2(left - 6.0, crosshair.y as f32),
                    egui::Align2::RIGHT_CENTER,
                    price_label,
                    egui::FontId::proportional(11.0),
                    crosshair_color,
                );

                // Etiqueta de índice de vela (fondo, alineada con la línea vertical).
                let index_label = format!("{}", index_at_cursor);
                painter.text(
                    egui::pos2(crosshair.x as f32, bottom + 4.0),
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
