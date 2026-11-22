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
  como motor de transformaciones independiente. Route: inline. Status: pending.
- T2: Mejorar `auto_scale` con validación de rangos degenerados y centrado
  bidireccional (S1). Route: inline. Status: pending.
- T3: Mejorar `zoom_at` para anclar el zoom al cursor (S2). Route: inline.
  Status: pending.
- T4: Mejorar `screen_y_to_price` / `screen_x_to_data` para crosshair preciso
  usando el área de dibujo útil (S3). Route: inline. Status: pending.
- T5: Separar escalas X e Y independientes en `scale.rs` (S4). Route: inline.
  Status: pending.
- T6: Pruebas de propiedades adicionales para el engine (S6). Route: inline.
  Status: pending.
- T7: Verificación final: cargo check, clippy -D warnings, cargo test, cargo
  fmt, verify_architecture. Route: inline. Status: pending.

## Log

L1: Continuación de la auditoría técnica de ApexTrader v0.1.0. Etapas 1 y 2
completadas en ramas previas. Inicio de la Etapa 3 (profesionalizar Chart
Engine) aprobada por el usuario.
