# Proceso de releases

## Versionado

Usamos [Semantic Versioning](https://semver.org/lang/es/): `MAJOR.MINOR.PATCH`.

Ejemplos:

- `0.1.0`: primera versión experimental.
- `0.2.0`: funcionalidad compatible nueva.
- `0.2.1`: corrección.
- `1.0.0`: primera versión estable con contratos públicos definidos.

Durante `0.x` se pueden cambiar APIs con mayor libertad, pero se documentan los cambios importantes.

## Qué mantener

- `Cargo.lock` versionado (para la aplicación).
- Tags Git para releases.
- `CHANGELOG.md` actualizado con cada release.
- ADR en `docs/decisions/` para decisiones importantes.
- Registro de cambios incompatibles.

## Cómo hacer un release

1. Verificar que `main` está limpio y pasa todos los gates.
2. Actualizar versión en `Cargo.toml`.
3. Actualizar `CHANGELOG.md`.
4. Crear commit `release: prepare vX.Y.Z`.
5. Crear tag Git `vX.Y.Z`.
6. Push del commit y el tag.
7. GitHub Actions puede publicar automáticamente (opcional, por definir).