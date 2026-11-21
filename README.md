# ApexTrader

Terminal de trading de escritorio escrita en Rust, construida con egui (interfaz) y wgpu (renderizado GPU). El objetivo es una terminal rápida, modular y fácil de extender, con los gráficos de velas como pieza central.

## Estado

Proyecto en fase inicial. Versión `0.1.0` (experimental).

## Arquitectura en resumen

- `src/domain` — reglas de negocio, tipos e invariantes (independiente de UI y proveedores).
- `src/renderer` — preparación y dibujo de elementos gráficos (wgpu / glow).
- `src/errors` — errores tipados con `thiserror`.
- `src/logging` — logging estructurado con `tracing`.

## Cómo ejecutar

```bash
cargo run
```

## Calidad del código

Antes de integrar cualquier cambio:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Documentación

- `docs/` — estándares de ingeniería, arquitectura, Git y CI.
- `docs/decisions/` — ADR (Architecture Decision Records).

## Licencia

Privado — proyecto interno de ApexTrader.