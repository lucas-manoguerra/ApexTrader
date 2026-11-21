# Integración de winit-gtk con wgpu

## No asumas compatibilidad automática

Una `wgpu::Surface` necesita una fuente de handle de ventana compatible con la versión de `raw-window-handle` usada por `wgpu`. `winit-gtk` 0.29.1 declara `raw-window-handle 0.5`; `wgpu 30.0.2` declara `raw-window-handle 0.6.2`. Los tipos de dos versiones mayores distintas del crate no son el mismo tipo Rust y no se convierten automáticamente.

Antes de escribir la integración ejecuta:

```bash
cargo tree -i raw-window-handle
cargo tree -i winit-gtk
cargo tree -i wgpu
```

Inspecciona los `Cargo.toml` y el source real de ambas versiones. Confirma las traits que implementa la ventana (`HasRawWindowHandle`/`HasRawDisplayHandle` en la línea antigua frente a traits actuales de `raw-window-handle`) y los requisitos del constructor de `Surface` de la versión instalada de `wgpu`.

## Opciones si no coinciden

Evalúa en este orden, según el proyecto:

1. Elegir una combinación de versiones de renderer y windowing que comparta el mismo contrato de handles y esté mantenida.
2. Mantener un fork de `winit-gtk` migrado a la versión requerida de `raw-window-handle`, con tests y CI de plataforma.
3. Usar otro backend de ventanas compatible con la versión elegida de `wgpu`.
4. Solo considerar un adaptador de handles si existe una estrategia documentada y segura para esa plataforma, con revisión especializada. No inventes una conversión ni uses transmute.

No presentes una combinación como funcional hasta que compile y pase una prueba de creación, resize y presentación de superficie en el entorno de destino.

## Arquitectura del renderer

Separa:

- `WindowHost`: vida de ventana y eventos.
- `SurfaceHost`: superficie y configuración de presentación.
- `GpuContext`: instance/adapter/device/queue, según la API concreta de wgpu.
- `Renderer`: pipelines, recursos y render passes.

La ventana debe vivir tanto como los recursos que dependen de su handle. Documenta el ownership y evita fugas artificiales para satisfacer `'static`.

## Eventos que afectan al renderer

- Resize: recalcular tamaño físico y reconfigurar superficie cuando las dimensiones sean válidas.
- Scale factor: actualizar dimensiones físicas y layout/UI.
- Minimized/zero size: suspender la presentación.
- Redraw: adquirir el frame, dibujar y presentar de acuerdo con la API concreta de wgpu.
- Surface lost/outdated/timeout: aplicar política de recuperación específica de la versión.
- Cierre: detener workers relacionados y liberar renderer/surface en orden seguro.

No llames a `device.poll`, esperas de GPU o readbacks bloqueantes en handlers de UI sin medir y justificar el impacto.

## Regla de versionado

La matriz de versiones probadas debe incluir como mínimo:

- Rust toolchain y target.
- `winit-gtk` y commit/tag si es fork.
- GTK-rs / GTK del sistema.
- `raw-window-handle`.
- `wgpu`.
- Backend/display server y driver probado.

Guarda esta matriz en la documentación de arquitectura del proyecto.
