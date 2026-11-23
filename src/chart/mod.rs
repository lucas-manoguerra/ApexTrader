//! Motor de transformaciones del chart.
//!
//! Este módulo concentra la lógica de escalas y coordenadas del chart,
//! separada del estado del viewport (`domain/viewport.rs`) y del
//! renderizado (`renderer/`). Representa el "Chart Engine" de la
//! arquitectura objetivo: determina cómo se comporta el gráfico y cómo
//! responde a las interacciones, sin conocer la GPU ni la UI.

pub mod coordinate;
pub mod crosshair;
pub mod hit_testing;
pub mod layout;
pub mod scale;
pub mod visible_range;
