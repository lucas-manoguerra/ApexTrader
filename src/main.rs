#![allow(dead_code)]

// Esqueleto inicial: renderer wgpu y errores de trading se integran en
// etapas futuras. Se irá quitando a medida que cada módulo se active.

use apex_trader::domain::candle::Candle;
use apex_trader::domain::chart_state::ChartState;
use apex_trader::logging::init as init_logging;
use apex_trader::renderer::ChartRenderer;
use apex_trader::renderer::create_renderer;
use eframe::egui;

/// Aplicación de escritorio con egui.
///
/// Solo coordina: crea el `ChartState` (dueño de los datos y el viewport),
/// crea el renderer (wgpu o glow) y le pide al renderer que dibuje prestando
/// esos datos (`&[Candle]` y `&Viewport`).
///
/// Maneja el input de usuario: scroll para zoom, arrastre para pan.
struct ApexTraderApp {
    chart: ChartState,
    renderer: Box<dyn ChartRenderer>,
    crosshair_pos: Option<(f64, f64)>,
}

impl eframe::App for ApexTraderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Pedimos repintado continuo para que el chart se mantenga vivo.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("ApexTrader — Chart de Velas");
            ui.separator();

            // Mostrar qué renderer está activo
            ui.label(format!("Renderer: {}", self.renderer.name()));

            ui.separator();

            // Área interactiva para el chart: aquí detectamos scroll y arrastre.
            let available = ui.available_rect_before_wrap();
            let painter = ui.painter();

            // Creamos un área interactiva que cubre todo el panel disponible.
            let interaction_rect = available.shrink(0.0);
            let response = ui.interact(
                interaction_rect,
                ui.scope_id().with("chart_area"),
                egui::Sense::click_and_drag(),
            );

            // --- Manejo de input: Zoom con scroll ---
            if response.hovered() {
                let scroll_delta = ui.input(|i| i.smooth_scroll_delta);
                if scroll_delta.y != 0.0 {
                    // El centro del zoom es la posición actual del mouse.
                    if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                        // Normalizamos la posición del mouse a 0..=1 dentro del rect.
                        let center_x = (pos.x - interaction_rect.left()) / interaction_rect.width();
                        let center_y = (pos.y - interaction_rect.top()) / interaction_rect.height();

                        let candle_count = self.chart.len();
                        let screen_width = available.width() as f64;
                        let screen_height = available.height() as f64;

                        let vp = self.chart.viewport_mut();
                        vp.zoom_at(
                            scroll_delta.y as f64,
                            center_x as f64,
                            center_y as f64,
                            screen_width,
                            screen_height,
                        );
                        vp.clamp_zoom_to_candles(candle_count, screen_width);
                        vp.clamp_pan_to_candles(candle_count, screen_width);
                    }
                }

                // --- Manejo de input: Pan con arrastre ---
                if response.dragged() {
                    let delta = response.drag_delta();

                    // Limitar pan al rango de velas cargadas.
                    let candle_count = self.chart.len();
                    let screen_width = available.width() as f64;

                    let vp = self.chart.viewport_mut();
                    vp.pan(delta.x as f64, delta.y as f64, available.height() as f64);
                    vp.clamp_pan_to_candles(candle_count, screen_width);
                }

                // --- Manejo de input: Crosshair sigue el cursor ---
                if response.hovered()
                    && let Some(pos) = ui.input(|i| i.pointer.hover_pos())
                {
                    // Coordenadas relativas al área de interacción (0..=width, 0..=height).
                    let rel_x = pos.x - interaction_rect.left();
                    let rel_y = pos.y - interaction_rect.top();
                    self.crosshair_pos = Some((rel_x as f64, rel_y as f64));
                }
            }

            // Después de manejar input, obtenemos el viewport inmutable para dibujar.
            let viewport = self.chart.viewport();

            // Prestamos las velas del ChartState para dibujarlas.
            // El renderer no las posee ni las modifica.
            let candles = self.chart.candles();
            if let Err(e) =
                self.renderer
                    .draw(painter, available, candles, viewport, self.crosshair_pos)
            {
                tracing::error!("Error renderizando chart: {}", e);
                ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
            }

            // Mostrar estado del viewport (zoom/pan) para depuración.
            ui.label(format!(
                "Zoom: {:.2}x | Offset: ({:.2}, {:.2})",
                viewport.zoom_level(),
                viewport.offset_x(),
                viewport.offset_y()
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(format!("Velas: {}", candles.len()));
                ui.label("Scroll: zoom | Arrastre: pan");
            });
        });
    }
}

fn main() -> eframe::Result {
    // Inicializar logging antes de cualquier otra operación.
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
            let mut price = 100.0;

            for i in 0..20 {
                let open = price;
                let change = (i as f64 * 0.5).sin() * 3.0;

                // high debe ser >= max(open, close), low debe ser <= min(open, close)
                let close = open + change * 0.7;
                let high = open.max(close) + change.abs() * 0.3;
                let low = open.min(close) - change.abs() * 0.3;

                if let Ok(candle) = Candle::new(open, high, low, close) {
                    if let Err(e) = chart.add_candle(candle) {
                        tracing::error!("No se pudo agregar vela {}: {}", i, e);
                    }
                } else {
                    tracing::error!("No se pudo crear vela {}: precios inválidos", i);
                }
                price = close; // la siguiente vela abre donde cerró la anterior
            }

            tracing::info!("ChartState inicializado con {} velas", chart.len());

            // Crear renderer (intenta wgpu, cae a glow si falla)
            let renderer = create_renderer();

            Ok(Box::new(ApexTraderApp {
                chart,
                renderer,
                crosshair_pos: None,
            }))
        }),
    )
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
