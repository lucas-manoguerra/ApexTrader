#![allow(dead_code)]

// Esqueleto inicial: renderer wgpu y errores de trading se integran en
// etapas futuras. Se irá quitando a medida que cada módulo se active.

/// Biblioteca de ApexTrader.
///
/// Expone los módulos públicos para que los tests de integración y otros
/// crates puedan depender de ellos sin arrastrar el binario de escritorio.
pub mod application;
pub mod chart;
pub mod domain;
pub mod errors;
pub mod logging;
pub mod market;
pub mod renderer;
