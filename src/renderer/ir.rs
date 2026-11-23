//! Representación intermedia de renderizado.
//!
//! Separa la construcción de la escena (qué dibujar) del backend de renderizado
//! (cómo dibujarlo). Tanto Glow como Wgpu consumen la misma `ChartScene`.
//! El dominio no conoce este módulo: solo el renderer y la aplicación lo usan.

use crate::chart::layout::ChartLayout;
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;
use eframe::egui;

/// Una capa de renderizado: agrupa elementos del mismo tipo para dibujar juntos.
#[derive(Debug, Clone, Default)]
pub struct RenderLayer {
    /// Velas a dibujar en esta capa.
    pub candles: Vec<CandleRender>,
    /// Barras de volumen.
    pub volume_bars: Vec<VolumeBarRender>,
    /// Etiquetas de eje de precios.
    pub price_axis_labels: Vec<PriceAxisLabel>,
    /// Etiquetas de eje de tiempo.
    pub time_axis_labels: Vec<TimeAxisLabel>,
    /// Líneas de crosshair.
    pub crosshair: Option<CrosshairRender>,
}

/// Datos para renderizar una vela individual.
#[derive(Debug, Clone, Copy)]
pub struct CandleRender {
    /// Índice de la vela en la serie.
    pub index: usize,
    /// Precio de apertura (píxel Y).
    pub open_y: f32,
    /// Precio máximo (píxel Y).
    pub high_y: f32,
    /// Precio mínimo (píxel Y).
    pub low_y: f32,
    /// Precio de cierre (píxel Y).
    pub close_y: f32,
    /// Posición X del centro (píxel).
    pub center_x: f32,
    /// Ancho del cuerpo (píxeles).
    pub body_width: f32,
    /// Color: verde si subió, rojo si bajó.
    pub color: egui::Color32,
    /// Volumen (para la barra de volumen).
    pub volume: f64,
}

/// Datos para renderizar una barra de volumen.
#[derive(Debug, Clone, Copy)]
pub struct VolumeBarRender {
    /// Posición X del centro (píxel).
    pub center_x: f32,
    /// Altura de la barra (píxeles).
    pub height: f32,
    /// Posición Y de la base (píxel).
    pub base_y: f32,
    /// Color de la barra.
    pub color: egui::Color32,
}

/// Etiqueta de eje de precios (Y).
#[derive(Debug, Clone)]
pub struct PriceAxisLabel {
    /// Texto de la etiqueta.
    pub text: String,
    /// Posición Y (píxel).
    pub y: f32,
}

/// Etiqueta de eje de tiempo (X).
#[derive(Debug, Clone)]
pub struct TimeAxisLabel {
    /// Texto de la etiqueta.
    pub text: String,
    /// Posición X (píxel).
    pub x: f32,
}

/// Renderizado de crosshair.
#[derive(Debug, Clone, Copy)]
pub struct CrosshairRender {
    /// Posición X (píxel).
    pub x: f32,
    /// Posición Y (píxel).
    pub y: f32,
    /// Precio bajo el cursor.
    pub price: f64,
    /// Índice de vela bajo el cursor.
    pub index: usize,
    /// Color de las líneas.
    pub color: egui::Color32,
}

/// Un frame de renderizado: todo lo necesario para dibujar un frame.
#[derive(Debug, Clone)]
pub struct RenderFrame {
    /// Rect de dibujo completo.
    pub rect: egui::Rect,
    /// Layout del chart.
    pub layout: ChartLayout,
    /// Capas de renderizado.
    pub layers: Vec<RenderLayer>,
    /// Posición del crosshair en pantalla.
    pub crosshair_pos: Option<ScreenPoint>,
}

/// Geometría lista para enviar al backend GPU.
#[derive(Debug, Clone, Default)]
pub struct RenderGeometry {
    /// Vértices para cuerpos de velas (x, y, r, g).
    pub candle_vertices: Vec<[f32; 4]>,
    /// Índices para cuerpos de velas.
    pub candle_indices: Vec<u16>,
    /// Vértices para méchas (wicks).
    pub wick_vertices: Vec<[f32; 4]>,
    /// Vértices para barras de volumen.
    pub volume_vertices: Vec<[f32; 4]>,
}

