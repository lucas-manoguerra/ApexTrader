# Estrategia de ramas

Usamos *trunk-based development* con ramas cortas.

## Ramas

| Rama | Propósito |
|------|-----------|
| `main` | Código estable, integrado, comprobado y siempre compilable. |
| `feature/...` | Funcionalidades nuevas. |
| `fix/...` | Correcciones de errores. |
| `refactor/...` | Cambios internos sin cambio funcional intencional. |
| `docs/...` | Cambios de documentación. |

## Flujo

1. Creá una rama corta desde `main`.
2. Implementá en commits pequeños y coherentes.
3. Abrí un Pull Request hacia `main`.
4. Revisá y aprobá.
5. Fusioná en `main`.
6. Eliminá la rama de trabajo.

## Recomendación inicial

No crees ramas permanentes `develop`, `release`, `staging` ni `testing` desde el primer día. Añaden complejidad sin beneficio en esta etapa.

Cuando haya equipo, despliegues frecuentes o varias versiones soportadas, reevaluamos la estrategia.
