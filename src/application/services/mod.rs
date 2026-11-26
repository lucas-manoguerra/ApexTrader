use crate::application::commands::load_candles::LoadCandles;
use crate::presentation::input::egui::{self, InputHandler};
use crate::chart::layout::ChartLayout;
use crate::domain::chart::chart_data::ChartDataPoint;
use crate::domain::chart_state::ChartState;
use crate::domain::price::Price;
use crate::domain::price_range::PriceRange;
use crate::domain::transforms::ScreenPoint;
use crate::domain::viewport::{PriceScaleMode, PriceScaleType, Viewport};
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

#[derive(Debug, Clone, PartialEq)]
enum LoadState {
    Idle,

    Loading,

    Loaded { count: usize },

    Error(String),
}

struct ApexTraderApp {
    chart: ChartState,
    renderer: Box<dyn ChartRenderer>,
    input_handler: InputHandler,
    load_state: LoadState,
    symbol_input: String,
    price_tick_size: f64,
    prev_viewport: Option<Viewport>,
    prev_crosshair_pos: Option<ScreenPoint>,
    prev_data_revision: u64,
    fit_content_pending: bool,
    realtime_provider: Option<Box<dyn RealtimeMarketDataProvider>>,
    follow_latest: bool,
    follow_latest_pending: bool,
    next_realtime_tick: Instant,
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

