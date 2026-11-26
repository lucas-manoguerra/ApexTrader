# WGPU chart callback architecture

## Responsibility boundaries

| Component | Owns | Must not own |
| --- | --- | --- |
| eframe/winit integration | Window, events, surface lifecycle, presentation | Chart geometry or financial rules |
| egui-wgpu | Adapter, device, queue, shared render pass, callback scheduling | ApexTrader chart state |
| Chart scene | Immutable visible market snapshot and viewport inputs | GPU resources or UI primitives |
| Geometry builder | Deterministic CPU-side vertices from scene/layout | Device, queue, surface, market loading |
| WGPU chart backend | Pipelines, persistent buffers, upload policy, draw calls | Market identity, price calculations, window lifecycle |
| egui presentation | Axes, labels, panels, interaction and clipping | Creating a second GPU context |

## Callback contract

Use egui_wgpu::Callback::new_paint_callback with CallbackTrait.

- prepare is the only callback stage that may create/update chart GPU resources.
- paint only binds the prepared pipeline/buffers and issues draw commands.
- GPU resources live in eframe's RenderState.renderer.callback_resources type map, not in per-frame callback objects.
- Callback values own an immutable, bounded geometry snapshot. They do not borrow application/domain state.
- Callback rectangles use egui logical points. GPU viewport/scissor conversion must honor pixels_per_point.

## Geometry and upload policy

The neutral geometry layer emits triangle-list vertices for candle bodies, wicks and volume bars. Vertices use chart-local coordinates; the backend converts them to clip-space using the callback viewport. Geometry is grouped into as few buffers/draw calls as practical.

Persistent vertex buffers grow to the next required capacity and never shrink during ordinary frames. Buffer writes occur only when the immutable geometry snapshot changes. Cursor-only changes do not invalidate series geometry.

## Resource lifecycle

The application initializes chart pipeline resources from the wgpu_render_state supplied by eframe. eframe remains the sole owner of device and surface. If a device is recreated, chart resources are dropped/recreated through the host renderer's callback resource lifecycle. The chart backend never creates an Instance, Adapter, Device, Queue, Surface, or native window.

## Validation

- Unit tests validate geometry counts, finite coordinates, clipping bounds, empty/degenerate inputs and invalid dimensions.
- Invalidation tests prove cursor-only changes preserve series geometry.
- CI checks the WGPU feature graph on Windows, macOS and Linux.
- Hardware-backed runtime validation is a separate acceptance level; compilation alone does not prove presentation on a real driver.
