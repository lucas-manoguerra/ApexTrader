//! Datos de chart del dominio: vela OHLCV asociada a su punto temporal.
//!
//! Separa el dato financiero (OHLCV) de su punto temporal, permitiendo al
//! viewport establecer la relación entre timestamp e índice lógico sin
//! depender del proveedor de mercado ni de la UI.

pub mod chart_data;
pub mod time_point;
