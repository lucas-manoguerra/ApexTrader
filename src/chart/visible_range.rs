//! Re-exporta el `VisibleRange` canónico definido en `domain::transforms`.
//!
//! Este módulo existió como una definición duplicada de `VisibleRange`.
//! Ahora apunta a la única fuente de verdad en `domain::transforms` para
//! evitar dos tipos independientes con el mismo nombre.

pub use crate::domain::transforms::VisibleRange;
