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
- GPU resources live in eframe's RenderState.renderer.callback_resources type map, not in per-frame callback objects. A stable callback identity selects an independent resource set so multiple chart instances cannot overwrite one another's vertex buffers. Each renderer owns a lifecycle token; callback resources keep only a weak reference to it, and prepare prunes entries whose renderer has been dropped. This bounds retained callback resources to currently live chart renderers (plus entries awaiting the next callback prepare pass).
- Callback values own an immutable, bounded geometry snapshot. They do not borrow application/domain state.
- Callback rectangles use egui logical points. The callback viewport maps clip-space vertices to the logical chart rectangle; egui's scissor limits drawing to the plot and volume panels.

## Geometry and upload policy

The neutral geometry layer emits triangle-list vertices for candle bodies, wicks and volume bars. Vertices use chart-local coordinates; the backend converts them to clip-space using the callback viewport. Geometry is grouped into as few buffers/draw calls as practical.

Persistent vertex buffers grow to the next required capacity and never shrink during ordinary frames. Buffer writes occur only when the immutable geometry snapshot changes. Cursor-only changes do not invalidate series geometry.

## Resource lifecycle

The callback lazily creates chart pipeline resources during its first prepare call, using the device and target format captured from eframe's wgpu_render_state. If the target texture format changes, the chart pipeline and buffer resource set is recreated for that format. Failed vertex-count or buffer-size validation clears the drawable vertex count so stale geometry cannot be submitted. Buffers and the last immutable geometry snapshot live in CallbackResources. eframe remains the sole owner of device and surface. If a device/renderer is recreated, its callback resource map is recreated and the next prepare call rebuilds chart resources. The chart backend never creates an Instance, Adapter, Device, Queue, Surface, or native window.

## Validation

- Unit tests validate geometry counts, finite coordinates, clip-space conversion, empty/degenerate inputs and invalid viewport dimensions.
- Invalidation tests prove cursor-only changes preserve series geometry and empty scenes invalidate the cached series.
- The chart painter applies an egui clip rectangle spanning the plot and volume panel to the WGPU callback shape. The callback relies on egui-wgpu to translate that clip rectangle into the render-pass scissor; the CPU conversion tests do not prove the final scissor behavior.
- CI checks compilation and tests on Windows, macOS and Linux.
- Hardware-backed runtime validation, including visual clipping and DPI behavior, is a separate acceptance level; compilation alone does not prove presentation on a real driver.

## Current implementation

- The visible candle series is emitted as one egui-wgpu paint callback when eframe provides a WGPU target format.
- The callback draws candle wicks, bodies and volume bars from the neutral triangle-list geometry.
- Vertex positions are converted from chart-local logical points to clip-space using the chart rectangle dimensions.
- The pipeline uses a WGSL vertex/fragment shader and the eframe target format.
- Each chart callback has an independent persistent vertex buffer and pipeline resource set keyed by stable callback identity. Buffers grow only when required capacity exceeds current capacity, and are written only when the immutable geometry snapshot changes.
- The callback's prepare stage creates resources lazily and uploads changed vertices. The paint stage only binds the pipeline/buffer and issues one draw call.
- Axes, labels, panel surfaces and crosshair overlays remain egui presentation responsibilities.
- A Painter series path remains for test/headless construction where no WGPU target format is supplied; this path does not select Glow.
- CI compilation and unit tests do not yet prove runtime behavior on physical GPU drivers.
