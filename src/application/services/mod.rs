//! Composición de dependencias de la capa de aplicación.
//!
//! Encapsula la creación de `ChartState`, el renderer, la carga de fuentes y
//! los datos de prueba de ejemplo. Aquí vive la `ApexTraderApp` que antes
//! estaba en `main.rs`.

use crate::application::commands::load_candles::LoadCandles;
use crate::chart::layout::ChartLayout;
use crate::application::events::{self, InputHandler};
use crate::domain::candle::Candle;
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::chart::time_point::TimePoint;
use crate::domain::chart_state::ChartState;
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;
use crate::logging::init as init_logging;
use crate::market::mock::MockMarketDataProvider;
use crate::market::provider::MarketDataProvider;
use crate::market::symbol::Symbol;
use crate::renderer::{
    ChartRenderer, ChartScene, InvalidationMask, InvalidationReason, create_renderer,
};
use eframe::egui;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;
use tracing;

/// Estado de carga de datos de mercado.
#[derive(Debug, Clone, PartialEq)]
enum LoadState {
    /// Todavía no se inició la carga.
    Idle,
    /// Los datos se están cargando.
    Loading,
    /// Se cargaron `count` velas correctamente.
    Loaded { count: usize },
    /// La carga falló con el mensaje dado.
    Error(String),
}

/// Aplicación de escritorio con egui.
///
/// Solo coordina: crea el `ChartState` (dueño de los datos y el viewport),
/// crea el renderer (wgpu o glow) y le pide al renderer que dibuje prestando
/// esos datos (`&[Candle]` y `&Viewport`).
///
/// Maneja el input de usuario a través de `InputHandler`.
struct ApexTraderApp {
    chart: ChartState,
    renderer: Box<dyn ChartRenderer>,
    input_handler: InputHandler,
    load_state: LoadState,
    /// Símbolo ingresado por el usuario para cargar datos de mercado.
    symbol_input: String,
    /// Viewport del frame anterior (para calcular invalidation mask).
    prev_viewport: Option<Viewport>,
    /// Posición del crosshair del frame anterior.
    prev_crosshair_pos: Option<ScreenPoint>,
    /// Cantidad de puntos de datos del frame anterior.
    prev_data_len: usize,
    /// Rect de dibujo del frame anterior.
    prev_rect: Option<egui::Rect>,
}

impl eframe::App for ApexTraderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("ApexTrader — Chart de Velas");
            ui.separator();

            ui.label(format!("Renderer: {}", self.renderer.name()));

            ui.separator();

            // Controles para cambiar el símbolo y recargar datos de mercado.
            ui.horizontal(|ui| {
                ui.label("Símbolo:");
                let response = ui.text_edit_singleline(&mut self.symbol_input);
                let recargar = response.lost_focus()
                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    || ui.button("Recargar").clicked();
                if recargar {
                    match Symbol::from_str(&self.symbol_input) {
                        Ok(symbol) => {
                            let provider = MockMarketDataProvider::new(50000.0);
                            let to_ms = 0;
                            let from_ms = -(200i64 * 60_000);
                            self.load_state = load_symbol_into_chart(
                                &mut self.chart,
                                &provider,
                                symbol,
                                from_ms,
                                to_ms,
                            );
                        }
                        Err(e) => {
                            self.load_state = LoadState::Error(format!("Símbolo inválido: {e}"));
                        }
                    }
                }
            });

            ui.separator();

            let available = ui.available_rect_before_wrap();
            let painter = ui.painter();

            let interaction_rect = available.shrink(0.0);
            let response = ui.interact(
                interaction_rect,
                ui.scope_id().with("chart_area"),
                egui::Sense::click_and_drag(),
            );

            let crosshair_pos = {
                let chart = &mut self.chart;
                events::handle_input(
                    &mut self.input_handler,
                    ui,
                    &response,
                    interaction_rect,
                    chart,
                );
                self.input_handler.crosshair_pos()
            };

            let viewport = self.chart.viewport();
            let points = self.chart.data_points();

            // Calcular máscara de invalidación basada en qué cambió entre frames.
            let mask =
                self.calculate_invalidation_mask(viewport, crosshair_pos, points.len(), available);

            // Calcular rango de precios y centro a partir de los puntos visibles.
            let chart_layout =
                ChartLayout::compute(available.width() as f64, available.height() as f64);
            let drawing_width = chart_layout.plot_width();
            let drawing_height = chart_layout.plot_height();
            let visible = viewport.visible_range(points.len(), drawing_width);
            let (price_range, center_price) = if visible.is_empty() {
                (0.0, 0.0)
            } else {
                let visible_points = &points[visible.first..=visible.last];
                let min_price = visible_points
                    .iter()
                    .map(|p| p.low())
                    .fold(f64::INFINITY, f64::min);
                let max_price = visible_points
                    .iter()
                    .map(|p| p.high())
                    .fold(f64::NEG_INFINITY, f64::max);
                let mut pr = max_price - min_price;
                if pr == 0.0 {
                    pr = 1.0;
                }
                let cp = (min_price + max_price) / 2.0;
                (pr, cp)
            };

            let scene = ChartScene {
                data_points: points.to_vec(),
                viewport: *viewport,
                price_range,
                center_price,
                candle_count: points.len(),
                screen_width: drawing_width,
                screen_height: drawing_height,
                crosshair_pos,
                invalidation_mask: mask,
            };

            if let Err(e) = self.renderer.draw(painter, available, &scene) {
                tracing::error!("Error renderizando chart: {}", e);
                ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
            }

            // Guardar estado actual para el próximo frame.
            self.prev_viewport = Some(*viewport);
            self.prev_crosshair_pos = crosshair_pos;
            self.prev_data_len = points.len();
            self.prev_rect = Some(available);

            ui.label(format!(
                "Zoom X: {:.2}x | Zoom Y: {:.2}x | Offset: ({:.2}, {:.2})",
                viewport.zoom_x(),
                viewport.zoom_y(),
                viewport.offset_x(),
                viewport.offset_y()
            ));

            ui.label(format!("Carga de mercado: {:?}", self.load_state));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(format!("Puntos: {}", points.len()));
                ui.label("Scroll: zoom | Arrastre: pan");
            });
        });
    }
}

