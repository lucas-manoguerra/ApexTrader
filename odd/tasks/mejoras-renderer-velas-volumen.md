# Mejoras al renderer: velas con cuerpo real, wicks y volumen — Etapa 6

Objetivo: mejorar el renderer glow para mostrar velas con cuerpo real (open/close),
méchas (high/low) y agregar barras de volumen debajo del chart principal con su
propio eje Y.

Branch: `feature/mejoras-renderer-velas-volumen`
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (velas con cuerpo real): el cuerpo de cada vela se dibuja como rectángulo entre
open y close, con color según dirección (verde si subió, rojo si bajó).

S2 (wicks/méchas): línea vertical entre high y low para cada vela, centrada en
el cuerpo.

S3 (volumen): barras de volumen debajo del chart principal, una por vela,
escaladas al volumen máximo visible.

S4 (eje de volumen): ticks dinámicos en el eje Y del panel de volumen para
leer el valor de cada barra.

S5 (layout): separación visual entre el chart de precios y el panel de volumen
(margen intermedio).

## Tasks

- T1: Revisar dibujo actual de velas en glow.rs (cuerpo y wick) y verificar que
  usen open/close/high/low correctamente. Route: inline. Status: done.
  Cuerpo (open→close como rectángulo) y wick (high→low como línea) ya están
  implementados en glow.rs.
- T2: Agregar panel de volumen debajo del chart principal con barras por vela.
  Route: inline. Status: done. Barras de volumen dibujadas bajo cada vela,
  coloreadas igual que el cuerpo, altura proporcional al volumen.
- T3: Agregar eje de volumen con ticks dinámicos. Route: inline. Status: done.
  `draw_volume_axis` dibuja ticks con formato K/M para volumen grande.
- T4: Ajustar layout para separar chart de precios y panel de volumen (S5).
  Route: inline. Status: done. `bottom_precios` separa el área de precios del
  panel de volumen; crosshair se limita al área de precios.
- T5: Pruebas unitarias para el renderer de volumen y cálculo de escalas.
  Route: inline. Status: pending.
- T6: Verificación final: cargo test --workspace, clippy -D warnings, cargo fmt.
  Route: inline. Status: pending.

## Log

L1: Etapa 6 aprobada tras completar la Etapa 5 (rama
feature/integrar-datos-mercado-ui mergeada vía PR #4).
L2: T1-T4 completados y commiteados. `Candle` ahora tiene campo `volume`.
  `CandleError::InvalidVolume` para volumen negativo/no finito.
  `glow.rs`: dibuja barras de volumen bajo cada vela, `draw_volume_axis` con
  ticks dinámicos (formato K/M), layout separado: área de precios arriba y
  panel de volumen abajo. Crosshair limitado al área de precios. `auto_scale`
  calcula `max_volume` para escalar las barras.
