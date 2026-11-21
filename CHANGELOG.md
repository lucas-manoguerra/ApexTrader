# Changelog

Todos los cambios notables de ApexTrader se documentan aquí.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto sigue [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### Agregado

- Estructura inicial del proyecto con `Cargo.toml` y dependencias base (egui, eframe, wgpu, winit, thiserror, tracing, bytemuck, pollster).
- Aplicación de escritorio con egui y renderer de velas (wgpu con respaldo glow).
- `ChartState` con manejo de viewport (zoom y pan).
- `Candle` como tipo de dominio para datos de mercado.
- Logging estructurado con `tracing`.

### Cambiado

- Ninguno todavía.

### Deprecado

- Ninguno todavía.

### Eliminado

- Ninguno todavía.

### Corregido

- Ninguno todavía.

### Seguridad

- Ninguno todavía.

## [0.1.0] - 2026-11-20

### Agregado

- Primera versión experimental. Chart de velas con zoom (scroll) y pan (arrastre).