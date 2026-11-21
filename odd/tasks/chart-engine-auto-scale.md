# Chart Engine: autoescala automática (S4)

**Objective:** Implementar ajuste automático de zoom y offsets del viewport para que todas las velas quepan en el área de dibujo con un margen del 5% en cada lado, usando los tipos decimal `Price` y `PriceRange`.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test auto_scale`

## Specs

**S4.1** — `auto_scale` ajusta `zoom_level` dentro de `[min_zoom, max_zoom]`.
- Acceptance criteria: después de `auto_scale`, `viewport.zoom_level >= min_zoom` y `<= max_zoom`. ✅ Verificado con test.

**S4.2** — `auto_scale` con una sola vela produce un zoom razonable (no extremo).
- Acceptance criteria: `zoom_level > 0.0` y `<= max_zoom` cuando todas las velas tienen el mismo precio. ✅ Verificado con test.

**S4.3** — Después de `auto_scale`, precios mínimo y máximo mapean dentro de los márgenes de pantalla.
- Acceptance criteria: `price_to_screen(min)` y `price_to_screen(max)` devuelven Y dentro de [5%, 95%] del alto. ✅ Verificado con test.

**S4.4** — El precio central del rango mapea al centro vertical de la pantalla.
- Acceptance criteria: `price_to_screen(mid_price, ..., mid_price)` ≈ `screen_height / 2.0`. ✅ Verificado con test.

**S4.5** — Más velas implican mayor zoom para que quepan horizontalmente.
- Acceptance criteria: `auto_scale` con 1000 velas produce `zoom_level` mayor que con 10 velas. ✅ Verificado con test.

**S4.6** — Primera y última vela mapean dentro del margen horizontal.
- Acceptance criteria: `data_x_to_screen(0)` y `data_x_to_screen(N-1)` dentro de [5%, 95%] del ancho. ✅ Verificado con test.

**S4.7** — `auto_scale` es idempotente (mismos parámetros → mismo zoom).
- Acceptance criteria: llamar dos veces con los mismos argumentos da el mismo `zoom_level`. ✅ Verificado con test.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S4.1 | inline | done | pendiente |
| T2 | S4.2 | inline | done | pendiente |
| T3 | S4.3 | inline | done | pendiente |
| T4 | S4.4 | inline | done | pendiente |
| T5 | S4.5 | inline | done | pendiente |
| T6 | S4.6 | inline | done | pendiente |
| T7 | S4.7 | inline | done | pendiente |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()`, `Decimal::is_zero()`, `Decimal::to_f64()`, `rust_decimal::prelude::ToPrimitive` |

## Log

**L1** (verbatim): Implementar autoescala automática del viewport para que todas las velas quepan en pantalla con margen del 5%.

**L2**: `Viewport::auto_scale(price_range, candle_count, screen_width, screen_height)` ajusta `zoom_level` y `offset_x`. `offset_y` se deja en 0 (el llamante pasa `center_price` como punto medio).

**L3**: El centrado horizontal usa `offset_x = -(N-1)/2` para que primera y última vela queden simétricas alrededor de `center_x`.

**L4**: Si el rango de precios es 0 (mismo precio en todas las velas), se usa $0.01 como rango mínimo para evitar división por cero.

**L5**: Todos los 7 tests pasan, clippy -D warnings clean, fmt clean.
