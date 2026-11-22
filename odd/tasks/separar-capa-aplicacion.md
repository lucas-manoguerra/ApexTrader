# Separar la capa de aplicación — Etapa 2

Objetivo: extraer la lógica de interacción y la composición de dependencias de
`main.rs` hacia una capa `application` diferenciada, de modo que `main.rs`
quede como punto de entrada mínimo y el dominio no conozca cómo se ejecutan las
acciones del usuario.
Branch: `feature/separar-capa-aplicacion`
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (auditoría 3.1): `main.rs` termina siendo muy pequeño:
`fn main() -> eframe::Result { apex_trader::application::run() }`.
S2 (auditoría 3.1): La configuración y la composición de dependencias viven en
la capa de aplicación, no en el punto de entrada.
S3 (auditoría 3.2): Existe una capa `application` diferenciada con
`commands/`, `events/` y `services/`.
S4 (auditoría 3.2): El flujo de zoom sigue: Input → Chart Controller → Chart
Engine → Viewport / Chart Domain → Nuevo estado del gráfico → Renderer.
S5 (auditoría 3.2): El renderer no decide cómo funciona el zoom; representa el
estado que recibe.
S6 (auditoría 3.1): `main.rs` no contiene inicialización de logging,
configuración de fuentes, creación de datos de prueba, gestión de eventos del
mouse, cálculo de zoom/pan, gestión del crosshair, coordinación del renderer ni
presentación de información de depuración.

## Tasks

- T1: Crear módulo `application` con `mod.rs`, `commands/`, `events/`,
  `services/` y registrar en `lib.rs`. Route: inline. Status: pending. Commit: -
- T2: Mover lógica de input (zoom con scroll, pan con arrastre, crosshair) a
  `application/events/`. Route: inline. Status: done. Commit: pending.
- T3: Mover composición de dependencias (ChartState, renderer, datos de prueba
  de ejemplo, fuentes) a `application/services/`. Route: inline. Status:
  pending. Commit: -
- T4: Crear `application::run()` que `main()` llame, encapsulando `eframe::run_native`.
  Route: inline. Status: pending. Commit: -
- T5: Reducir `main.rs` a solo `fn main() -> eframe::Result { apex_trader::application::run() }`.
  Route: inline. Status: pending. Commit: -
- T6: Verificación final: cargo check, clippy -D warnings, cargo test, cargo
  fmt, verify_architecture. Route: inline. Status: pending. Commit: -

## Log

L1: Continuación de la auditoría técnica de ApexTrader v0.1.0. Etapa 1
(estabilizar Chart Engine) completada y commiteada en
`feature/stabilize-chart-engine`. Inicio de la Etapa 2 (separar capa de
aplicación) aprobada por el usuario.
