# Contribuir a ApexTrader

Gracias por interesarte en ApexTrader. Este documento explica cómo trabajar con el proyecto.

## Requisitos previos

- Rust toolchain estable (edición 2024).
- `cargo fmt`, `cargo clippy` disponibles.
- Git configurado con tu nombre y correo.

## Flujo de trabajo

1. Crea una rama corta desde `main`: `feature/...`, `fix/...`, `refactor/...` o `docs/...`.
2. Implementa el cambio siguiendo los estándares en `docs/`.
3. Ejecuta los gates de calidad (ver más abajo).
4. Abre un Pull Request hacia `main`.
5. Un revisor aprueba y se fusiona en `main`.

## Gates de calidad obligatorios

Cada cambio debe pasar antes de integrarse:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Convenciones de commits

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
tipo(ámbito): descripción breve
```

Tipos: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`.

Ejemplo:

```
feat(chart): add horizontal price scale
```

## Documentación

Cada decisión de arquitectura importante queda registrada como ADR en `docs/decisions/`.

## Código

- Sin `unwrap()` ni `expect()` en caminos de producción normales.
- Sin `panic!()` para errores recuperables.
- `unsafe` solo con justificación documentada y pruebas.
- APIs públicas pequeñas, explícitas y documentadas.

## Comunidad

Este es un proyecto personal en construcción. Abre un issue para proponer cambios o reportar problemas.