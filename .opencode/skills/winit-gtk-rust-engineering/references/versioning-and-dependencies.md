# Versiones, Cargo y compatibilidad

## Identidad del paquete

En el snapshot documentado:

- Nombre del paquete Cargo: `winit-gtk`.
- Nombre de la biblioteca Rust: `winit` (`[lib] name = "winit"`).
- Versión publicada visible en docs.rs: `0.29.1`.
- Edition: `2021`.
- `raw-window-handle`: `0.5`.
- GTK-rs del target Unix: `gtk`, `gdk`, `glib`, `gio`, `cairo-rs` en la línea `0.18`.

No infieras de estos datos la versión de Rust mínima soportada si el `Cargo.toml` no la declara. Comprueba el `rust-version` del artefacto que se utiliza y la toolchain del proyecto.

## Alias de dependencia

El alias Cargo de dependencia y el nombre de biblioteca pueden diferir. Antes de escribir el `Cargo.toml`, comprueba la forma de consumo que corresponde al paquete seleccionado. Una dependencia directa puede necesitar el nombre de paquete explícito, por ejemplo:

```toml
[dependencies]
winit = { package = "winit-gtk", version = "0.29.1" }
```

Este fragmento es una ilustración para el snapshot; valida que coincida con la fuente y el lockfile del proyecto. No agregues simultáneamente `winit` upstream y `winit-gtk` sin entender cuál crate importan los módulos y las dependencias transitivas.

## Contradicciones que el agente debe detectar

El README del crate contiene una tabla de compatibilidad antigua que relaciona `winit 0.28.7` con `winit-gtk 0.29`, y `winit 0.28.6` con `winit-gtk 0.28`. Sin embargo, la documentación generada para el paquete actual muestra `winit 0.29.1` y el manifiesto publicado declara la versión `0.29.1`. Esa tabla no constituye por sí sola una matriz completa de compatibilidad.

El README también describe que los módulos de plataforma X11/Wayland se sustituyen por GTK, pero el manifiesto expone features `x11` y `wayland`, y el código fuente contiene cfgs y módulos Unix específicos. No inventes un feature `gtk` ni deduzcas la ruta de compilación por el texto del README: inspecciona el `Cargo.toml`, `build.rs`, source y `cargo tree -e features`.

## Comandos de inspección

```bash
cargo tree -i winit
cargo tree -i winit-gtk
cargo tree -i raw-window-handle
cargo tree -e features -i winit-gtk
cargo metadata --format-version 1
cargo check -vv
```

Algunos comandos pueden devolver un error si un nombre de paquete no está presente; eso no es prueba de que la integración sea compatible. Inspecciona la salida real y los manifiestos.

## Política de actualización

Para cada actualización:

1. Fija la versión candidata y lee su manifiesto publicado.
2. Revisa changelog, README, API generada y source.
3. Compara `raw-window-handle`, GTK-rs, MSRV, features y targets.
4. Actualiza el lockfile de forma controlada.
5. Ejecuta compilaciones de la matriz de plataformas.
6. Registra cambios de API y pruebas manuales necesarias.

No hagas actualizaciones masivas de `winit-gtk`, GTK-rs, `wgpu` y `raw-window-handle` en un solo paso si no puedes aislar la causa de una regresión.
