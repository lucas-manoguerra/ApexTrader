//! Composición de dependencias de la capa de aplicación.
//!
//! Encapsula la creación de `ChartState`, el renderer, la carga de fuentes y
//! los datos de prueba de ejemplo. Aquí vive la `ApexTraderApp` que antes
//! estaba en `main.rs`.

use crate::application::commands::load_candles::LoadCandles;
use crate::application::events::{self, InputHandler};
use crate::chart::layout::ChartLayout;
use crate::domain::chart_state::ChartState;
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::Viewport;
use crate::logging::init as init_logging;
use crate::market::candle::MarketCandle;
use crate::market::mock::MockMarketDataProvider;
use crate::market::mock_realtime::MockRealtimeMarketDataProvider;
use crate::market::provider::{MarketDataProvider, RealtimeMarketDataProvider};
use crate::market::symbol::Symbol;
use crate::renderer::{
    ChartRenderer, ChartScene, InvalidationMask, InvalidationReason, create_renderer,
};
use eframe::egui;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing;

const MOCK_INTERVAL_MS: i64 = 60_000;
const MOCK_HISTORY_CANDLES: i64 = 200;
const MOCK_TICKS_PER_CANDLE: u32 = 20;
const MOCK_REALTIME_VOLUME: f64 = 35.0;
const MOCK_REALTIME_TICK_INTERVAL: Duration = Duration::from_millis(250);

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
    /// Incremento mínimo de precio del instrumento seleccionado.
    price_tick_size: f64,
    /// Viewport del frame anterior (para calcular invalidation mask).
    prev_viewport: Option<Viewport>,
    /// Posición del crosshair del frame anterior.
    prev_crosshair_pos: Option<ScreenPoint>,
    /// Revisión de datos del frame anterior.
    prev_data_revision: u64,
    /// Indica que una carga completa de historial requiere ajustar el eje X una vez.
    fit_content_pending: bool,
    /// Stream realtime intercambiable por un proveedor WebSocket real.
    realtime_provider: Option<Box<dyn RealtimeMarketDataProvider>>,
    /// Sigue la última vela cuando el usuario está mirando el extremo derecho.
    follow_latest: bool,
    /// Indica que se agregó una vela nueva y hay que evaluar el seguimiento.
    follow_latest_pending: bool,
    /// Instante previsto para la siguiente actualización realtime.
    next_realtime_tick: Instant,
    /// Rect de dibujo del frame anterior.
    prev_rect: Option<egui::Rect>,
}