                let current_mode = self.chart.viewport().price_scale_mode();
                let mut selected_mode = current_mode;
                egui::ComboBox::from_id_salt("price-scale-auto-mode")
                    .selected_text(selected_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut selected_mode,
                            PriceScaleMode::HighLow,
                            "Máximo/Mínimo (OHLC)",
                        );
                        ui.selectable_value(
                            &mut selected_mode,
                            PriceScaleMode::Close,
                            "Solo cierres",
                        );
                    });
                if selected_mode != current_mode {
                    self.chart
                        .viewport_mut()
                        .set_price_scale_mode(selected_mode);
                }

                let current_scale_type = self.chart.viewport().price_scale_type();
                let mut selected_scale_type = current_scale_type;
                let points = self.chart.data_points();
                let logarithmic_supported = points.iter().all(|point| point.low() > 0.0);
                let percentage_base = points
                    .first()
                    .map(|point| point.close())
                    .filter(|price| price.is_finite() && *price > 0.0);

                egui::ComboBox::from_id_salt("price-scale-type")
                    .selected_text(selected_scale_type.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut selected_scale_type,
                            PriceScaleType::Linear,
                            "Lineal",
                        );
                        ui.add_enabled_ui(logarithmic_supported, |ui| {
                            ui.selectable_value(
                                &mut selected_scale_type,
                                PriceScaleType::Logarithmic,
                                "Logarítmica",
                            );
                        });
                        ui.add_enabled_ui(percentage_base.is_some(), |ui| {
                            ui.selectable_value(
                                &mut selected_scale_type,
                                PriceScaleType::Percentage,
                                "Porcentual (%)",
                            );
                        });
                    });
                if selected_scale_type != current_scale_type {
                    let viewport = self.chart.viewport_mut();
                    if selected_scale_type == PriceScaleType::Percentage
                        && let Some(base) = percentage_base
                    {
                        let _ = viewport.set_price_scale_percentage_base(base);
                    }
                    let _ = viewport.set_price_scale_type(selected_scale_type);
                }
            });

            ui.horizontal(|ui| {
                let (top_margin, bottom_margin) = self.chart.viewport().price_scale_margins();
                let mut top_percent = top_margin * 100.0;
                let mut bottom_percent = bottom_margin * 100.0;

                ui.label("Margen superior:");
                let top_changed = ui
                    .add(
                        egui::DragValue::new(&mut top_percent)
                            .range(0.0..=45.0)
                            .suffix("%"),
                    )
                    .changed();

                ui.label("Inferior:");
                let bottom_changed = ui
                    .add(
                        egui::DragValue::new(&mut bottom_percent)
                            .range(0.0..=45.0)
                            .suffix("%"),
                    )
                    .changed();

                if ui.small_button("Restablecer 5%").clicked() {
                    self.chart.viewport_mut().reset_price_scale_margins();
                } else if top_changed || bottom_changed {
                    let _ = self
                        .chart
                        .viewport_mut()
                        .set_price_scale_margins(top_percent / 100.0, bottom_percent / 100.0);
                }
            });

            ui.separator();

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
                        self.chart
                            .viewport_mut()
                            .set_offset_x(current_offset - 1.0);
                    }
                }
            }
            self.follow_latest_pending = false;

            let pre_input_scale_range = {
                let pre_input_count = self.chart.time_scale().len();
                let pre_input_visible = self
                    .chart
                    .viewport()
                    .visible_range(pre_input_count, chart_layout.plot_width());
                if pre_input_visible.is_empty() {
                    None
                } else {
                    let points = self.chart.data_points();
                    let visible_points =
                        &points[pre_input_visible.first..=pre_input_visible.last];
                    visible_price_range(
                        visible_points,
                        self.chart.viewport().price_scale_mode(),
                    )
                }
            };
            if let Some((min_price, max_price)) = pre_input_scale_range {
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

            let data_revision = self.chart.data_revision();
            let drawing_width = chart_layout.plot_width();
            let drawing_height = chart_layout.plot_height();
            let logical_count = self.chart.time_scale().len();
            let visible_before_scale_update = self
                .chart
                .viewport()
                .visible_range(logical_count, drawing_width);

            let auto_scale_range = if self.chart.viewport().price_scale_auto()
                && !visible_before_scale_update.is_empty()
            {
                let points = self.chart.data_points();
                let visible_points = &points[visible_before_scale_update.first
                    ..=visible_before_scale_update.last];
                visible_price_range(
                    visible_points,
                    self.chart.viewport().price_scale_mode(),
                )
            } else {
                None
            };
            if let Some((min_price, max_price)) = auto_scale_range {
                self.chart
                    .viewport_mut()
                    .update_auto_price_scale(min_price, max_price);
            }

            let viewport = self.chart.viewport();
            let points = self.chart.data_points();
            let visible = viewport.visible_range(logical_count, drawing_width);

            let mask =
                self.calculate_invalidation_mask(viewport, crosshair_pos, data_revision, chart_rect);

            let max_volume = if visible.is_empty() {
                0.0
            } else {
                let visible_points = &points[visible.first..=visible.last];
                visible_points
                    .iter()
                    .map(|point| point.volume())
                    .fold(0.0_f64, f64::max)
            };
            let (price_range, center_price) = if visible.is_empty() {
                (0.0, 0.0)
            } else {
                viewport.price_scale_reference()
            };

            let scene = ChartScene {
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
        if self.prev_viewport.is_none() {
            return InvalidationMask {
                level: InvalidationReason::Full,
            };
        }

        let mut mask = InvalidationMask::none();

        if let Some(prev_rect) = self.prev_rect
            && prev_rect != rect
        {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::ScaleLayout,
            });
        }

        if let Some(prev_vp) = self.prev_viewport
            && *viewport != prev_vp
        {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::ScaleLayout,
            });
        }

        if data_revision != self.prev_data_revision {
            mask = mask.merge(InvalidationMask {
                level: InvalidationReason::SeriesData,
            });
        }

        if crosshair_pos != self.prev_crosshair_pos && mask.level == InvalidationReason::None {
            mask = InvalidationMask {
                level: InvalidationReason::CursorOnly,
            };
        }

        mask
    }
}

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

fn latest_closed_candle_open_time(interval_ms: i64) -> i64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64;
    let current_open = now_ms.div_euclid(interval_ms) * interval_ms;
    current_open.saturating_sub(interval_ms)
}

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
            refresh_price_scale_type_for_data(chart);
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

            tracing::warn!("ChartState vaciado: no se publicarán datos sintéticos tras un fallo de carga");
            LoadState::Error(e.to_string())
        }
    }
}

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

