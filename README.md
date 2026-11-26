# ApexTrader

ApexTrader es una terminal de trading de escritorio desarrollada en Rust. El proyecto prioriza una arquitectura modular, límites explícitos entre responsabilidades, rendimiento predecible y una evolución técnica sostenible.

> **Estado:** versión experimental `0.1.0`. Las capacidades disponibles y las limitaciones conocidas deben consultarse en la documentación de arquitectura y en el registro de cambios.

## Objetivos de ingeniería

- Mantener el dominio independiente de la interfaz, los proveedores y la infraestructura.
- Separar casos de uso, integración de mercado, visualización y renderizado mediante contratos explícitos.
- Preferir Rust idiomático, tipos que expresen invariantes y errores recuperables.
- Medir antes de optimizar y proteger el rendimiento con benchmarks y pruebas.
- Automatizar formato, análisis estático, pruebas, seguridad y límites arquitectónicos.
- Documentar decisiones y procedimientos fuera del código fuente.

## Arquitectura actual

| Área | Responsabilidad |
|---|---|
| `domain` | Tipos, invariantes y reglas centrales del negocio |
| `application` | Casos de uso, coordinación y composición de la aplicación |
| `market` | Ingesta, validación y representación de datos de mercado |
| `chart` | Estado, transformaciones, layout e interacción de gráficos |
| `renderer` | Representación visual y backends de renderizado |
| `errors` | Errores tipados compartidos |
| `logging` | Inicialización y configuración de observabilidad |

La arquitectura se describe en [docs/architecture/README.md](docs/architecture/README.md). La implementación puede evolucionar; esta tabla no autoriza dependencias que contradigan los límites documentados.

## Requisitos

- Rust estable con soporte para Edition 2024.
- Cargo y los componentes `rustfmt` y `clippy`.
- PowerShell 7 para los scripts de verificación existentes.
- Python 3 para los controles estáticos del repositorio.

## Ejecutar

```bash
cargo run
```

## Validación local

Ejecutar antes de solicitar revisión:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
python3 scripts/verify_no_code_comments.py
pwsh -File scripts/verify_architecture.ps1
pwsh -File scripts/audit_dependencies.ps1
```

La CI es la autoridad final para los controles automatizados. Los comandos que dependan del sistema operativo deben ejecutarse en un entorno compatible.

## Política de código

**Los comentarios están prohibidos en todo el código y la configuración ejecutable del repositorio.** No se permiten comentarios de línea, bloque, documentación Rustdoc, TODO/FIXME ni código comentado. Las explicaciones pertenecen a la documentación Markdown externa. El código debe expresar su intención mediante nombres, tipos, módulos y contratos claros.

La política y su verificación están definidas en [docs/engineering/standards.md](docs/engineering/standards.md).

## Documentación

- [Índice documental](docs/README.md)
- [Arquitectura](docs/architecture/README.md)
- [Estándares de ingeniería](docs/engineering/standards.md)
- [Flujo de desarrollo](docs/development/workflow.md)
- [Calidad y validación](docs/quality/README.md)
- [Seguridad](docs/security/README.md)
- [Rendimiento](docs/performance/README.md)
- [Decisiones de arquitectura](docs/decisions/README.md)
- [Contribuir](CONTRIBUTING.md)
- [Cambios](CHANGELOG.md)

## Contribuciones

Toda modificación debe tener un alcance claro, respetar los límites arquitectónicos, incluir pruebas proporcionales al riesgo y superar los controles de calidad. Consultar [CONTRIBUTING.md](CONTRIBUTING.md) y [AGENTS.md](AGENTS.md) antes de trabajar.

## Licencia

Proyecto privado en cuanto a su distribución. No se concede una licencia de redistribución mediante este repositorio.
