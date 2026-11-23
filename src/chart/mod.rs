//! Motor de transformaciones del chart.
//!
//! Este módulo concentra la lógica de layout, interacción, crosshair y
//! hit-testing del chart, separada del estado de escala y cámara del
//! viewport (`domain/viewport.rs`) y del renderizado (`renderer/`).
//! Representa el "Chart Engine" de la arquitectura objetivo: determina
//! cómo se comporta el gráfico y cómo responde a las interacciones, sin
//! conocer la GPU ni la UI.
//!
//! `Viewport` (`domain/viewport.rs`) es el único dueño del estado de escala
//! (`zoom_x`, `zoom_y`, `offset_x`, `offset_y`, `min_zoom`, `max_zoom`).
//! Este módulo no duplica ese estado.

pub mod coordinate;
pub mod crosshair;
pub mod hit_testing;
pub mod interaction;
pub mod layout;
pub mod visible_range;
