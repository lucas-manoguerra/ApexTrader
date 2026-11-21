# Dominio: Separación de espacios de coordenadas (H7 de auditoría)

**Objective:** Separar explícitamente los espacios de coordenadas en Viewport para evitar mezclar coordenadas de datos con coordenadas de pantalla, y corregir la inconsistencia entre el espaciamiento de velas en Viewport y el ancho de vela en glow.rs.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test coordinate-spaces --test coordinate_transforms --test auto_scale --test crosshair`

## Specs

**H7.1** — Definir espacios de coordenadas explícitos.
- Acceptance criteria: Existen tres espacios bien definidos: DataSpace (índice de vela, precio), ViewportSpace (estado de zoom/pan), ScreenSpace (píxeles de pantalla).

**H7.2** — `zoom_at()` usa coordenadas normalizadas 0..=1 de forma consistente.
- Acceptance criteria: `center_x` y `center_y` en `zoom_at` se interpretan como fracciones 0..=1 del área de pantalla, no como coordenadas de datos.

**H7.3** — Espaciamiento de velas consistente entre Viewport y renderer.
- Acceptance criteria: `data_x_to_screen` y el cálculo de `candle_width` en glow.rs usan la misma fórmula de espaciamiento.

**H7.4** — Transformaciones de ida y vuelta son consistentes.
- Acceptance criteria: `screen_x_to_data(data_x_to_screen(i, w), w) == i` y `screen_y_to_price(price_to_screen(p, ...), ...) ≈ p`.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | H7.1 | inline | pending | |
| T2 | H7.2 | inline | pending | |
| T3 | H7.3 | inline | pending | |
| T4 | H7.4 | inline | pending | |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()`, `Decimal::to_f64()` |

## Log

**L1** (verbatim): Separar espacios de coordenadas en Viewport: DataSpace, ViewportSpace, ScreenSpace. Corregir inconsistencia entre espaciamiento en Viewport y ancho de vela en renderer.

**L2**: `zoom_at()` interpreta `center_x` y `center_y` como fracciones 0..=1 del área de pantalla.

**L3**: `candle_spacing` y `candle_width` usan la misma fórmula base (20.0 píxeles por vela a zoom 1.0).

**L4**: Tests de transformaciones bidireccionales verifican consistencia.
