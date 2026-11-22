/// Punto de entrada de ApexTrader.
///
/// Toda la configuración y composición de dependencias vive en la capa
/// de aplicación (`application::run`). Este archivo solo delega en ella.
fn main() -> eframe::Result {
    apex_trader::application::run()
}