/// Calcula la máscara de invalidación basada en qué cambió entre frames.
///
/// Compara el estado actual con el del frame anterior para determinar
/// qué partes del chart necesitan ser re-renderizadas.
impl ApexTraderApp {
    fn calculate_invalidation_mask(
        &self,
        viewport: &Viewport,
        crosshair_pos: Option<ScreenPoint>,
        data_len: usize,
        rect: egui::Rect,
    ) -> InvalidationMask {
        // Primer frame: todo es nuevo.
        if self.prev_viewport.is_none() {
            return InvalidationMask {
                level: InvalidationReason::Full,
            };
        }

        let mut mask = InvalidationMask::none();

        // Si cambió el rect (resize), es ScaleLayout.
        if let Some(prev_rect) = self.prev_rect
            && prev_rect != rect
        {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::ScaleLayout,
            });
        }

        // Si cambió el viewport (zoom/pan), es ScaleLayout.
        if let Some(prev_vp) = self.prev_viewport
            && *viewport != prev_vp
        {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::ScaleLayout,
            });
        }

        // Si cambiaron los datos, es SeriesData.
        if data_len != self.prev_data_len {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::SeriesData,
            });
        }

        // Si cambió solo el crosshair, es CursorOnly (solo si no hay algo peor).
        if let Some(prev_ch) = self.prev_crosshair_pos
            && crosshair_pos != Some(prev_ch)
            && mask.level == InvalidationReason::None
        {
            mask = InvalidationMask {
                level: InvalidationReason::CursorOnly,
            };
        }

        mask
    }
}

/// Ejecuta la aplicación de escritorio (punto de entrada de la capa de
/// aplicación).
///
/// Esta función es invocada desde `application::run()` y contiene la lógica
/// que antes vivía directamente en `main.rs`.
pub fn run() -> eframe::Result {
    if let Err(e) = init_logging() {
        eprintln!("Advertencia: no se pudo inicializar logging: {}", e);
    }

    tracing::info!("Iniciando ApexTrader");

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "ApexTrader",
        native_options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);

            let mut chart = ChartState::new();

            let provider = MockMarketDataProvider::new(50000.0);
            let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
            let to_ms = 0;
            let from_ms = -(200i64 * 60_000);

            let load_state = load_symbol_into_chart(&mut chart, &provider, symbol, from_ms, to_ms);

            // Crear renderer (intenta wgpu, cae a glow si falla)
            let renderer = create_renderer();

            Ok(Box::new(ApexTraderApp {
                chart,
                renderer,
                input_handler: InputHandler::new(),
                load_state,
                symbol_input: "BTC/USDT".to_string(),
                prev_viewport: None,
                prev_crosshair_pos: None,
                prev_data_len: 0,
                prev_rect: None,
            }))
        }),
    )
}

