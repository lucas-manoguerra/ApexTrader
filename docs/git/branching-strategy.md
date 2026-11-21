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

1. Crear rama corta desde `main`.
2. Implementar en commits pequeños y coherentes.
3. Abrir Pull Request hacia `main`.
4. Revisar y aprobar.
5. Fusionar en `main`.
6. Eliminar la rama de trabajo.

## Recomendación inicial

No crear ramas permanentes `develop`, `release`, `staging` ni `testing` desde el primer día. Añaden complejidad sin beneficio en esta etapa.

Cuando haya equipo, despliegues frecuentes o varias versiones soportadas, reevaluaremos la estrategia.