# Chart Engine Modernization

Objective: modernize ApexTrader's Chart Engine following Lightweight Charts engineering decisions, correcting AT-001 through AT-010 and establishing TimeScale, PriceScale, ChartLayout, interaction, rendering and extension contracts with clear module boundaries. Idiomatic Rust, no TypeScript/Canvas port.

Branch: feature/chart-engine-modernization

Delivery strategy: ask-on-risk (default)

Test runner: cargo test

## Specs

S1: "un zoom alrededor del cursor debe conservar exactamente el punto de datos anclado, dentro de la tolerancia numérica definida." Cursor-anchored zoom must preserve the data point under the cursor on both axes.

S2: "el costo de actualizar un gráfico no debe crecer proporcionalmente con todo el histórico cuando solo cambia el cursor." Crosshair movement must not trigger full-history recalculation.

S3: "debemos poder probar TimeScale, PriceScale, coordenadas, zoom, pan y hit-testing sin inicializar una ventana ni una GPU." Chart engine must be testable without a window or GPU.

S4: AT-001 — "La conversión del desplazamiento vertical a píxeles utiliza `screen_height / zoom_y`, mientras que la transformación de precio a pantalla representa el efecto del desplazamiento mediante `screen_height * zoom_y`." Vertical offset calculation must be consistent with price-to-screen transformation.

S5: AT-003 — "Cuando `first > last`, `visible_range()` devuelve `{ first: 0, last: 0 }`." Empty visible range must never be interpreted as containing candle zero; must return a real empty range.

S6: AT-002 — "las coordenadas del input y las del renderer no tienen una fuente de verdad común." Input and renderer must share a single authoritative ChartLayout.

S7: AT-006 — "no puede representar correctamente fechas, sesiones, intervalos irregulares, huecos temporales ni sincronización por tiempo real." Time information (timestamp, interval) must be preserved from MarketCandle through to domain chart data.

S8: AT-005 — "el procesamiento del rango sigue siendo O(N total) por frame." Visible range must be computed before min/max/volume; no full-history scan per frame.

S9: AT-008 — "Parte de la lógica de zoom y offsets está duplicada. Incluso `ChartScales::zoom_at()` reproduce el mismo enfoque matemático problemático del zoom vertical." One authoritative owner for scale state, visible range and transformations; no competing implementations.

S10: AT-004 — "El eje temporal calcula sus índices visibles mediante una fórmula que no coincide con la transformación de coordenadas de `domain::transforms`." Time axis must use the same VisibleRange and transformer as candles.

S11: AT-007 — wgpu must be either genuinely functional or clearly isolated as unfinished and cannot be mistaken for a working backend.

S12: The chart domain and engine must not depend on `egui`, `wgpu` or `winit`.

S13: `Price` uses `Decimal`, while `Candle` stores `f64`. "Lo importante es que la conversión sea explícita y que no perdamos precisión antes de tiempo." Conversion between domain precision and renderer precision must be explicit.

## Tasks

| ID | Specs | Route | Status | Commit |
|----|-------|-------|--------|--------|
| T1 | S4, S5, S9, S10, S12, S13 | inline | done | 4db2d76 |
| T2 | S1, S9, S12 | inline | done | 4db2d76 |
| T3 | S5, S12 | inline | done | 4db2d76 |
| T4 | S6, S12 | inline | done | 4a6cfea |
| T5 | S1, S9, S12 | inline | done | c5b1b2f |
| T6 | S7, S12 | inline | done | ac67b37 |
| T7 | S1, S5, S8, S12 | inline | done | c5b1b2f |
| T8 | S2, S12 | inline | done | 3fb7672 |
| T9 | S3, S12 | inline | done | c9f3ae5 |
| T10 | S3, S9, S12 | inline | done | fa71a69 |
| T11 | S9, S12 | inline | done | cdf9c6d |
| T12 | S11 | inline | pending | — |
| T13 | S3, S12 | writer | pending | — |
| T14 | S3, S8, S12 | inline | pending | — |
| T15 | S12, S13 | inline | pending | — |
| T16 | S12 | inline | pending | — |