impl eframe::App for ApexTraderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.advance_realtime(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("ApexTrader — Chart de Velas");
            ui.separator();

            ui.label(format!("Renderer: {}", self.renderer.name()));
            ui.checkbox(&mut self.follow_latest, "Seguir última vela");
            ui.horizontal(|ui| {
                let mut auto_scale = self.chart.viewport().price_scale_auto();
                if ui.checkbox(&mut auto_scale, "Auto escala de precios").changed() {
                    self.chart.viewport_mut().set_price_scale_auto(auto_scale);
                }

                let mut locked = self.chart.viewport().price_scale_locked();
                if ui.checkbox(&mut locked, "Bloquear escala vertical").changed() {
                    self.chart.viewport_mut().set_price_scale_locked(locked);
                }
            });

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
                            self.price_tick_size = price_tick_size_for_symbol(&symbol);
                            let provider = MockMarketDataProvider::new(50_000.0)
                                .with_interval_ms(MOCK_INTERVAL_MS)
                                .with_base_volume(MOCK_REALTIME_VOLUME);
                            let to_ms = latest_closed_candle_open_time(MOCK_INTERVAL_MS);
                            let from_ms = to_ms.saturating_sub(
                                (MOCK_HISTORY_CANDLES - 1) * MOCK_INTERVAL_MS,
                            );
                            self.load_state = load_symbol_into_chart(
                                &mut self.chart,
                                &provider,
                                symbol.clone(),
                                from_ms,
                                to_ms,
                            );
                            self.realtime_provider = create_realtime_provider(
                                &self.chart,
                                symbol,
                                MOCK_REALTIME_VOLUME,
                                MOCK_TICKS_PER_CANDLE,
                            );
                            self.next_realtime_tick = Instant::now() + MOCK_REALTIME_TICK_INTERVAL;
                            self.fit_content_pending = true;
                        }
                        Err(e) => {
                            self.realtime_provider = None;
                            self.load_state = LoadState::Error(format!("Símbolo inválido: {e}"));
                        }
                    }
                }
            });

            ui.separator();

            let available = ui.available_rect_before_wrap();
            let footer_height = 58.0_f32.min(available.height() * 0.24);
            let chart_height = (available.height() - footer_height).max(96.0);
            let chart_rect = egui::Rect::from_min_size(
                available.min,
                egui::vec2(available.width(), chart_height.min(available.height())),
            );
            let response = ui.allocate_rect(chart_rect, egui::Sense::click_and_drag());
            let painter = ui.painter_at(chart_rect);
            let chart_layout =
                ChartLayout::compute(chart_rect.width() as f64, chart_rect.height() as f64);

            // Ajustar el historial completo solo al iniciar o tras una carga
            // explícita. Las actualizaciones incrementales no deben resetear
            // el zoom ni el desplazamiento que eligió el usuario.
            if self.fit_content_pending {
                let logical_count = self.chart.time_scale().len();
                if logical_count > 0 {
                    self.chart
                        .viewport_mut()
                        .fit_content(logical_count, chart_layout.plot_width());
                    self.fit_content_pending = false;
                }
            } else if self.follow_latest_pending && self.follow_latest {
                let point_count = self.chart.len();
                if point_count > 1 && chart_layout.plot_width() > 0.0 {
                    let (was_at_right_edge, current_offset) = {
                        let viewport = self.chart.viewport();
                        let previous_last = (point_count - 2) as f64;
                        let last_x = viewport
                            .logical_to_screen(previous_last, chart_layout.plot_width());
                        (
                            last_x >= chart_layout.plot_width() * 0.8
                                && last_x <= chart_layout.plot_width() * 1.05,
                            viewport.offset_x(),
                        )
                    };
                    if was_at_right_edge {
                        // Una posición lógica a la izquierda mantiene la nueva
                        // vela en el borde derecho sin tocar zoom ni escala Y.
                        self.chart
                            .viewport_mut()
                            .set_offset_x(current_offset - 1.0);
                    }
                }
            }
            self.follow_latest_pending = false;

            let pre_input_count = self.chart.time_scale().len();
            let pre_input_visible = self
                .chart
                .viewport()
                .visible_range(pre_input_count, chart_layout.plot_width());
            if !pre_input_visible.is_empty() {
                let points = self.chart.data_points();
                let visible_points = &points[pre_input_visible.first..=pre_input_visible.last];
                let (min_price, max_price) = visible_points.iter().fold(
                    (f64::INFINITY, f64::NEG_INFINITY),
                    |(min_price, max_price), point| {
                        (min_price.min(point.low()), max_price.max(point.high()))
                    },
                );
                self.chart
                    .viewport_mut()
                    .update_auto_price_scale(min_price, max_price);
            }

            let interaction_rect = chart_rect;

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
            let data_revision = self.chart.data_revision();
            let mask =
                self.calculate_invalidation_mask(viewport, crosshair_pos, data_revision, chart_rect);

            // El rango de referencia pertenece al Viewport: Auto lo actualiza
            // con el rango visible; Manual lo conserva aunque llegue realtime.
            let drawing_width = chart_layout.plot_width();
            let drawing_height = chart_layout.plot_height();
            let logical_count = self.chart.time_scale().len();
            let visible = viewport.visible_range(logical_count, drawing_width);
            let max_volume = if visible.is_empty() {
                0.0
            } else {
                let visible_points = &points[visible.first..=visible.last];
                visible_points
                    .iter()
                    .map(|point| point.volume())
                    .fold(0.0_f64, f64::max)
            };
            if viewport.price_scale_auto() && !visible.is_empty() {
                let visible_points = &points[visible.first..=visible.last];
                let (min_price, max_price) = visible_points.iter().fold(
                    (f64::INFINITY, f64::NEG_INFINITY),
                    |(min_price, max_price), point| {
                        (min_price.min(point.low()), max_price.max(point.high()))
                    },
                );
                self.chart
                    .viewport_mut()
                    .update_auto_price_scale(min_price, max_price);
            }
            let viewport = self.chart.viewport();
            let (price_range, center_price) = if visible.is_empty() {
                (0.0, 0.0)
            } else {
                viewport.price_scale_reference()
            };

            let scene = ChartScene {
                // ChartScene toma un préstamo de ChartState; no duplica el historial.
                data_points: points,
                viewport: *viewport,
                price_range,
                center_price,
                price_tick_size: self.price_tick_size,
                max_volume,
                candle_count: logical_count,
                screen_width: drawing_width,
                screen_height: drawing_height,
                crosshair_pos,
                invalidation_mask: mask,
            };

            if let Err(e) = self.renderer.draw(&painter, chart_rect, &scene) {
                tracing::error!("Error renderizando chart: {}", e);
                ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
            }

            // Guardar estado actual para el próximo frame.
            self.prev_viewport = Some(*viewport);
            self.prev_crosshair_pos = crosshair_pos;
            self.prev_data_revision = data_revision;
            self.prev_rect = Some(chart_rect);

            ui.horizontal_wrapped(|ui| {
                ui.small(format!(
                    "Zoom X: {:.2}x | Zoom Y: {:.2}x | Offset: ({:.2}, {:.2})",
                    viewport.zoom_x(),
                    viewport.zoom_y(),
                    viewport.offset_x(),
                    viewport.offset_y()
                ));
                ui.separator();
                ui.small(format!(
                    "Carga: {:?} | Realtime: {} | Puntos: {}",
                    self.load_state,
                    if self.realtime_provider.is_some() { "activo" } else { "inactivo" },
                    points.len()
                ));
            });
            ui.small(
                "Rueda vertical: zoom temporal | Rueda horizontal: desplazar tiempo | Arrastrar: mover | Arrastrar escala derecha/inferior: ajustar | Doble clic en escala: restablecer",
            );
        });
    }
}

