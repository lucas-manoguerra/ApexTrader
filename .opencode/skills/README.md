# Rust Library Engineering Skills

Paquete de 10 skills independientes para agentes de programación Rust. Cada directorio contiene una skill autocontenida (`SKILL.md`) y un `README.md` con propósito, versión de referencia y documentación oficial.

## Skills incluidas

| Skill | Crate | Referencia observada (18-11-2026) | Propósito |
|---|---|---:|---|
| `protest-property-testing` | `protest` | ver SKILL.md | Diseño, ejecución y depuración de pruebas basadas en propiedades con protest. |
| `thiserror-error-design` | `thiserror` | ver SKILL.md | Diseño de errores tipados, encadenamiento de fuentes y conversión explícita de errores. |
| `pollster-async-bridge` | `pollster` | ver SKILL.md | Uso seguro de block_on como puente puntual entre código síncrono y futures. |
| `bytemuck-derive-layout-contracts` | `bytemuck_derive` | ver SKILL.md | Derives para declarar contratos de representación binaria seguros y verificables. |
| `bytemuck-memory-casting` | `bytemuck` | ver SKILL.md | Conversión segura entre valores Rust y bytes con garantías explícitas de validez y alineación. |
| `glyphon-gpu-text-rendering` | `glyphon` | ver SKILL.md | Renderizado eficiente de texto 2D mediante wgpu, cosmic-text y atlas de glifos. |
| `eframe-app-framework` | `eframe` | ver SKILL.md | Arquitectura de aplicaciones egui con eframe en desktop y WebAssembly. |
| `egui-immediate-mode-ui` | `egui` | ver SKILL.md | Diseño de interfaces immediate-mode, estado, layout, interacción, accesibilidad y rendimiento. |
| `tracing-subscriber-observability` | `tracing-subscriber` | ver SKILL.md | Composición de subscribers/layers, formatos, filtros, salida JSON y configuración de observabilidad. |
| `tracing-instrumentation` | `tracing` | ver SKILL.md | Instrumentación estructurada con spans, events, campos y propagación contextual async. |

> Las versiones son una fotografía de las páginas docs.rs consultadas el 18-11-2026, no restricciones obligatorias ni garantías de MSRV. La skill siempre exige validar `Cargo.toml`, `Cargo.lock`, features, toolchain y documentación de la versión realmente resuelta antes de generar código.

## Uso

Copia el directorio de la skill elegida a la carpeta de skills de tu agente, manteniendo `SKILL.md`. Si el agente carga skills por nombre, usa el nombre del directorio como identificador. También puedes instalar el paquete completo y pedirle al agente que active solo la skill relevante para cada tarea.

## Reglas transversales

- No asumir que `latest` coincide con la versión del proyecto.
- No inventar APIs, features, tipos, macros ni MSRV.
- Ejecutar `cargo check`, tests, Clippy y pruebas de plataforma según alcance.
- Hacer benchmark/profile antes de optimizar.
- Separar lógica de dominio de detalles de cada crate.
- Revisar seguridad, privacidad, errores y compatibilidad semver.

## Interoperabilidad de alto nivel

- `eframe` se apoya en `egui`; ambos deben tener versiones compatibles.
- `glyphon` integra texto en `wgpu`; la versión observada 0.12.0 depende de `wgpu ^30.0.0`.
- `bytemuck_derive` genera contratos para traits consumidos por `bytemuck`; activa la feature `derive` en `bytemuck` o depende directamente del proc-macro solo si hace falta.
- `tracing` emite spans/events; `tracing-subscriber` los recoge, filtra y exporta. Las librerías instrumentan; el binario configura el subscriber.
- `pollster` bloquea un hilo para completar un future; no es un runtime de tareas ni debe bloquear el loop de UI.