T1: Consolidate VisibleRange into a single canonical type with `empty()` and `is_empty()`.
T2: Fix viewport vertical offset in zoom_at to use consistent pixels-per-unit math (price_to_screen convention).
T3: Make visible_range return true empty range when first > last; handle empty datasets.
T4: Introduce ChartLayout (full rect, plot rect, price-axis rect, time-axis rect, volume pane rect) used by both input and renderer.
T5: Fix time axis to use the same VisibleRange and transforms as candles (remove independent formula).
T6: Preserve timestamp/time-interval through MarketCandle -> domain chart data (separate OHLCV from temporal point).
T7: Renderer computes visible range first; min/max/volume from visible range or precomputed indexes, not full-history scan.
T8: Interaction engine: pan/zoom with explicit start/update/finish, cursor-anchored zoom, no cumulative delta drift.
T9: Crosshair model using same transforms and scale state as renderer; no duplicated formulas.
T10: Hit-testing abstraction for series, candles, drawings, markers, axis labels.
T11: Remove duplicated scale models (ChartScales/Viewport/TimeScale/PriceScale consolidation); document ownership.
T12: Isolate wgpu as unfinished/experimental; do not present as functional backend; no pretend surface.
T13: Rendering architecture: ChartScene/RenderFrame/RenderGeometry/RenderLayer intermediate representation; egui renderer functional; wgpu separated.
T14: Invalidation mask: full, scale/layout, series data, overlay, cursor-only distinction.
T15: Typed, validated, extensible options architecture (chart, time scale, price scale, pane, series, crosshair).
T16: Panes foundation: multiple independent panes, independent heights, multiple series per pane, independent price scales.

## Referencias oficiales de dependencias

Referencias de API para las decisiones de implementación descritas en esta tarea
(consultar la versión fijada por Cargo.lock al trabajar en cada cambio):

- egui: https://docs.rs/egui/0.37/egui/
- wgpu: https://docs.rs/wgpu/30/wgpu/
- winit: https://docs.rs/winit/0.30.13/winit/
- proptest: https://docs.rs/proptest/1/proptest/
- rust_decimal: https://docs.rs/rust_decimal/1.43/rust_decimal/

## Log

L1: Auditoría de ApexTrader y estudio de TradingView Lightweight Charts. Ejecutar modernización del Chart Engine por etapas (núcleo matemático/temporal → interacción → renderizado → extensibilidad), manteniendo el proyecto verificable en cada etapa. Criterios de aceptación: precisión matemática del zoom anclado, escalabilidad sin O(N_total) por frame, independencia de ventana/GPU para pruebas.

L2: Commits granulares T1-T3 (4db2d76): VisibleRange canónico en domain::transforms con empty()/new()/Default; chart::visible_range como re-export; viewport usa VisibleRange::empty(); offset vertical en zoom_at/pan corregido a screen_height * zoom_y (consistente con price_to_screen) para zoom anclado al cursor (AT-001, AT-004, S4, S5).

L3: Commit S13 (40b41f8): Candle getores pasan de self a &self.

L4: Commits T6 (ac67b37, f727484, bf0c248, 295c0e7, 9cafba4, b25436d): domain::chart con ChartDataPoint y TimePoint; MarketCandle::to_chart_data_point preserva timestamp/intervalo; ChartState usa ChartDataPoint; LoadCandles con rejections explícitas; servicios usan ChartDataPoint; domain/mod.rs declara módulo chart.

L5: Commits T4 (4a6cfea, c9f3ae5, f0d2907, c5b1b2f, 7be00cc): ChartLayout única fuente de verdad; renderer usa ChartLayout + ChartDataPoint + O(N_visible); trait de renderer usa ChartDataPoint; input usa ChartLayout para coordenadas normalizadas; time axis usa viewport.visible_range (AT-002, AT-004, AT-005, S6).

L6: Commits T9 (c9f3ae5) y T10 (fa71a69): modelo de crosshair sin fórmulas duplicadas; hit-testing con HitTestable/HitTester/HitTestContext/HitResult.

L7: Commit T8 (3fb7672): sistema de interacción pan/zoom con Controller y Events, anclas al inicio para evitar drift acumulado.

L8: Tests actualizados (2019fd9): architecture.rs usa ChartDataPoint; coordinate_transforms.rs agrega zoom_at_y_preserves_cursor_price y pan_y_is_consistent_with_price_transforms; market_proptest.rs usa data_points.

L9: Assets (4b05272): capturas de referencia de pantalla.

L10: Todos los tests pasan (110 tests, 0 failures). T1-T10 completos.

L11: Commits T11 (bfc5d7c, f86ced4, 3a3d86a, 98ae57a, cdf9c6d): consolidación de modelos de escala duplicados.
- Viewport (domain/viewport.rs) es el único dueño del estado de escala: zoom_x, zoom_y, offset_x, offset_y, min_zoom, max_zoom.
- Se eliminó chart/scale.rs con ChartScales/TimeScale/PriceScale (duplicaban el estado de Viewport).
- Tests de independencia de ejes y clamping de zoom movidos a domain::viewport::tests.
- Documentación de ownership agregada en viewport.rs (sección ## Ownership) y chart/mod.rs.
- Comentarios de doc actualizados en viewport.rs, chart/mod.rs, chart_data.rs y market/candle.rs que referenciaban TimeScale/PriceScale como tipos separados.

Siguiente: T12 (wgpu aislado como experimental), T13-T16 pendientes.