fn refresh_price_scale_type_for_data(chart: &mut ChartState) {
    let points = chart.data_points();
    let first_close = points.first().map(|point| point.close());
    let logarithmic_supported = points.iter().all(|point| point.low() > 0.0);
    let scale_type = chart.viewport().price_scale_type();

    if scale_type == PriceScaleType::Percentage {
        if let Some(base) = first_close.filter(|price| price.is_finite() && *price > 0.0) {
            let _ = chart.viewport_mut().set_price_scale_percentage_base(base);
        } else {
            let _ = chart
                .viewport_mut()
                .set_price_scale_type(PriceScaleType::Linear);
        }
    } else if scale_type == PriceScaleType::Logarithmic && !logarithmic_supported {
        let _ = chart
            .viewport_mut()
            .set_price_scale_type(PriceScaleType::Linear);
    }
}

fn visible_price_range(points: &[ChartDataPoint], mode: PriceScaleMode) -> Option<(f64, f64)> {
    let mut range: Option<(f64, f64)> = None;

    for point in points {
        let (point_min, point_max) = match mode {
            PriceScaleMode::HighLow => (point.low(), point.high()),
            PriceScaleMode::Close => (point.close(), point.close()),
        };
        if !point_min.is_finite() || !point_max.is_finite() {
            continue;
        }

        range = Some(match range {
            Some((min_price, max_price)) => (min_price.min(point_min), max_price.max(point_max)),
            None => (point_min, point_max),
        });
    }

    range
}

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

#[cfg(test)]
mod price_scale_mode_tests {
    use super::*;
    use crate::domain::candle::Candle;
    use crate::domain::chart::time_point::TimePoint;

    fn sample_points() -> Vec<ChartDataPoint> {
        let first_time = TimePoint::new(0, 60_000, 60_000).expect("intervalo válido");
        let second_time = TimePoint::new(60_000, 120_000, 60_000).expect("intervalo válido");
        vec![
            ChartDataPoint::new(
                Candle::new(100.0, 120.0, 80.0, 110.0, 500.0).expect("vela válida"),
                first_time,
            ),
            ChartDataPoint::new(
                Candle::new(112.0, 130.0, 90.0, 105.0, 700.0).expect("vela válida"),
                second_time,
            ),
        ]
    }

    #[test]
    fn high_low_auto_scale_includes_visible_wicks() {
        let points = sample_points();

        assert_eq!(
            visible_price_range(&points, PriceScaleMode::HighLow),
            Some((80.0, 130.0))
        );
    }

    #[test]
    fn close_auto_scale_ignores_wicks_outside_visible_closes() {
        let points = sample_points();

        assert_eq!(
            visible_price_range(&points, PriceScaleMode::Close),
            Some((105.0, 110.0))
        );
    }

    #[test]
    fn auto_scale_range_is_empty_without_valid_points() {
        assert_eq!(visible_price_range(&[], PriceScaleMode::HighLow), None);
    }
}


#[cfg(test)]
mod load_failure_tests {
    use super::*;
    use crate::market::errors::MarketError;

    struct FailingMarketDataProvider;

    impl MarketDataProvider for FailingMarketDataProvider {
        fn name(&self) -> &str {
            "failing-provider"
        }

        fn fetch_candles(
            &self,
            _symbol: Symbol,
            _from_ms: i64,
            _to_ms: i64,
        ) -> Result<Vec<MarketCandle>, MarketError> {
            Err(MarketError::Network("provider unavailable".to_string()))
        }
    }

    #[test]
    fn failed_market_load_does_not_publish_synthetic_data() {
        let mut chart = ChartState::new();
        let symbol = Symbol::from_str("BTC/USDT").expect("símbolo válido");

        let state = load_symbol_into_chart(
            &mut chart,
            &FailingMarketDataProvider,
            symbol,
            0,
            MOCK_INTERVAL_MS,
        );

        assert!(matches!(state, LoadState::Error(_)));
        assert!(chart.is_empty());
    }
}