/// La escena completa del chart: qué dibujar, sin decir cómo.
#[derive(Debug, Clone)]
pub struct ChartScene<'a> {
    /// Vista prestada de los puntos de datos propiedad de ChartState.
    ///
    /// La escena no clona ni toma posesión de las velas; su vida útil queda
    /// limitada al préstamo entregado por la aplicación en este frame.
    pub data_points: &'a [ChartDataPoint],
    /// Viewport (zoom y pan).
    pub viewport: Viewport,
    /// Rango de precios visible (máximo - mínimo).
    pub price_range: f64,
    /// Precio central de zoom.
    pub center_price: f64,
    /// Volumen máximo entre las velas visibles.
    pub max_volume: f64,
    /// Cantidad de velas cargadas.
    pub candle_count: usize,
    /// Ancho del área de dibujo (píxeles).
    pub screen_width: f64,
    /// Alto del área de dibujo (píxeles).
    pub screen_height: f64,
    /// Posición del crosshair en pantalla, si hay uno activo.
    pub crosshair_pos: Option<ScreenPoint>,
    /// Máscara de invalidación: qué partes del chart necesitan ser re-renderizadas.
    pub invalidation_mask: InvalidationMask,
}

/// Motivo por el cual el chart necesita ser re-renderizado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvalidationReason {
    /// No hay cambios; nada que re-renderizar.
    #[default]
    None,
    /// Solo el cursor/crosshair se movió; los datos no cambiaron.
    CursorOnly,
    /// Se superpuso un overlay o dibujo nuevo; el resto del chart es igual.
    Overlay,
    /// Cambiaron los datos de las velas (nuevos símbolos, recarga).
    SeriesData,
    /// Cambiaron escala, layout o dimensiones (zoom, pan, resize).
    ScaleLayout,
    /// Cambio total; hay que re-renderizar todo desde cero.
    Full,
}

/// Máscara de invalidación para el renderizado.
///
/// Permite al renderer saber qué partes del chart son caras de regenerar
/// y cuáles se pueden reusar del frame anterior.
///
/// Jerarquía de costo (de menor a mayor):
///   `None` < `CursorOnly` < `Overlay` < `SeriesData` < `ScaleLayout` < `Full`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InvalidationMask {
    /// Nivel más alto de invalidación detectado.
    pub level: InvalidationReason,
}

impl InvalidationMask {
    /// Máscara vacía: no hay cambios.
    pub fn none() -> Self {
        Self {
            level: InvalidationReason::None,
        }
    }

    /// Devuelve `true` si no hay nada que re-renderizar.
    pub fn is_none(&self) -> bool {
        self.level == InvalidationReason::None
    }

    /// Combina dos máscaras, quedándose con el nivel de mayor costo.
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
    /// Crea una `ChartScene` vacía.
    pub fn empty() -> Self {
        Self {
            data_points: &[],
            viewport: Viewport::new(),
            price_range: 0.0,
            center_price: 0.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::candle::Candle;
    use crate::domain::chart::time_point::TimePoint;

    #[test]
    fn chart_scene_borrows_data_without_copying_the_history() {
        let candle = Candle::new(100.0, 105.0, 99.0, 103.0, 1_000.0)
            .expect("vela válida");
        let time_point = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let points = [ChartDataPoint::new(candle, time_point)];
        let original_ptr = points.as_ptr();

        let scene = ChartScene {
            data_points: &points,
            viewport: Viewport::new(),
            price_range: 6.0,
            center_price: 102.0,
            max_volume: 1_000.0,
            candle_count: points.len(),
            screen_width: 800.0,
            screen_height: 500.0,
            crosshair_pos: None,
            invalidation_mask: InvalidationMask::none(),
        };

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
}
