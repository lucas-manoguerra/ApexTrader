# Chart: primera vela japonesa

**Objective:** Open a desktop window with egui and render one Japanese candlestick (OHLC) to prove the chart pipeline works end to end.

**Branch:** `feature/chart-primera-vela` (to be created from default branch)

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo test` and `cargo run`

## Specs

**S1** — The application opens a native desktop window titled "ApexTrader" using eframe + egui.
- Acceptance criteria: `cargo run` launches a visible window; closing it exits cleanly. ✅ Verificado con glow.

**S2** — The window displays multiple Japanese candlesticks (OHLC) with price and time axes.
- Acceptance criteria: visible series of vertical wicks and filled bodies appear; green when close >= open, red otherwise. ✅ Verificado.
- Body color: green when close >= open, red otherwise.

**S3** — The candlestick data is owned by `ChartState` and borrowed by the renderer, not moved.
- Acceptance criteria: code compiles with `cargo check`; renderer receives `&[Candle]` references. ✅ Verificado.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S1 | inline | done | 97a06da |
| T2 | S2 | inline | done | 97a06da |
| T3 | S3 | inline | done | 97a06da |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Ui`, `egui::Painter`, `egui::Rect`, `egui::Sense::click_and_drag`, `egui::Color32`, `egui::FontId`, `egui::Align2`, `egui::Stroke`, `egui::pos2`, `egui::vec2` |
| eframe | 0.37 | https://docs.rs/eframe/0.37/ | `eframe::run_native`, `eframe::NativeOptions`, `eframe::App`, `eframe::Result`, `eframe::Frame` |

## Log

**L2**: Project `ApexTrader` exists at `D:\Code\ApexTrader` with empty `Cargo.toml` and `src/main.rs`. Learning work was done in `D:\Code\rust-check` (PriceRange with Result, match, enum errors, 4 passing tests).

**L3**: Decision: accelerate to visual output. Build egui + eframe window with multiple hardcoded candles and axes.

**L4**: Backend decision: tried `wgpu` (DirectX 12 on Windows) — crashes with `STATUS_ACCESS_VIOLATION` on Intel UHD 630. Switched to `glow` (OpenGL) — works. Will revisit wgpu when GPU/drivers allow.

**L5**: Font fix: characters appeared as boxes (tofu). Added `configure_fonts()` that loads a system Windows font (Segoe UI / Arial / Tahoma) and prepends it to egui's `FontFamily::Proportional`, keeping egui's default fonts as fallback.
