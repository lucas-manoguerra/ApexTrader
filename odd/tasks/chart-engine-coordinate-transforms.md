# Chart Engine: transformaciones de coordenadas bidireccionales (S3)

**Objective:** Implementar conversiones bidireccionales entre coordenadas de datos (precio, tiempo) y coordenadas de pantalla (píxeles), usando los tipos decimal `Price` y `PriceRange` para evitar errores de punto flotante.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test coordinate_transforms`

## Specs

**S3.1** — `data_x_to_screen` convierte índice de vela a píxel X de pantalla.
- Acceptance criteria: `Viewport::data_x_to_screen(index, screen_width)` devuelve `f64`; índice 0 en centro con zoom 1.0 da `screen_width / 2.0`. ✅ Verificado con test.

**S3.2** — `screen_x_to_data` convierte píxel X de pantalla a índice de vela (inverso de S3.1).
- Acceptance criteria: `Viewport::screen_x_to_data(screen_x, screen_width)` devuelve `usize`; `screen_x_to_data(data_x_to_screen(i, w), w) == i` para índices válidos. ✅ Verificado con test.

**S3.3** — `price_to_screen` convierte `Price` a píxel Y de pantalla usando `PriceRange`.
- Acceptance criteria: `Viewport::price_to_screen(price, height, range, center)` devuelve `f64`; precio por encima del centro da Y menor (eje invertido). ✅ Verificado con test.

**S3.4** — `screen_y_to_price` convierte píxel Y de pantalla a `Price` (inverso de S3.3).
- Acceptance criteria: `Viewport::screen_y_to_price(y, height, range, center)` devuelve `Price`; `screen_y_to_price(price_to_screen(p, h, r, c), h, r, c) ≈ p`. ✅ Verificado con test.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S3.1 | inline | done | e8e7a53 |
| T2 | S3.2 | inline | done | e8e7a53 |
| T3 | S3.3 | inline | done | e8e7a53 |
| T4 | S3.4 | inline | done | e8e7a53 |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::to_f64()`, `Decimal::from_f64()`, `Decimal::new()`, `rust_decimal::prelude::ToPrimitive`, `rust_decimal::prelude::FromPrimitive` |

## Log

**L1** (verbatim): Implementar transformaciones de coordenadas bidireccionales (data ↔ screen) para el chart engine, usando tipos decimal `Price` y `PriceRange` para evitar errores de punto flotante en precios financieros.

**L2**: `Viewport::data_x_to_screen` cambió de `index: f64` a `index: usize` (más semántico para conteo de velas).

**L3**: Agregado `Viewport::screen_x_to_data` como inverso de `data_x_to_screen`.

**L4**: `Viewport::price_to_screen` actualizado para recibir `Price` y `PriceRange` en lugar de `f64` crudo.

**L5**: Agregado `Viewport::screen_y_to_price` como inverso de `price_to_screen`, con fallback a `center_price` si el resultado no es un precio válido.

**L6**: `src/renderer/glow.rs` actualizado para llamar a `data_x_to_screen` con índice `usize`.

**L7**: Todos los 4 gates pasan: fmt, check, clippy -D warnings, test.