/// Carga velas de un símbolo desde un proveedor de mercado en el `ChartState`.
///
/// Limpia las velas previas, obtiene las nuevas del proveedor, las agrega al
/// chart y aplica `auto_scale` para que quepan en pantalla.
/// En caso de error, hace fallback a velas sintéticas y retorna
/// `LoadState::Error`.
fn load_symbol_into_chart(
    chart: &mut ChartState,
    provider: &dyn MarketDataProvider,
    symbol: Symbol,
    from_ms: i64,
    to_ms: i64,
) -> LoadState {
    match LoadCandles::execute(provider, symbol, from_ms, to_ms) {
        Ok(result) => {
            if !result.is_clean() {
                tracing::warn!(
                    "Ingestión con {} rechazado(s) de {} punto(s) recibido(s) para {}",
                    result.rejected.len(),
                    result.total_received(),
                    result.symbol
                );
                for r in &result.rejected {
                    tracing::warn!("  índice {} ({} ms): {}", r.index, r.open_time_ms, r.reason);
                }
            }

            chart.clear();
            for dp in &result.data_points {
                if let Err(e) = chart.add_data_point(*dp) {
                    tracing::error!("No se pudo agregar punto de datos: {}", e);
                }
            }
            tracing::info!(
                "ChartState inicializado con {} puntos de datos del mercado ({})",
                chart.len(),
                result.symbol
            );

            if let Some(range) = price_range_from_data_points(chart) {
                let point_count = chart.len();
                let screen_width = 1920.0;
                let screen_height = 1080.0;
                chart
                    .viewport_mut()
                    .auto_scale(range, point_count, screen_width, screen_height);
            }

            LoadState::Loaded { count: chart.len() }
        }
        Err(e) => {
            tracing::error!("No se pudieron cargar datos de mercado: {e}");
            chart.clear();
            let mut price = 100.0;
            for i in 0..20 {
                let open = price;
                let change = (i as f64 * 0.5).sin() * 3.0;
                let close = open + change * 0.7;
                let high = open.max(close) + change.abs() * 0.3;
                let low = open.min(close) - change.abs() * 0.3;
                let volume = 1000.0 + (i as f64 * 0.5).sin().abs() * 500.0;
                let open_time_ms = i as i64 * 60_000;
                let close_time_ms = open_time_ms + 60_000 - 1;
                if let Some(tp) = TimePoint::new(open_time_ms, close_time_ms, 60_000) {
                    if let Ok(candle) = Candle::new(open, high, low, close, volume) {
                        if let Err(e) = chart.add_data_point(ChartDataPoint::new(candle, tp)) {
                            tracing::error!("No se pudo agregar punto de datos {}: {}", i, e);
                        }
                    } else {
                        tracing::error!("No se pudo crear vela sintética {}: {}", i, open);
                    }
                }
                price = close;
            }
            tracing::info!(
                "ChartState inicializado con {} puntos de datos (fallback)",
                chart.len()
            );
            LoadState::Error(e.to_string())
        }
    }
}

/// Intenta cargar una fuente del sistema Windows para que los caracteres
/// especiales (tildes, ñ, em dash, etc.) se rendericen correctamente.
fn configure_fonts(ctx: &egui::Context) {
    let mut font_defs = egui::FontDefinitions::default();

    let windows_font_paths = [
        r"C:\Windows\Fonts\segui.ttf",
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\segoeuib.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
        r"C:\Windows\Fonts\times.ttf",
    ];

    let mut loaded_system_font = false;

    for path in windows_font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            font_defs.font_data.insert(
                "system_font".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );

            if let Some(family) = font_defs.families.get_mut(&egui::FontFamily::Proportional) {
                family.insert(0, "system_font".to_owned());
            }

            loaded_system_font = true;
            break;
        }
    }

    if !loaded_system_font {
        font_defs = egui::FontDefinitions::default();
    }

    ctx.set_fonts(font_defs);
}

/// Calcula el `PriceRange` (mínimo/máximo) de los puntos de datos del chart.
///
/// Devuelve `None` si no hay puntos de datos.
fn price_range_from_data_points(chart: &ChartState) -> Option<PriceRange> {
    let points = chart.data_points();
    if points.is_empty() {
        return None;
    }

    let mut min_price = f64::INFINITY;
    let mut max_price = f64::NEG_INFINITY;

    for p in points {
        min_price = min_price.min(p.low());
        max_price = max_price.max(p.high());
    }

    let min = Price::new(Decimal::from_f64(min_price).expect("precio mínimo convertible"))
        .expect("precio mínimo válido");
    let max = Price::new(Decimal::from_f64(max_price).expect("precio máximo convertible"))
        .expect("precio máximo válido");

    PriceRange::new(min, max).ok()
}
