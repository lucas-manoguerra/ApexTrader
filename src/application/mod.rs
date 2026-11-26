pub mod commands;
pub mod services;

pub fn run() -> eframe::Result {
    services::run()
}
