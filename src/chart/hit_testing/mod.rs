//! Sistema de hit-testing del chart.
//!
//! Permite detectar qué elemento visual (vela, serie, dibujo, marcador,
//! overlay, etiqueta de eje) está bajo una posición de pantalla, sin que
//! el dominio conozca `egui` ni `wgpu`.
//!
//! Flujo:
//! 1. La capa de aplicación construye un `HitTestContext` a partir del
//!    viewport, layout y posición del cursor.
//! 2. El `HitTester` itera sobre los objetos `HitTestable` ordenados por
//!    z-order y devuelve el mejor hit.

pub mod hit_tester;
pub mod result;
pub mod r#trait;

pub use hit_tester::{HitTesterResult, hit_test_all, hit_test_objects};
pub use result::{
    HitAxis, HitDrawingId, HitKind, HitMarkerId, HitOverlayId, HitResult, HitSeriesId,
    HitTestContext,
};
pub use r#trait::HitTestable;
