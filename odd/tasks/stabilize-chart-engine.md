# Estabilizar el Chart Engine — Etapa 1

Objetivo: corregir los hallazgos de alta prioridad de la auditoría (2.1 a 2.6) sin crear todavía los 20 crates de la arquitectura objetivo.
Branch: `feature/stabilize-chart-engine`
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (auditoría 2.1): `create_renderer()` selecciona el backend por capacidad efectiva (surface real), no solo por creación de `Device` wgpu. wgpu no paga costo de inicialización si no puede renderizar.
S2 (auditoría 2.2): `screen_height_for_pan()` recibe el área de dibujo real (`ChartArea`); se elimina el `720.0` hardcodeado.
S3 (auditoría 2.3): existe un módulo de transformaciones/geometría que es la única fuente de verdad; `Viewport`, `GlowChartRenderer`, `WgpuPipeline` y el benchmark usan la misma fórmula; candle spacing y candle width tienen relaciones definidas explícitamente.
S4 (auditoría 2.4): espacios de coordenadas `ScreenPoint`/`ChartPoint`/`DataPoint` definidos con conversiones explícitas; el crosshair compara coordenadas del mismo espacio.
S5 (auditoría 2.5): `VisibleRange { first, last }` definido; `GlowChartRenderer` itera solo velas visibles (`first..=last`); complejidad O(N_visible).
S6 (auditoría 2.6): `Price`/`Candle`/`Quantity` consistentes; `Quantity` con errores propios (`QuantityError`); `Price` documenta comportamiento con precios negativos (futuros).
S7 (auditoría 3.4): renderer no reconstruye `Price`/`PriceRange` por frame; conversiones f64→Decimal con validaciones explícitas y sin `expect()` ocultando fallos numéricos.
S8 (auditoría numéricos): `Viewport` valida rangos degenerados, división por cero, y `screen_x_to_data` comprueba límites del dataset.
S9 (benchmark): `benches/chart_transform_benchmark.rs` mide funciones reales de `Viewport` y declara `harness = false`.
S10 (testing): pruebas de propiedades con `proptest` (zoom, pan, round-trip, auto_scale, visible_range).

## Tasks

- T1: Corregir selección de renderer en `renderer/mod.rs` y `renderer/wgpu.rs` (S1, S9). Route: inline. Status: done. Commit: 32da148.
- T2: Eliminar altura fija 720.0 en `domain/viewport.rs`; pasar `ChartArea` real (S2, S8). Route: inline. Status: done. Commit: 2ac1923.
- T3: Crear módulo de transformaciones/geometría como única fuente de verdad (S3). Route: inline. Status: done. Commit: 14b031c.
- T4: Definir espacios de coordenadas y corregir crosshair en `main.rs` y `renderer/glow.rs` (S4). Route: inline. Status: done. Commit: 9c0dbe6.
- T5: Implementar `VisibleRange` y renderizado solo de velas visibles en `renderer/glow.rs` (S5). Route: inline. Status: done. Commit: 2ec226a.
- T6: Unificar `Price`/`Quantity`/`Candle`; `Quantity` con errores propios (S6). Route: inline. Status: done. Commit: 44c1812.
- T7: Eliminar reconstrucción de `Price`/`PriceRange` y `expect()` en camino de renderizado (S7). Route: inline. Status: done. Commit: 1a4a574.
- T8: Validaciones numéricas en `Viewport` (rangos degenerados, división por cero, límites dataset) (S8). Route: inline. Status: done. Commit: 1a4a574.
- T9: Corregir benchmark para medir funciones reales de `Viewport` con `harness = false` (S9). Route: inline. Status: done. Commit: 436f7fe.
- T10: Ampliación de tests con `proptest` (round-trip, pan, zoom, auto_scale, visible_range) (S1-S8). Route: inline. Status: done. Commit: 436f7fe.
- T11: Verificación final: cargo check, clippy -D warnings, cargo test, cargo fmt, verify_architecture (S1-S9). Route: inline. Status: done. Commit: 436f7fe.

## Referencias oficiales de dependencias

Referencias de API para el backend experimental, la precisión numérica y
las pruebas de propiedades:

- wgpu: https://docs.rs/wgpu/30/wgpu/
- raw-window-handle: https://docs.rs/raw-window-handle/0.6/raw_window_handle/
- rust_decimal: https://docs.rs/rust_decimal/1.43/rust_decimal/
- proptest: https://docs.rs/proptest/1/proptest/

## Log

L1: Auditoría técnica de ApexTrader v0.1.0. Revisión estática de código fuente, manifiesto Cargo, módulos de dominio, renderizadores, pruebas, documentación de arquitectura, scripts y CI sobre la rama `main`. Plan de refactorización aprobado por el usuario para la Etapa 1 (estabilizar Chart Engine). Inicio de implementación.
L2: Rama `feature/stabilize-chart-engine` creada desde `main`. Documento de seguimiento creado en `odd/tasks/stabilize-chart-engine.md`.
L3: T1 completado y commiteado (32da148). `create_renderer()` ya no inicializa wgpu sin surface real; Glow se usa por capacidad efectiva.
L4: T2 completado y commiteado (2ac1923). `Viewport::pan()` recibe `screen_height` real; eliminada la altura fija 720.0.
L5: T3 completado y commiteado (14b031c). Módulo `domain/transforms.rs` como única fuente de verdad para coordenadas.
L6: T4 completado y commiteado (9c0dbe6). Espacios de coordenadas ScreenPoint/ChartPoint/DataPoint; crosshair usa ScreenPoint consistente.
L7: T5 completado y commiteado (2ec226a). `VisibleRange` y renderizado solo de velas visibles (O(N_visible)).
L8: T6 completado y commiteado (44c1812). `QuantityError` propio; `PriceError` limpio de variantes Quantity.
L9: T7+T8 completados y commiteados (1a4a574). Validaciones numéricas en `Viewport`; crosshair sin `expect()` por frame.
L10: T9+T10 completados y commiteados (436f7fe). Benchmark real con `harness = false`; pruebas de propiedades con `proptest`.
L11: Verificación final completada: cargo check OK, clippy -D warnings OK, cargo test OK (6 suites + proptest verdes), cargo fmt OK.
