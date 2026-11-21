# Chart Engine: Ejes de precios y tiempo (S6)

**Objective:** Implementar ticks dinámicos en los ejes Y (precio) y X (tiempo/índice) que se ajusten al zoom, con formato legible y etiquetas alineadas a ambos ejes.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test chart_axes`

## Specs

**S6.1** — Eje Y (precio): mostrar ticks dinámicos ajustados al zoom.
- Acceptance criteria: al hacer zoom in/out, los ticks de precio se recalculan y muestran valores legibles con 2 decimales.

**S6.2** — Eje X (tiempo/índice): mostrar ticks dinámicos ajustados al zoom.
- Acceptance criteria: al hacer zoom in/out, los ticks de índice se recalculan mostrando índices de vela legibles.

**S6.3** — Formato de precio legible.
- Acceptance criteria: precios mostrados con 2 decimales fijos (ej. `52345.67`), sin notación científica.

**S6.4** — Etiquetas alineadas a los ejes.
- Acceptance criteria: etiquetas Y a la izquierda del área de dibujo, etiquetas X abajo del área de dibujo.

**S6.5** — Los ticks no se superponen ni saturan el eje.
- Acceptance criteria: cantidad de ticks razonable (5-10 por eje) independientemente del zoom.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S6.1 | inline | done | pendiente |
| T2 | S6.2 | inline | done | pendiente |
| T3 | S6.3 | inline | done | pendiente |
| T4 | S6.4 | inline | done | pendiente |
| T5 | S6.5 | inline | done | pendiente |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()`, `Decimal::to_string()` |
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Painter::text()`, `egui::FontId`, `egui::Align2`, `egui::Color32` |

## Log

**L1** (verbatim): Implementar ticks dinámicos en ejes Y (precio) y X (tiempo/índice) que se ajusten al zoom, con formato legible.

**L2**: `GlowChartRenderer::draw` delega el dibujo de ejes a `draw_price_axis` y `draw_time_axis`.

**L3**: `nice_tick_step()` calcula steps "nice" (1, 2, 5, 10, etc.) basados en el rango visible y cantidad objetivo de ticks.

**L4**: `draw_price_axis()` dibuja ticks de precio en el eje Y con formato de 2 decimales fijos usando `Price::value()`.

**L5**: `draw_time_axis()` dibuja índices de vela en el eje X con step dinámico según la cantidad de velas visibles.

**L6**: Los precios se convierten de f64 a Decimal para evitar errores de punto flotante en la etiqueta.

**L7**: Todos los 5 tests de chart_axes pasan, junto con los 43 tests existentes.

**L8**: `PriceAxisParams` struct agrupa los parámetros del eje de precio para evitar `too_many_arguments` en clippy.
