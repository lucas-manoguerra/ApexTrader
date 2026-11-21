# Chart Engine: Crosshair (S5)

**Objective:** Implementar un crosshair (crucero) que sigue el cursor del mouse sobre el chart, mostrando una línea vertical y horizontal, junto con el precio y el tiempo bajo el cursor.

**Branch:** `feature/chart-engine-mvp`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test --test crosshair`

## Specs

**S5.1** — El renderer dibuja una línea vertical en la posición X del crosshair.
- Acceptance criteria: `draw()` con `crosshair_pos = Some((x, y))` dibuja una línea vertical en `x` dentro del área del chart.

**S5.2** — El renderer dibuja una línea horizontal en la posición Y del crosshair.
- Acceptance criteria: `draw()` con `crosshair_pos = Some((x, y))` dibuja una línea horizontal en `y` dentro del área del chart.

**S5.3** — El crosshair muestra el precio bajo el cursor.
- Acceptance criteria: el renderer usa `screen_y_to_price` para calcular el precio y muestra una etiqueta.

**S5.4** — El crosshair muestra el tiempo/índice bajo el cursor.
- Acceptance criteria: el renderer usa `screen_x_to_data` para calcular el índice de vela y muestra una etiqueta.

**S5.5** — Sin crosshair (None), no se dibujan líneas ni etiquetas.
- Acceptance criteria: `draw()` con `crosshair_pos = None` no dibuja líneas cruzadas ni etiquetas de crosshair.

**S5.6** — `main.rs` rastrea la posición del mouse y pasa el crosshair al renderer.
- Acceptance criteria: `ApexTraderApp` almacena la posición del mouse y la pasa al `draw()` del renderer.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S5.1 | inline | pending | |
| T2 | S5.2 | inline | pending | |
| T3 | S5.3 | inline | pending | |
| T4 | S5.4 | inline | pending | |
| T5 | S5.5 | inline | pending | |
| T6 | S5.6 | inline | pending | |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Painter::line_segment()`, `egui::Painter::text()`, `egui::pos2()`, `egui::Stroke`, `egui::Color32`, `egui::FontId`, `egui::Align2` |
| rust_decimal | 1.43 | https://docs.rs/rust_decimal/1.43.0/ | `Decimal::new()` |

## Log

**L1** (verbatim): Implementar crosshair que sigue el mouse sobre el chart con líneas vertical/horizontal y etiquetas de precio y tiempo.

**L2**: `ChartRenderer::draw` agrega parámetro `crosshair_pos: Option<(f64, f64)>` con coordenadas relativas al área del chart.

**L3**: `GlowChartRenderer::draw` dibuja línea vertical y horizontal usando `painter.line_segment()`, y etiquetas con `painter.text()`.

**L4**: `ApexTraderApp` rastrea `response.hover_pos()` y normaliza a coordenadas relativas al `interaction_rect`.

**L5**: Tests en `tests/crosshair.rs` verifican dibujo de líneas, etiquetas de precio/tiempo, y comportamiento con None.
