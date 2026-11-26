# Dependency graph and boundary audit

## Scope and method

This audit inspects all 51 Rust source files under `src/` at the governance branch head used for this review. It records explicit `use crate::...` dependencies and module declarations, then evaluates them against the intended dependency direction. Test-only imports are distinguished from production imports where visible. This is a source-level graph, not a compiler-generated dependency graph; macro expansion, type-level coupling and indirect dependencies are outside its scope.

The graph is an architectural snapshot. It is not yet an enforcement specification and does not justify moving files by itself.

## Current dependency graph

| Area | Observed dependencies | Assessment |
|---|---|---|
| `main` | `application::run`, `eframe` | Entry point delegates into an application module that owns desktop startup |
| `application::commands::load_candles` | `domain`, `market::provider`, `market::errors`, `market::symbol` | Use case depends on a provider contract owned by the market module rather than an application port |
| `application::events` | `chart::interaction`, `chart::layout`, `domain`, `eframe::egui` | UI input translation lives in application and imports the presentation framework |
| `application::services` | `application`, `chart`, `domain`, `logging`, `market`, `renderer`, `eframe`, `rust_decimal`, `std` | Composition, UI, market-provider construction, simulation, chart policy, timing and rendering are concentrated in one module |
| `domain` | domain types, `errors`, `rust_decimal` | Mostly framework-independent, but shares a dependency cycle with errors |
| `chart` | chart interaction/layout/hit testing and domain types | Chart policy is separated from the desktop framework except where application adapters call it |
| `market` | market model/provider/symbol/errors and domain chart types | Mixes provider contracts and mock implementations; market candle conversion knows chart-specific domain structures |
| `renderer` | domain, chart layout, errors, `eframe::egui`, `wgpu`, `bytemuck`, tracing | Rendering contract leaks egui; renderer intermediate representation depends on chart layout |
| `errors` | error types; `errors::chart` imports `domain::candle::Candle` | Error ownership creates a cycle with domain |

## Verified boundary violations and risks

### DG-001 — Domain and errors depend on each other

Evidence:
- `src/domain/candle.rs` imports `crate::errors::candle::{CandleDump, CandleError}`.
- `src/domain/price.rs`, `quantity.rs`, `price_range.rs` import error types from `crate::errors`.
- `src/errors/chart.rs` imports and embeds `crate::domain::candle::Candle`.

The crate compiles, but the conceptual dependency direction is cyclic: domain types depend on the error module while a chart error depends on a domain type. The `errors` namespace is not a lower-level independent layer.

**Direction:** place errors next to the abstraction that owns their invariants, or define a lower-level error contract that does not import domain types. Avoid a broad error crate that becomes a shared dependency hub. Preserve error messages and matching behavior with focused tests.

### DG-002 — Application use case depends on a market-owned provider contract

Evidence:
- `src/application/commands/load_candles.rs` imports `MarketDataProvider`, `MarketError` and `Symbol` from `crate::market`.
- `src/market/provider.rs` owns `MarketDataProvider`.

The use case is coupled to a module that also contains concrete market models and mock implementations. This reverses the intended ownership of the port: the application should define the capability it requires; infrastructure/market adapters should implement it.

**Direction:** define a narrow application port around the use case's actual needs, then adapt existing providers. Do not duplicate market domain types or introduce generic abstractions without a second real implementation.

### DG-003 — Framework input adapter is placed inside application

Evidence:
- `src/application/events/mod.rs` imports `eframe::egui` and accepts `egui::Ui`, `egui::Response` and `egui::Rect`.
- The same module owns conversion from pointer/scroll input to chart interaction events.
- `src/application/mod.rs` exposes `run() -> eframe::Result`.

This makes the application boundary require the desktop UI framework and prevents a framework-neutral application entry point.

**Direction:** move only the egui input translation and desktop bootstrap into presentation. Keep interaction events and controller policies framework-neutral. Retain behavior with interaction regression tests.

### DG-004 — Renderer contract is coupled to egui and chart layout

Evidence:
- `src/renderer/mod.rs` defines `ChartRenderer::draw` using `&egui::Painter` and `egui::Rect`.
- `src/renderer/ir.rs` imports `ChartLayout` and `Rect` from `crate::chart::layout`.
- `src/renderer/glow.rs` implements the egui-backed renderer.
- `src/renderer/wgpu.rs` also implements the same egui-shaped trait.

