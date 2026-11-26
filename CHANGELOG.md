# Changelog

Todos los cambios notables de ApexTrader se documentan aquí.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto sigue [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### Cambiado

- Se reorganizó la documentación de ingeniería y se establecieron contratos para agentes, contribuciones y decisiones arquitectónicas.
- Se amplió el README con arquitectura, requisitos, validación y enlaces a la documentación.
- Se incorporó un control de CI para impedir comentarios en los archivos de código modificados.
- Se eliminaron comentarios y Rustdoc del código fuente, pruebas, benchmarks, scripts y configuración ejecutable mantenidos por el proyecto.


### Agregado

- Estructura inicial del proyecto con `Cargo.toml` y dependencias base (egui, eframe, wgpu, winit, thiserror, tracing, bytemuck, pollster).
- Aplicación de escritorio con egui y renderer de velas (wgpu con respaldo glow).
- `ChartState` con manejo de viewport (zoom y pan).
- `Candle` como tipo de dominio para datos de mercado.
- Logging estructurado con `tracing`.
- `domain::chart` con `ChartDataPoint` (vela OHLCV + `TimePoint` + índice lógico) y `TimePoint` (open/close time + intervalo).
- `MarketCandle::to_chart_data_point` preserva timestamp e intervalo de la vela de mercado.
- `ChartState` posee y presta `Vec<ChartDataPoint>` en lugar de `Vec<Candle>`.
- `LoadCandles` devuelve puntos de datos válidos y `Vec<IngestionRejection>` con motivo explícito por rechazo (sin descartar silenciosamente).
- `chart::layout` con `ChartLayout`: única fuente de verdad para rectángulos de plot, ejes y paneles (usado por input y renderer).
- `chart::crosshair` con `Crosshair`, `CrosshairUpdateParams` y modos `Full`/`LabelsOnly`/`Hidden`.
- `chart::hit_testing` con `HitTestable`, `HitTester`, `HitTestContext` y `HitResult`.
- `chart::interaction` con `Controller` (estado del gesto con anclas al inicio, sin drift) y `Events`.
- `VisibleRange` consolidado como única fuente de verdad en `domain::transforms` con `empty()`/`new()`/`Default`.
- Corrección del offset vertical en `Viewport::zoom_at` y `Viewport::pan` (consistente con `price_to_screen` / `screen_y_to_price`): zoom anclado al cursor conserva el precio bajo el cursor.
- `Candle` getores toman `&self` en lugar de consumir `self`.
- Tests: `zoom_at_y_preserves_cursor_price` y `pan_y_is_consistent_with_price_transforms` (AT-001, AT-004).
- Capturas de referencia de pantalla en `assets/screen_reference/`.

### Cambiado

- `chart::visible_range` ahora re-exporta `VisibleRange` desde `domain::transforms` (elimina duplicación).
- `domain::candle` accesores retornan `&self` en lugar de consumir.
- `src/domain/mod.rs` declara `pub mod chart`.
- Renderer (`glow.rs`) usa `ChartLayout`, `ChartDataPoint` y `O(N_visible)` por frame.
- Trait `ChartRenderer::draw` recibe `&[ChartDataPoint]` en lugar de `&[Candle]`.
- Aplicación (`services/mod.rs`, `events/mod.rs`) usa `ChartDataPoint` y `ChartLayout`.
- Tests de arquitectura, coordenadas y mercado actualizados a `ChartDataPoint`.

### Deprecado

- `ChartState::add_candle` / `add_candles` / `candles()`: reemplazados por `add_data_point` / `add_data_points` / `data_points()`.
- `MarketCandle::to_domain_candle` sin contexto temporal: reemplazado por `to_chart_data_point`.

### Eliminado

- Definición duplicada de `VisibleRange` en `src/chart/visible_range.rs` (ahora re-exporta desde `domain::transforms`).

### Corregido

- Offset vertical en `Viewport::zoom_at`: usaba `screen_height / zoom_y`, ahora `screen_height * zoom_y` (consistente con la transformación de precio a pantalla).
- Offset vertical en `Viewport::pan`: mismo criterio de consistencia con `price_to_screen` / `screen_y_to_price`.
- `visible_range()` cuando `first > last`: retorna `VisibleRange::empty()` en lugar de `{0, 0}` (que representaba la vela cero).
- `visible_range()` cuando no hay velas: retorna `VisibleRange::empty()`.

### Seguridad

- Ninguna todavía.

## [0.1.0] - 2026-11-20

### Agregado

- Primera versión experimental. Chart de velas con zoom (scroll) y pan (arrastre).