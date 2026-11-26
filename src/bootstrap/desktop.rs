use crate::logging::init as init_logging;
use crate::presentation::desktop::app::ApexTraderApp;

pub fn run() -> eframe::Result {
    if let Err(error) = init_logging() {
        eprintln!("Advertencia: no se pudo inicializar logging: {}", error);
    }

    tracing::info!("Iniciando ApexTrader");

    eframe::run_native(
        "ApexTrader",
        eframe::NativeOptions::default(),
        Box::new(|creation_context| Ok(Box::new(ApexTraderApp::new(creation_context)))),
    )
}
