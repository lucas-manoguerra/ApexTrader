# Ownership y separación de responsabilidades

**Objective:** Refactorizar el chart para que quede claro quién posee los datos (ownership) y quién los presta (borrowing), con separación de responsabilidades entre dominio, renderer y app.

**Branch:** `feature/ownership-separacion-responsabilidades`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo check` and `cargo run`

## Specs

**S1** — `Candle` y `ChartState` viven en un módulo de dominio (`src/domain/`). `ChartState` posee `Vec<Candle>`.
- Acceptance criteria: `cargo check` compila; `ChartState` tiene `candles: Vec<Candle>`.

**S2** — `draw_chart` vive en un módulo de renderer (`src/renderer/`). Recibe `&[Candle]` (referencia inmutable, borrowing) y solo dibuja.
- Acceptance criteria: `cargo check` compila; `draw_chart` no toma ownership de ningún `Candle`.

**S3** — `ApexTraderApp` en `src/main.rs` solo coordina: crea `ChartState`, llama al renderer pasando prestados los datos.
- Acceptance criteria: `cargo check` compila; app renderiza igual que antes.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S1 | inline | done | |
| T2 | S2 | inline | done | |
| T3 | S3 | inline | done | |

## Log

**L1** (verbatim): User wants to continue with ownership/borrowing and build separation of responsibilities from the start.

**L2**: Current state: `src/main.rs` has `Candle`, `ChartState`, `draw_chart`, and `ApexTraderApp` all in one file. Backend is `glow` (OpenGL) because `wgpu`/DirectX 12 crashes on Intel UHD 630. Fonts fixed with `configure_fonts()`.

**L3**: Refactored into three modules:
- `src/domain/candle.rs` — `Candle` (tipo de dominio, puro dato).
- `src/domain/chart_state.rs` — `ChartState` **posee** `Vec<Candle>` (ownership). Expone `&[Candle]` vía `candles()`.
- `src/renderer/mod.rs` — `draw_chart` recibe `&[Candle]` (borrowing, solo lectura).
- `src/main.rs` — `ApexTraderApp` coordina: crea `ChartState`, pide prestado al renderer.

**L4**: `cargo check` pasa limpio (sin warnings). `cargo run` abre la ventana correctamente con glow.