/// Calcula la máscara de invalidación basada en qué cambió entre frames.
///
/// Compara el estado actual con el del frame anterior para determinar
/// qué partes del chart necesitan ser re-renderizadas.
impl ApexTraderApp {
    fn advance_realtime(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        if let Some(provider) = self.realtime_provider.as_mut() {
            if now >= self.next_realtime_tick {
                self.next_realtime_tick = now + MOCK_REALTIME_TICK_INTERVAL;
                match provider.next_update() {
                    Ok(update) => {
                        let candle_closed = update.is_closed;
                        match update.candle.to_chart_data_point() {
                            Ok(point) => {
                                let previous_count = self.chart.len();
                                if let Err(error) = self.chart.add_data_point(point) {
                                    tracing::error!(
                                        "No se pudo aplicar la actualización realtime: {error}"
                                    );
                                } else {
                                    self.follow_latest_pending |= self.chart.len() > previous_count;
                                    if candle_closed {
                                        tracing::debug!(
                                            "Mock realtime cerró vela {} en {} ms",
                                            update.candle.symbol,
                                            update.candle.open_time_ms
                                        );
                                    }
                                }
                            }
                            Err(error) => tracing::error!(
                                "El mock realtime generó un punto inválido: {error}"
                            ),
                        }
                    }
                    Err(error) => tracing::error!("Error en mock realtime: {error}"),
                }
            }

            ctx.request_repaint_after(
                self.next_realtime_tick
                    .saturating_duration_since(Instant::now()),
            );
        }
    }

    fn calculate_invalidation_mask(
        &self,
        viewport: &Viewport,
        crosshair_pos: Option<ScreenPoint>,
        data_revision: u64,
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
        if data_revision != self.prev_data_revision {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::SeriesData,
            });
        }

        // Si cambió solo el crosshair, es CursorOnly (solo si no hay algo peor).
        if crosshair_pos != self.prev_crosshair_pos && mask.level == InvalidationReason::None {
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

            let provider = MockMarketDataProvider::new(50_000.0)
                .with_interval_ms(MOCK_INTERVAL_MS)
                .with_base_volume(MOCK_REALTIME_VOLUME);
            let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");
            let price_tick_size = price_tick_size_for_symbol(&symbol);
            let to_ms = latest_closed_candle_open_time(MOCK_INTERVAL_MS);
            let from_ms = to_ms.saturating_sub((MOCK_HISTORY_CANDLES - 1) * MOCK_INTERVAL_MS);

            let load_state =
                load_symbol_into_chart(&mut chart, &provider, symbol.clone(), from_ms, to_ms);
            let realtime_provider = create_realtime_provider(
                &chart,
                symbol,
                MOCK_REALTIME_VOLUME,
                MOCK_TICKS_PER_CANDLE,
            );

            // Crear renderer (intenta wgpu, cae a glow si falla)
            let renderer = create_renderer();
            let data_revision = chart.data_revision();

            Ok(Box::new(ApexTraderApp {
                chart,
                renderer,
                input_handler: InputHandler::new(),
                load_state,
                symbol_input: "BTC/USDT".to_string(),
                price_tick_size,
                prev_viewport: None,
                prev_crosshair_pos: None,
                prev_data_revision: data_revision,
                fit_content_pending: true,
                realtime_provider,
                follow_latest: true,
                follow_latest_pending: false,
                next_realtime_tick: Instant::now() + MOCK_REALTIME_TICK_INTERVAL,
                prev_rect: None,
            }))
        }),
    )
}

