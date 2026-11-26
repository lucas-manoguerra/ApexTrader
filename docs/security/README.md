# Seguridad

## Dependencias

- Mantener Cargo.lock para la aplicación.
- Ejecutar cargo-audit y cargo-deny en CI.
- Revisar licencias y advisories antes de incorporar crates.
- Preferir dependencias mantenidas y con superficie funcional acotada.

## Rust

- Preferir APIs seguras.
- Aislar cualquier uso de unsafe y exigir revisión especializada.
- Validar datos externos antes de convertirlos en tipos internos.
- No registrar secretos, credenciales ni datos sensibles en logs.

## Reporte

Los hallazgos de seguridad deben incluir impacto, condiciones de explotación, versiones afectadas y mitigación. Los secretos nunca deben incluirse en issues o commits.
