//! Sistema de interacción del chart.
//!
//! Traduce eventos de entrada normalizados (sin depender de egui) en
//! operaciones del `Viewport`: pan, zoom anclado al cursor, crosshair.
//! El estado del gesto se gestiona con anclas capturadas al inicio, evitando
//! el drift acumulado de aplicar deltas frame tras frame.

pub mod controller;
pub mod events;
pub mod options;

pub use options::{InteractionOptions, ScaleOptions, ScrollOptions};
