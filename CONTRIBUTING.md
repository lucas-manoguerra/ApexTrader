# Contribuir a ApexTrader

## Principios

Toda contribución debe preservar la separación de responsabilidades, la independencia del dominio, la seguridad de Rust y la capacidad de evolución del proyecto. Los cambios deben ser pequeños, verificables y coherentes con las decisiones registradas.

## Flujo de trabajo

1. Actualizar la rama de trabajo desde `main`.
2. Crear una rama descriptiva con prefijo `feat/`, `fix/`, `refactor/`, `perf/`, `docs/` o `chore/`.
3. Definir el alcance, los contratos afectados y los riesgos.
4. Implementar el cambio con pruebas apropiadas.
5. Ejecutar los controles locales aplicables.
6. Abrir un Pull Request con contexto, decisiones, pruebas y riesgos.
7. Resolver observaciones y verificar nuevamente la CI antes de integrar.

## Controles obligatorios

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
python scripts/verify_no_code_comments.py
```

Los scripts de arquitectura y dependencias deben ejecutarse en PowerShell 7:

```powershell
pwsh -File scripts/verify_architecture.ps1
pwsh -File scripts/audit_dependencies.ps1
```

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
