# Profesionalizar el Chart Engine — Etapa 3

Objetivo: consolidar las fronteras del chart engine (escalas independientes,
autoescala robusta, zoom anclado al cursor, crosshair preciso) antes de
incorporar datos de mercado reales. Seguir el flujo Input → Chart Controller →
Chart Engine → Viewport / Chart Domain → Nuevo estado → Renderer.
Branch: `feature/profesionalizar-chart-engine`
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (autoescala robusta): `auto_scale` valida rangos degenerados y dimensiones
inválidas; centra horizontal y verticalmente todas las velas con margen 5%.
S2 (zoom anclado al cursor): `zoom_at` mantiene el punto bajo el cursor fijo
en pantalla antes y después del zoom.
S3 (crosshair preciso): `screen_y_to_price` y `screen_x_to_data` devuelven
el precio e índice correctos bajo el cursor, usando el área de dibujo útil.
S4 (escalas independientes): el eje X y el eje Y escalan de forma
independiente; el zoom vertical no distorsiona el horizontal y viceversa.
S5 (visible_range): el renderer itera solo velas visibles (ya implementado en
Etapa 1, se conserva y se cubre con pruebas).
S6 (tests de propiedades): proptest cubre auto_scale, zoom, pan, round-trip
coordenadas y límites del viewport.

## Tasks

- T1: Crear módulo `chart/` con `scale.rs`, `coordinate.rs` y `visible_range.rs`
  como motor de transformaciones independiente. Route: inline. Status: done.
  Commit: 4167f8f.
- T2: Mejorar `auto_scale` con validación de rangos degenerados y centrado
  bidireccional (S1). Route: inline. Status: done. Commit: 93cbe59.
- T3: Mejorar `zoom_at` para anclar el zoom al cursor (S2). Route: inline.
  Status: done. Commit: 93cbe59.
- T4: Mejorar `screen_y_to_price` / `screen_x_to_data` para crosshair preciso
  usando el área de dibujo útil (S3). Route: inline. Status: done.
  Nota: el renderer (`glow.rs`) pasa `drawing_width`/`screen_height_drawing`
  y resta márgenes vía `screen_to_chart_point`; `tests/crosshair.rs` (4 tests)
  pasa.
- T5: Exponer zoom independiente X/Y en `Viewport` (S4). Route: inline.
  Status: done. `Viewport` reemplaza `zoom_level` unificado por `zoom_x`/`zoom_y`
  independientes; `zoom_at(delta_x, delta_y, ...)` acepta delta por eje;
  `set_zoom`/`set_zoom_x`/`set_zoom_y` independientes; `auto_scale` ajusta solo
  el eje Y (PriceScale / precio) y deja el eje X (TimeScale / velas) intacto.
  `scale.rs` expone `TimeScale` (eje X) y `PriceScale` (eje Y) dentro de
  `ChartScales`, reemplazando el `AxisScale` genérico. Consumidores actualizados:
  `events/mod.rs` (scroll → eje Y), `services/mod.rs` (debug), `glow.rs` (zoom_x
  para velas, zoom_y para precio y crosshair), tests y benchmark.
- T6: Pruebas de propiedades adicionales para el engine (S6). Route: inline.
  Status: done. `chart_proptest.rs` (auto_scale, zoom, pan, round-trip,
  límites del viewport).
- T7: Verificación final: cargo check, clippy -D warnings, cargo test,
  cargo fmt, verify_architecture. Route: inline. Status: done.
  `cargo test --workspace`, `clippy -D warnings`, `cargo fmt --check` y
  `tests/architecture.rs` todos verdes.

## Log

L1: Continuación de la auditoría técnica de ApexTrader v0.1.0. Etapas 1 y 2
completadas en ramas previas. Inicio de la Etapa 3 (profesionalizar Chart
Engine) aprobada por el usuario.
L2: T2 (auto_scale) y T3 (zoom anclado al cursor) completados y commiteados
en 93cbe59. auto_scale valida rangos degenerados/no finitos con
validate_price_range antes de calcular; el zoom vertical se rige por
available_height/screen_height (price_range se cancela en price_to_screen,
no por price_range_value, que desbordaba los precios al estar el límite
horizontal más apretado). zoom_at mantiene el punto bajo el cursor fijo;
proptest prop_zoom_in_out_keeps_cursor_vela_fija valida que el índice de
vela bajo el cursor no se mueve tras zoom in/out (tolerancia sub-píxel por
punto flotante). cargo test --workspace, clippy -D warnings y cargo fmt
todos verdes.
L3: T4 (crosshair preciso con área útil), T5 (zoom independiente X/Y en
Viewport y TimeScale/PriceScale en scale.rs), T6 (pruebas de propiedades) y
T7 (verificación final) completados y commiteados en 93cbe59 (T2+T3) más el
commit de T5. Viewport reemplaza zoom_level unificado por zoom_x/zoom_y;
zoom_at(delta_x, delta_y, ...) por eje; auto_scale solo ajusta el eje Y
(PriceScale/precio) y deja el eje X (TimeScale/velas) intacto; scale.rs
expone TimeScale y PriceScale dentro de ChartScales. Consumidores actualizados:
events/mod.rs (scroll → eje Y), services/mod.rs (debug), glow.rs (zoom_x para
velas, zoom_y para precio y crosshair), tests y benchmark. cargo test --workspace
(66 tests), clippy -D warnings, cargo fmt --check y tests/architecture.rs todos
verdes. Etapa 3 lista para entrega bajo `delivery strategy: ask-on-risk`
(sin push ni PR hasta que el usuario lo decida).
