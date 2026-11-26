pub mod commands;
pub mod events;
pub mod services;

pub fn run() -> eframe::Result {
    services::run()
}
