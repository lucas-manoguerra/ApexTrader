//! Trait para objetos que pueden ser testeados por hit-testing.
//!
//! Cualquier elemento visual (vela, serie, dibujo, marcador, overlay,
//! etiqueta de eje) que quiera recibir eventos de clic o hover debe
//! implementar este trait.

use crate::chart::hit_testing::result::{HitResult, HitTestContext};

/// Objeto visual que puede responder a un hit-test.
///
/// Implementar este trait permite que el `HitTester` determine si el
/// cursor está sobre este elemento, sin que el dominio conozca `egui`
/// ni `wgpu`.
pub trait HitTestable {
    /// Ejecuta el hit-test contra este elemento.
    ///
    /// Devuelve `Some(HitResult)` si el cursor está sobre el elemento
    /// (dentro de su tolerancia de selección), o `None` si no.
    fn hit_test(&self, ctx: &HitTestContext) -> Option<HitResult>;
}
