//! Capa de aplicación de ApexTrader.
//!
//! Esta capa coordina los casos de uso del usuario (zoom, pan, crosshair,
//! renderizado) sin que el dominio conozca cómo se ejecutan las acciones.
//! Mantiene la separación:
//! - `commands`: casos de uso (zoom, pan, crosshair).
//! - `events`: interpretación de entrada de usuario (mouse, scroll).
//! - `services`: composición de dependencias (ChartState, renderer, fuentes,
//!   datos de ejemplo).
//!
//! El punto de entrada `run()` es invocado por `main.rs`.

pub mod commands;
pub mod events;
pub mod services;

/// Ejecuta la aplicación de escritorio.
///
/// Este es el único punto de entrada expuesto a `main.rs`. Encapsula la
/// inicialización de `eframe`, la composición de dependencias y el bucle de
/// renderizado.
pub fn run() -> eframe::Result {
    services::run()
}