The interface is not a backend-neutral rendering contract. The scene representation also knows the chart layout implementation.

**Direction:** define a renderer-owned, framework-neutral drawing surface and geometry contract only if a second backend is genuinely supported. Keep egui conversion in the Glow/presentation adapter. Separate chart layout from render scene only after the ownership and coordinate invariants are explicit.

### DG-005 — Application services are the composition root and the application itself

Evidence:
- `src/application/services/mod.rs` imports UI framework, chart state/layout, renderer, market provider traits and concrete mock providers, logging, financial conversion, and system clocks.
- The same file implements the desktop app, starts the eframe event loop, loads history, creates realtime simulation, computes price tick size from symbol text, manages viewport/autoscale and invalidation, and renders the chart.

This is the strongest cohesion failure in the current graph. It makes unrelated changes converge on one file and lets production startup instantiate simulation providers directly.

**Direction:** first isolate the desktop composition root; then extract presentation state/rendering, application use cases and provider adapters along existing behavior seams. Remove synthetic fallback from any production market-data path before connecting a real provider. Keep simulation opt-in and visibly identified.

### DG-006 — Market candle conversion knows chart representation

Evidence:
- `src/market/candle.rs` imports `domain::candle::Candle`, `domain::chart::chart_data::ChartDataPoint` and `domain::chart::time_point::TimePoint`.
- `LoadCandles` receives market candles and immediately returns chart data points.

The market model is coupled to a specific chart projection. That makes alternate consumers (storage, analytics, execution, reconciliation) depend on chart data structures.

**Direction:** keep normalized market data independent of chart projection. Convert at an application/presentation boundary using an explicit mapper, preserving source identity, timestamps, precision and rejection metadata.

### DG-007 — Mock implementations are public members of the market module

Evidence:
- `src/market/mod.rs` publicly exposes `mock` and `mock_realtime`.
- `application/services` constructs both concrete mock providers during application startup and realtime operation.

Simulation is not isolated from application composition and its availability is not an explicit mode.

**Direction:** isolate simulation adapters and make their activation explicit. Do not allow mock fallback to masquerade as a market-data recovery strategy.

### DG-008 — Public module surface is broader than an explicit API

Evidence:
- `src/lib.rs` declares every major module as `pub mod`.
- Most module roots re-export their internal modules publicly, including mocks and renderer backends.
- `src/lib.rs` globally suppresses `dead_code`.

This exposes implementation details as de facto public API and weakens compiler feedback about unused code.

**Direction:** define the supported library surface deliberately, reduce internal modules to `pub(crate)` or private where consumers permit, and remove the global dead-code allowance only after classifying resulting diagnostics. Avoid a bulk visibility change without consumer and integration-test analysis.

## Dependency direction target

The intended direction for the next stages is:

```text
main / desktop bootstrap
        |
        v
presentation adapters -----> chart interaction and layout
        |
        v
application use cases -----> application-owned ports
        |                           ^
        v                           |
domain models <------------- market / storage / clock adapters
        ^
        |
chart projection

rendering consumes framework-neutral scene data;
a concrete presentation/backend adapter owns egui or wgpu integration.
```

This is a conceptual direction, not a request to create every listed folder immediately. Keep one crate until the boundaries are stable and independently valuable.

## Prioritized remediation sequence

1. Remove the synthetic fallback from production startup and make simulation an explicit mode, with tests proving provider failures cannot produce synthetic market data.
2. Extract desktop bootstrap and egui input translation from `application`, without changing interaction semantics.
3. Move provider contracts to application-owned ports and isolate mock adapters.
4. Separate normalized market data from chart projections; define series identity and timeframe before changing candle types.
5. Resolve the domain/errors dependency cycle without losing useful typed errors.
6. Define a backend-neutral renderer contract only when its geometry and coordinate semantics are specified.
7. Narrow public module visibility and remove global `dead_code` suppression after reviewing diagnostics.
8. Add graph-based architecture enforcement and tests before enforcing a broader layer matrix.

## Constraints for all remediation

- Preserve zero comments in code; explanatory rationale belongs in documentation.
- No financial calculations or OHLCV representation changes as part of structural-only commits.
- No bulk file moves or crate extraction for cosmetic reasons.
- Each behavior-preserving refactor must include tests for its public invariants.
- CI must pass on the final PR head; no merge until reviewed.
