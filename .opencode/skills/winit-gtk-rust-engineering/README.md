# winit-gtk-rust-engineering

Skill de ingeniería para agentes de código que trabajen con `winit-gtk` en proyectos profesionales de Rust.

## Objetivo

Guiar la integración de ventanas, event loops, eventos de entrada, GTK, concurrencia y renderers con un diseño mantenible y con comprobaciones de compatibilidad explícitas.

## Estructura

```text
winit-gtk-rust-engineering/
├── SKILL.md
├── README.md
├── references/
│   ├── versioning-and-dependencies.md
│   ├── event-loop-and-architecture.md
│   ├── gtk-threading.md
│   ├── input-and-windows.md
│   ├── wgpu-integration.md
│   ├── testing-and-platforms.md
│   └── anti-patterns.md
└── examples/
    └── README.md
```

## Instalación

Copia la carpeta `winit-gtk-rust-engineering` al directorio de skills que utilice tu agente. La ruta concreta depende de la herramienta (OpenCode, Codex, Claude Code u otra).

## Uso

Pide al agente que consulte esta skill antes de crear o modificar la capa de ventanas. La skill exige inspeccionar primero `Cargo.toml`, `Cargo.lock`, las features, el target y las versiones reales de `winit-gtk`, GTK-rs y `raw-window-handle`.

## Precaución de compatibilidad

La documentación enlazada resuelve `winit-gtk 0.29.1`. Ese snapshot declara `raw-window-handle 0.5`; `wgpu 30.0.2` declara `raw-window-handle 0.6.2`. No presupongas que ambos pueden integrarse directamente. Verifica las versiones que realmente usa tu proyecto antes de diseñar la creación de `wgpu::Surface`.

## Fuentes

- https://docs.rs/winit-gtk/latest/winit/
- https://github.com/wusyong/winit-gtk
- https://docs.rs/wgpu/latest/wgpu/
