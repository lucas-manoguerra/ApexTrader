# Chart Engine: Zoom y pan con límites de datos (S7)

**Objective:** Agregar límites al pan y zoom del viewport para que el usuario no pueda hacer pan fuera del rango de velas cargadas ni hacer zoom extremo que oculte o agrande excesivamente una sola vela.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test chart-zoom-pan-limits`

## Specs

**S7.1** — Pan horizontal limitado al rango de velas.
- Acceptance criteria: Después de `pan()`, `offset_x` no permite que la primera vela (index 0) quede fuera de la pantalla por la izquierda ni la última vela (index N-1) por la derecha.

**S7.2** — Zoom máximo limitado.
- Acceptance criteria: `set_zoom_level()` y `zoom_at()` no permiten un zoom tan alto que el ancho de una vela (`candle_spacing_base / zoom_level`) exceda el ancho de pantalla útil.

**S7.3** — Zoom mínimo limitado.
- Acceptance criteria: `set_zoom_level()` y `zoom_at()` no permiten un zoom tan bajo que menos de 2 velas quepan en la pantalla.

**S7.4** — Integración en main.rs.
- Acceptance criteria: Después de cada `zoom_at` o `pan` en `main.rs`, se llaman los métodos de límite para mantener el viewport dentro de los datos.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S7.1 | inline | done | pendiente |
| T2 | S7.2 | inline | done | pendiente |
| T3 | S7.3 | inline | done | pendiente |
| T4 | S7.4 | inline | done | pendiente |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()` |
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Painter`, `egui::Rect` |

## Log

**L1** (verbatim): Agregar límites al pan y zoom del viewport para que el usuario no pueda hacer pan fuera del rango de velas cargadas ni hacer zoom extremo.

**L2**: `Viewport::clamp_pan_to_candles(candle_count, screen_width)` limita offset_x para que primera y última vela queden visibles dentro de un margen del 5%.

**L3**: `Viewport::clamp_zoom_to_candles(candle_count, screen_width)` limita zoom_level: máximo 5.0 (una vela no llena toda la pantalla) y mínimo 40.0/screen_width (al menos 2 velas quepan).

**L4**: `main.rs` llama a `clamp_zoom_to_candles` y `clamp_pan_to_candles` después de cada `zoom_at` y `pan`.

**L5**: 8 tests en `tests/chart-zoom-pan-limits.rs` verifican pan izquierdo/derecho, zoom máximo/mínimo, pantalla pequeña, y una sola vela.

**L6**: Todos los 51 tests pasan, clippy -D warnings clean, fmt clean.
