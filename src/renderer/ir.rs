//! Representación intermedia de renderizado.
//!
//! Separa la preparación de geometría del backend concreto. La escena toma
//! datos prestados del dominio; el frame convierte solo el rango visible en
//! primitivas tipadas que pueden consumir distintos renderers.
//!
//! Este módulo no depende de egui ni de wgpu.

use crate::chart::layout::{ChartLayout, Rect};
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;

/// Color RGBA independiente de cualquier backend gráfico.
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

/// Una capa de renderizado: agrupa elementos del mismo tipo para dibujar juntos.
#[derive(Debug, Clone, Default)]
pub struct RenderLayer {
    /// Velas a dibujar en esta capa.
    pub candles: Vec<CandleRender>,
    /// Barras de volumen, ordenadas igual que las velas correspondientes.
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
    /// Índice lógico de la vela en la serie.
    pub index: usize,
    /// Precio de apertura (píxel Y relativo al rect del chart).
    pub open_y: f32,
    /// Precio máximo (píxel Y relativo al rect del chart).
    pub high_y: f32,
    /// Precio mínimo (píxel Y relativo al rect del chart).
    pub low_y: f32,
    /// Precio de cierre (píxel Y relativo al rect del chart).
    pub close_y: f32,
    /// Posición X del centro (píxel relativo al rect del chart).
    pub center_x: f32,
    /// Ancho del cuerpo (píxeles).
    pub body_width: f32,
    /// Color de la vela.
    pub color: RenderColor,
    /// Volumen original de la vela.
    pub volume: f64,
}

/// Datos para renderizar una barra de volumen.
#[derive(Debug, Clone, Copy)]
pub struct VolumeBarRender {
    /// Índice lógico de la vela asociada.
    pub index: usize,
    /// Posición X del centro (píxel relativo al rect del chart).
    pub center_x: f32,
    /// Altura de la barra (píxeles).
    pub height: f32,
    /// Posición Y de la base (píxel relativo al rect del chart).
    pub base_y: f32,
    /// Color de la barra.
    pub color: RenderColor,
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
    pub color: RenderColor,
}

/// Un frame preparado para que lo consuma un backend de renderizado.
///
/// Las coordenadas de layout y primitivas son locales al rect del chart.
/// origin permite al backend trasladarlas al espacio de pantalla.
#[derive(Debug, Clone)]
pub struct RenderFrame {
    /// Rect de dibujo completo, en coordenadas locales.
    pub rect: Rect,
    /// Origen del rect completo en coordenadas de pantalla.
    pub origin: ScreenPoint,
    /// Layout del chart.
    pub layout: ChartLayout,
    /// Capas preparadas para este frame.
    pub layers: Vec<RenderLayer>,
    /// Posición del crosshair en coordenadas relativas al rect completo.
    pub crosshair_pos: Option<ScreenPoint>,
}

impl RenderFrame {
    /// Prepara un frame y, opcionalmente, la geometría de la serie.
    ///
    /// include_series permite reutilizar la geometría cacheada cuando solo
    /// cambia el cursor u otro overlay que no modifica velas ni volumen.
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

/// Geometría lista para enviar a un backend GPU.
#[derive(Debug, Clone, Default)]
pub struct RenderGeometry {
    /// Vértices para cuerpos de velas (x, y, r, g).
    pub candle_vertices: Vec<[f32; 4]>,
    /// Índices para cuerpos de velas.
    pub candle_indices: Vec<u16>,
    /// Vértices para mechas (wicks).
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
    /// Incremento mínimo de precio del instrumento, para alinear y formatear ticks.
    pub price_tick_size: f64,
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

    /// Devuelve true si no hay nada que re-renderizar.
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
    /// Crea una ChartScene vacía.
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
        let relative_price =
            price - scene.center_price - scene.viewport.offset_y() * scene.price_range;
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
