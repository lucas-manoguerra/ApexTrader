# Zoom y Pan en el chart

**Objective:** Agregar interactividad al chart: zoom (scroll) y pan (arrastrar) sobre las velas, como en TradingView.

**Branch:** `feature/zoom-pan`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo check` and `cargo run`

## Specs

**S1** — `Viewport` en `domain/` posee el estado de zoom y pan (ownership).
- Acceptance criteria: `Viewport` tiene `zoom_level: f64`, `offset_x: f64`, `offset_y: f64`. Métodos para hacer zoom y pan.

**S2** — `ChartState` contiene un `Viewport`.
- Acceptance criteria: `ChartState` tiene un campo `viewport`. El renderer lee `&Viewport` para dibujar.

**S3** — El renderer aplica zoom y pan al dibujar las velas.
- Acceptance criteria: las velas se dibujan escaladas según `zoom_level` y desplazadas según `offset_x`/`offset_y`.

**S4** — Input de usuario: scroll para zoom, arrastre para pan.
- Acceptance criteria: `cargo run` abre ventana; scroll hace zoom; arrastrar mueve el chart.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S1 | inline | done | 97a06da |
| T2 | S2 | inline | done | 97a06da |
| T3 | S3 | inline | done | 97a06da |
| T4 | S4 | inline | done | 97a06da |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Ui`, `egui::Painter`, `egui::Rect`, `egui::Sense::click_and_drag`, `egui::Color32`, `egui::FontId`, `egui::Align2`, `egui::Stroke`, `egui::pos2`, `egui::vec2` |
| eframe | 0.37 | https://docs.rs/eframe/0.37/ | `eframe::run_native`, `eframe::NativeOptions`, `eframe::App`, `eframe::Result`, `eframe::Frame` |

## Log

**L2**: Created `Viewport` in `src/domain/viewport.rs` — posee `zoom_level`, `offset_x`, `offset_y`. Métodos: `zoom_at()`, `pan()`, `reset()`, `data_x_to_screen()`, `price_to_screen()`.

**L3**: `ChartState` ahora contiene un `Viewport`. Expone `viewport()` (inmutable) y `viewport_mut()` (mutable para input).

**L4**: Trait `ChartRenderer` actualizado para recibir `&Viewport` en `draw()`.

**L5**: `GlowChartRenderer` aplica zoom y pan al dibujar: `data_x_to_screen()` para X, `price_to_y` con `zoom_level` y `offset_y` para Y. Ejes de precio y tiempo ajustados al viewport.

**L6**: `main.rs` maneja input de egui: `smooth_scroll_delta` para zoom (centrado en posición del mouse), `drag_delta` para pan. Separación de borrows respetada: input mutable primero, luego viewport inmutable para renderizar.

**L7**: `cargo check` pasa limpio (solo warnings de código futuro). `cargo run` ejecuta sin errores. wgpu se inicializa en Intel UHD 630 (DX12) pero cae a glow como fallback (no hay surface de winit conectada).

**Separación de responsabilidades mantenida:**
- `domain/` POSEE: `ChartState` (velas + viewport)
- `renderer/` PRESTA: `draw_chart()` recibe `&[Candle]` y `&Viewport`
- `main.rs` COORDINA: input de usuario + llama al renderer

Próximo paso: conectar wgpu surface para dibujo real en GPU, o agregar más funcionalidades (más velas, indicadores).