/// Incremento mínimo para los instrumentos soportados por el mock.
////
//// La precisión de mercado real debe provenir de los metadatos del exchange;
//// estas reglas solo permiten que el gráfico sintético represente formatos
//// habituales para BTC, futuros CME y pares FX.
fn price_tick_size_for_symbol(symbol: &Symbol) -> f64 {
    let base = symbol.base().to_ascii_uppercase();
    let quote = symbol.quote().to_ascii_uppercase();

    if matches!(base.as_str(), "ES" | "MES" | "NQ" | "MNQ") {
        return 0.25;
    }

    const FX_CURRENCIES: [&str; 8] = ["USD", "EUR", "GBP", "JPY", "CHF", "CAD", "AUD", "NZD"];
    if FX_CURRENCIES.contains(&base.as_str()) && FX_CURRENCIES.contains(&quote.as_str()) {
        return if quote == "JPY" { 0.001 } else { 0.00001 };
    }

    0.01
}

#[cfg(test)]
mod price_tick_tests {
    use super::*;

    #[test]
    fn price_tick_size_uses_known_mock_instrument_conventions() {
        assert_eq!(
            price_tick_size_for_symbol(&Symbol::from_str("BTC/USDT").unwrap()),
            0.01
        );
        assert_eq!(
            price_tick_size_for_symbol(&Symbol::from_str("MNQ/USD").unwrap()),
            0.25
        );
        assert_eq!(
            price_tick_size_for_symbol(&Symbol::from_str("EUR/USD").unwrap()),
            0.00001
        );
        assert_eq!(
            price_tick_size_for_symbol(&Symbol::from_str("USD/JPY").unwrap()),
            0.001
        );
    }
}

/// Timestamp de apertura de la última vela completamente cerrada.
fn latest_closed_candle_open_time(interval_ms: i64) -> i64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64;
    let current_open = now_ms.div_euclid(interval_ms) * interval_ms;
    current_open.saturating_sub(interval_ms)
}

/// Construye el stream realtime a continuación del último punto histórico.
fn create_realtime_provider(
    chart: &ChartState,
    symbol: Symbol,
    base_volume: f64,
    ticks_per_candle: u32,
) -> Option<Box<dyn RealtimeMarketDataProvider>> {
    let last_point = chart.data_points().last()?;
    let close_time_ms = last_point
        .time_point
        .open_time_ms
        .checked_add(last_point.time_point.interval_ms)?;
    let last_candle = MarketCandle {
        symbol: symbol.clone(),
        open_time_ms: last_point.time_point.open_time_ms,
        open: last_point.open(),
        high: last_point.high(),
        low: last_point.low(),
        close: last_point.close(),
        volume: last_point.volume(),
        close_time_ms,
    };
    let seed_provider = MockMarketDataProvider::new(last_point.close())
        .with_interval_ms(last_point.time_point.interval_ms)
        .with_base_volume(base_volume);
    match MockRealtimeMarketDataProvider::from_last_candle(
        last_candle,
        ticks_per_candle,
        base_volume,
        seed_provider.seed_for_symbol(&symbol),
    ) {
        Ok(provider) => Some(Box::new(provider)),
        Err(error) => {
            tracing::warn!("No se pudo iniciar el mock realtime: {error}");
            None
        }
    }
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
    match LoadCandles::execute(provider, symbol.clone(), from_ms, to_ms) {
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
            if let Err(e) = chart.add_data_points(&result.data_points) {
                tracing::error!("No se pudieron agregar los datos al chart: {}", e);
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

            // El fallback usa el mismo generador OHLCV realista para no volver
            // a introducir una serie simplista y desconectada del histórico.
            let fallback_provider = MockMarketDataProvider::new(100.0)
                .with_interval_ms(MOCK_INTERVAL_MS)
                .with_base_volume(MOCK_REALTIME_VOLUME);
            match LoadCandles::execute(&fallback_provider, symbol.clone(), from_ms, to_ms) {
                Ok(fallback) => {
                    if let Err(error) = chart.add_data_points(&fallback.data_points) {
                        tracing::error!("No se pudo cargar el histórico de fallback: {error}");
                    }
                    if let Some(range) = price_range_from_data_points(chart) {
                        let point_count = chart.len();
                        chart
                            .viewport_mut()
                            .auto_scale(range, point_count, 1920.0, 1080.0);
                    }
                }
                Err(fallback_error) => {
                    tracing::error!(
                        "También falló el histórico sintético de fallback: {fallback_error}"
                    );
                }
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
