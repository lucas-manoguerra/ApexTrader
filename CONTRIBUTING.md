# Contribuir a ApexTrader

## Principios

Toda contribución debe preservar la separación de responsabilidades, la independencia del dominio, la seguridad de Rust y la capacidad de evolución del proyecto. Los cambios deben ser pequeños, verificables y coherentes con las decisiones registradas.

La coordinación entre colaboradores sigue el protocolo de [trabajo en equipo](docs/development/team-collaboration.md). Ese documento define cómo reclamar tareas, evitar solapamientos, compartir contratos y mantener ramas independientes.

## Flujo de trabajo

1. Consultar Issues y Pull Requests abiertos antes de comenzar. Buscar trabajo equivalente o en curso.
2. Acordar una tarea con alcance, criterios de aceptación y archivos o módulos principales.
3. Actualizar referencias remotas y crear una rama propia desde `main`, salvo dependencia explícita documentada entre PRs.
4. Leer `AGENTS.md` y la documentación de las áreas afectadas.
5. Inspeccionar implementación, consumidores, pruebas y contratos antes de editar.
6. Implementar el cambio mínimo cohesivo sin modificar archivos ajenos a la tarea.
7. Ejecutar controles locales aplicables y revisar el diff completo.
8. Abrir un Pull Request con objetivo, alcance, pruebas, riesgos y dependencias.
9. Atender revisión y mantener la rama sincronizada sin sobrescribir trabajo de otros.
10. Integrar únicamente cuando se cumplan los criterios de integración vigentes del proyecto.

## Coordinación obligatoria

- Una tarea debe tener un responsable principal visible en su Issue o PR.
- Antes de editar un archivo compartido, avisar en la Issue/PR y acordar quién integra el cambio.
- No trabajar simultáneamente en el mismo contrato, módulo o conjunto de archivos sin coordinación explícita.
- Si una tarea depende de otra PR, declararlo en la descripción y enlazarla; no copiar cambios manualmente ni abrir ramas divergentes con el mismo trabajo.
- Si aparece una necesidad fuera del alcance, abrir una tarea separada o solicitar aprobación antes de ampliarlo.
- No resolver conflictos eliminando cambios ajenos. Comparar ambas intenciones y coordinar con su autor.
- No hacer push a ramas de otra persona ni reescribir su historial sin autorización.
- No trabajar directamente sobre `main`.

## Controles obligatorios

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
bash scripts/verify_no_code_comments.sh
bash scripts/verify_architecture.sh
bash scripts/verify_lib_docs.sh
bash scripts/audit_dependencies.sh
```

La auditoría de dependencias requiere `cargo-audit` y `cargo-deny`. Si una herramienta no está instalada o un control no puede ejecutarse en el sistema local, declararlo en el PR; nunca marcarlo como ejecutado.

## Estándares de Rust

- Usar Rust estable y Edition 2024.
- Preferir ownership y borrowing claros antes que clonaciones innecesarias.
- Modelar estados e invariantes con tipos.
- Usar errores tipados y propagación explícita.
- Evitar `unwrap()`, `expect()` y `panic!` en flujos recuperables de producción.
- Mantener `unsafe` fuera del código ordinario; cualquier necesidad requiere revisión específica de seguridad y una justificación en documentación externa.
- Mantener APIs públicas mínimas y contratos estables.
- No introducir dependencias sin evaluar mantenimiento, licencia, superficie de seguridad y costo de compilación.

## Política de comentarios

Está prohibido agregar comentarios al código, incluidos comentarios Rustdoc, comentarios de bloque, TODO, FIXME y fragmentos de código comentados. La documentación explicativa se escribe en Markdown fuera del código. No se deben eliminar comentarios de terceros dentro de dependencias vendorizadas sin una decisión explícita; los artefactos generados y directorios de dependencias no forman parte del código mantenido por el proyecto.

## Arquitectura

Las dependencias deben apuntar hacia el dominio. La interfaz, el renderizador, el sistema operativo y los proveedores externos son detalles de infraestructura. No se permiten dependencias circulares ni acceso directo de capas externas a detalles internos de otras capas.

La extracción de un módulo a un crate independiente requiere una frontera de dominio o de compilación justificable, contratos claros y un beneficio operativo verificable.

## Pruebas y rendimiento

Cada corrección debe incluir una prueba de regresión cuando sea viable. Los cambios en transformaciones, interacción, ingesta o renderizado deben contemplar casos límite y propiedades relevantes. Las optimizaciones deben apoyarse en benchmarks reproducibles y no degradar claridad ni seguridad.

## Commits

Usar Conventional Commits:

```text
tipo(ámbito): descripción breve
```

Tipos habituales: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci` y `chore`.

## Decisiones

Las decisiones arquitectónicas relevantes se registran como ADR numerados en `docs/decisions/`. Los cambios de comportamiento visible también deben reflejarse en `CHANGELOG.md`.
