# Dominio: saneamiento de invariantes (H5, H6, H7 de auditoría)

**Objective:** Corregir defectos críticos del dominio identificados en la auditoría: Candle sin validación, Viewport con campos públicos, y transformaciones de coordenadas mixtas.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test candle_domain --test viewport_domain`

## Specs

**H5.1** — `Candle` tiene campos privados y constructor validado.
- Acceptance criteria: `Candle::new(open, high, low, close)` devuelve `Result<Candle, CandleError>`; rechazan `low > high`, `open` o `close` fuera de `[low, high]`, valores no finitos.

**H5.2** — `ChartState::add_candle` acepta `Candle` ya validado.
- Acceptance criteria: `add_candle` recibe `Candle` (no `f64` sueltos); no necesita volver a validar OHLC porque el tipo garantiza validez.

**H6.1** — `Viewport` tiene campos privados.
- Acceptance criteria: `zoom_level`, `offset_x`, `offset_y`, `zoom_step`, `min_zoom`, `max_zoom` son privados; acceso mediante getters y setters validados.

**H6.2** — `Viewport::zoom_at` ignora `delta == 0.0`.
- Acceptance criteria: llamar `zoom_at(0.0, x, y)` no modifica `zoom_level` ni los offsets.

**H6.3** — `Viewport` valida finitud de parámetros en `zoom_at`.
- Acceptance criteria: `zoom_at` con `delta`, `center_x` o `center_y` no finitos no modifica el estado (o devuelve error).

**H6.4** — `Viewport::min_zoom` no puede ser mayor que `max_zoom`.
- Acceptance criteria: setter de `min_zoom` y `max_zoom` mantienen `min_zoom <= max_zoom`.

**H7.1** — Separación explícita de espacios de coordenadas.
- Acceptance criteria: `data_x_to_screen` y `price_to_screen` operan sobre coordenadas de datos; `screen_x_to_data` y `screen_y_to_price` son las inversas; no hay mezcla de coordenadas normalizadas con datos en el mismo cálculo.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | H5.1 | inline | pending | |
| T2 | H5.2 | inline | pending | |
| T3 | H6.1 | inline | pending | |
| T4 | H6.2 | inline | pending | |
| T5 | H6.3 | inline | pending | |
| T6 | H6.4 | inline | pending | |
| T7 | H7.1 | inline | pending | |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()`, `Decimal::is_finite()` |
| thiserror | 2 | https://docs.rs/thiserror/2.0.21/ | `#[derive(Error)]` |

## Log

**L1** (verbatim): Saneamiento del dominio: Candle con invariantes, Viewport encapsulado, separación de espacios de coordenadas.
