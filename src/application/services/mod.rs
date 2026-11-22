//! Composición de dependencias de la capa de aplicación.
//!
//! Encapsula la creación de `ChartState`, el renderer, la carga de fuentes y
//! los datos de prueba de ejemplo. Aquí vive la `ApexTraderApp` que antes
//! estaba en `main.rs`.

use crate::application::events::{self, InputHandler};
use crate::domain::candle::Candle;
use crate::domain::chart_state::ChartState;
use crate::logging::init as init_logging;
use crate::renderer::{ChartRenderer, create_renderer};
use eframe::egui;
use tracing;

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

            let mut input_handler = InputHandler::new();

            // Procesar input de usuario (zoom, pan, crosshair).
            // El bloque limita el mutable borrow de self.chart para que
            // después se pueda prestar inmutablemente (candles, viewport).
            let crosshair_pos = {
                let chart = &mut self.chart;
                events::handle_input(&mut input_handler, ui, &response, interaction_rect, chart);
                input_handler.crosshair_pos()
            };

            // Después de manejar input, obtenemos el viewport inmutable para dibujar.
            let viewport = self.chart.viewport();

            // Prestamos las velas del ChartState para dibujarlas.
            // El renderer no las posee ni las modifica.
            let candles = self.chart.candles();
            if let Err(e) = self
                .renderer
                .draw(painter, available, candles, viewport, crosshair_pos)
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

/// Ejecuta la aplicación de escritorio (punto de entrada de la capa de
/// aplicación).
///
/// Esta función es invocada desde `application::run()` y contiene la lógica
/// que antes vivía directamente en `main.rs`.
pub fn run() -> eframe::Result {
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

            Ok(Box::new(ApexTraderApp { chart, renderer }))
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
