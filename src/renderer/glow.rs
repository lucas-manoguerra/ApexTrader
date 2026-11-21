use crate::domain::candle::Candle;
use crate::domain::viewport::Viewport;
use crate::errors::RendererError;
use crate::renderer::ChartRenderer;
use eframe::egui;

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

impl ChartRenderer for GlowChartRenderer {
    fn draw(
        &mut self,
        painter: &egui::Painter,
        rect: egui::Rect,
        candles: &[Candle],
        viewport: &Viewport,
    ) -> Result<(), RendererError> {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            tracing::warn!("Rectángulo de dibujo vacío, no se puede renderizar");
            return Err(RendererError::EmptyDrawRect);
        }

        tracing::debug!(
            "Renderizando {} velas con glow en rect {:?} (zoom={:.2})",
            candles.len(),
            rect,
            viewport.zoom_level
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
        let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
        let max_price = candles
            .iter()
            .map(|c| c.high)
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

        let n = candles.len();

        // Espaciamiento entre velas ajustado por zoom.
        let base_candle_width = 12.0;
        let candle_width = base_candle_width * viewport.zoom_level as f32;
        let candle_width = candle_width.clamp(4.0, 40.0);
        let _spacing = candle_width * 0.4;

        // Helpers: convierte precio f64 → píxel f32 en el eje Y invertido.
        // Aplica zoom_level y offset_y del viewport.
        let price_to_y = |price: f64| -> f32 {
            let bottom = bottom as f64;
            let top = top as f64;
            let screen_height = bottom - top;
            let pixels_per_unit = screen_height / price_range * viewport.zoom_level;
            let relative_price = price - center_price - viewport.offset_y * price_range;
            (bottom - relative_price * pixels_per_unit) as f32
        };

        // Dibujamos cada vela.
        for (i, candle) in candles.iter().enumerate() {
            // Posición X ajustada por zoom y pan del viewport.
            let screen_x = viewport.data_x_to_screen(i as f64, (right - left) as f64) + left as f64;

            let y_high = price_to_y(candle.high);
            let y_low = price_to_y(candle.low);
            let y_open = price_to_y(candle.open);
            let y_close = price_to_y(candle.close);

            // Color del cuerpo: verde si subió, rojo si bajó.
            let color = if candle.close >= candle.open {
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

        // Eje de precios (Y): mostramos mínimo y máximo ajustados por viewport.
        let font_id = egui::FontId::proportional(12.0);
        let min_visible = center_price + viewport.offset_y * price_range
            - price_range / (2.0 * viewport.zoom_level);
        let max_visible = center_price
            + viewport.offset_y * price_range
            + price_range / (2.0 * viewport.zoom_level);

        painter.text(
            egui::pos2(left - 30.0, top),
            egui::Align2::RIGHT_TOP,
            format!("{:.2}", max_visible),
            font_id.clone(),
            egui::Color32::GRAY,
        );
        painter.text(
            egui::pos2(left - 30.0, bottom),
            egui::Align2::RIGHT_BOTTOM,
            format!("{:.2}", min_visible),
            font_id.clone(),
            egui::Color32::GRAY,
        );

        // Eje de tiempo (X): mostramos rango de índices visibles.
        let first_visible = (-viewport.offset_x).max(0.0) as i32;
        let last_visible = ((-viewport.offset_x)
            + (right - left) as f64 / (base_candle_width as f64 * viewport.zoom_level))
            .min(n as f64 - 1.0) as i32;

        painter.text(
            egui::pos2(left, bottom + 20.0),
            egui::Align2::LEFT_TOP,
            format!("{}", first_visible.max(0)),
            font_id.clone(),
            egui::Color32::GRAY,
        );
        painter.text(
            egui::pos2(right, bottom + 20.0),
            egui::Align2::RIGHT_TOP,
            format!("{}", last_visible.min(n as i32 - 1).max(0)),
            font_id,
            egui::Color32::GRAY,
        );

        tracing::debug!("Renderizado con glow completado correctamente");
        Ok(())
    }

    fn name(&self) -> &str {
        "glow (egui Painter)"
    }
}
