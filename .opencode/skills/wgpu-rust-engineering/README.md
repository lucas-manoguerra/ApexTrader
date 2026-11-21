# wgpu-rust-engineering skill

Skill portable para agentes de programación. El archivo principal es `SKILL.md`.

## Uso

Copia la carpeta `wgpu-rust-engineering-skill` dentro del directorio de skills que utilice tu agente (por ejemplo, OpenCode u otro agente compatible con archivos `SKILL.md`). Luego invoca la skill en tareas que involucren `wgpu` en Rust.

Antes de aplicarla en un repositorio existente, el agente debe leer `Cargo.toml` y `Cargo.lock` y adaptar todas las recomendaciones a la versión real de `wgpu` del proyecto.
