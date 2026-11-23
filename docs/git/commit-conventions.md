# Convenciones de commits

Usamos [Conventional Commits](https://www.conventionalcommits.org/).

## Formato

```
tipo(ámbito): descripción breve
```

## Tipos principales

| Tipo | Uso |
|------|-----|
| `feat` | Funcionalidad nueva |
| `fix` | Corrección de error |
| `refactor` | Reestructuración sin cambio funcional intencional |
| `perf` | Mejora de rendimiento |
| `test` | Pruebas |
| `docs` | Documentación |
| `build` | Dependencias y construcción |
| `ci` | Integración continua |
| `chore` | Mantenimiento general |

## Ejemplos

```
feat(chart): add horizontal price scale
fix(market): handle invalid tick sequence
refactor(chart-engine): isolate viewport state
perf(renderer): reduce geometry allocations
test(trading): add order validation tests
docs(architecture): document module boundaries
build(deps): update egui dependency
```

## Reglas

- Los commits tienen que ser pequeños, coherentes y representar una unidad lógica de trabajo.
- Sin `Co-Authored-By` ni atribución de IA. Solo conventional commits.
- El ámbito es opcional pero recomendado cuando el cambio toca un módulo claro.
