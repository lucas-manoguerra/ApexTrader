# Migrar chart renderer a wgpu

**Objective:** Crear un trait `ChartRenderer` que abstraiga el renderizado del chart, permitiendo cambiar entre glow (egui Painter) y wgpu sin tocar el dominio ni la app.

**Branch:** `feature/migrar-a-wgpu`

**Delivery strategy:** ask-on-risk (default)

**Test runner:** `cargo check` and `cargo run`

## Specs

**S1** — Definir trait `ChartRenderer` en `src/renderer/mod.rs` con método `draw(&self, candles: &[Candle], rect: Rect)`.
- Acceptance criteria: `cargo check` compila; trait existe y es usado por `main.rs`.

**S2** — Mover código actual de `draw_chart` a `GlowChartRenderer` en `src/renderer/glow.rs`.
- Acceptance criteria: `cargo check` compila; renderer glow funciona igual que antes.

**S3** — Crear `WgpuChartRenderer` en `src/renderer/wgpu.rs` que intente inicializar wgpu.
- Acceptance criteria: `cargo check` compila; si wgpu falla al iniciar, la app usa glow como fallback sin panicar.

## Tasks

| ID | Spec | Route | Status | Commit |
|----|------|-------|--------|--------|
| T1 | S1 | inline | done | 97a06da |
| T2 | S2 | inline | done | 97a06da |
| T3 | S3 | inline | done | 97a06da |

## Librerías consultadas

| Crate | Versión | docs.rs | APIs usadas |
|-------|---------|---------|-------------|
| wgpu | 30.0.2 | https://docs.rs/wgpu/30.0.2/ | `wgpu::Instance`, `wgpu::InstanceDescriptor`, `wgpu::RequestAdapterOptions`, `wgpu::RequestAdapterOptions`, `wgpu::DeviceDescriptor`, `wgpu::Surface`, `wgpu::SurfaceConfiguration`, `wgpu::RenderPipeline`, `wgpu::Buffer`, `wgpu::util::DeviceExt` |
| bytemuck | 1.25.2 | https://docs.rs/bytemuck/1.25.2/ | `bytemuck::cast_slice`, `bytemuck::Pod`, `bytemuck::Zeroable` |
| pollster | 1.0.3 | https://docs.rs/pollster/1.0.3/ | `pollster::block_on` |
| eframe | 0.37 | https://docs.rs/eframe/0.37/ | `eframe::run_native`, `eframe::NativeOptions`, `eframe::App` |
| egui | 0.37 | https://docs.rs/egui/0.37/ | `egui::Painter`, `egui::Rect`, `egui::Color32`, `egui::Stroke` |

## Log

**L2**: wgpu/DirectX 12 previously crashed with STATUS_ACCESS_VIOLATION on Intel UHD 630. Strategy: trait-based renderer with glow fallback.

**L3**: Implemented `ChartRenderer` trait in `src/renderer/mod.rs`. `GlowChartRenderer` in `src/renderer/glow.rs`. `WgpuChartRenderer` in `src/renderer/wgpu.rs`. `create_renderer()` factory tries wgpu first, falls back to glow.

**L4**: `cargo check` pasa limpio. `cargo run` confirma que wgpu se inicializa correctamente en Intel UHD 630 (DX12): adapter encontrado, device y queue creados. "Usando renderer: wgpu".

**L5**: `create_renderer()` modificado para verificar `has_surface()` antes de seleccionar wgpu. Como no hay window handle de winit disponible, wgpu no tiene surface y `create_renderer()` cae a glow como fallback. Esto evita los errores de "wgpu surface no inicializada" en cada frame.

**L6**: `WgpuChartRenderer::has_surface()` agregado para consultar si wgpu puede dibujar.

**L7**: `WgpuPipeline` creado con shaders WGSL, pipeline de renderizado, vertex/index buffers. Placeholder listo para cuando se conecte la surface.

Próximo paso: conectar wgpu con winit surface para dibujo real en GPU, o seguir con zoom/pan.
