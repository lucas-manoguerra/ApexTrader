# Harness y agentes

## Modelo de coordinación

El trabajo automatizado utiliza un agente coordinador que define alcance, asigna responsabilidades y verifica la integración. Los agentes especializados no modifican contratos fuera de su ámbito sin aprobación del coordinador.

## Roles

| Rol | Responsabilidad | Límites |
|---|---|---|
| Arquitectura | Límites de módulos, dependencias y ADR | No implementar cambios funcionales amplios |
| Rust | Implementación idiomática y contratos | No redefinir arquitectura sin ADR o aprobación |
| Testing | Unit, integration, property y regression tests | No debilitar assertions para hacer pasar CI |
| Performance | Benchmarks, profiling y análisis de asignaciones | No optimizar sin línea base |
| Seguridad | Dependencias, unsafe, validación y exposición de datos | No aprobar excepciones sin evidencia |
| Documentación | README, guías, ADR e instrucciones | No describir capacidades no verificadas |
| Revisión | Analizar diff, riesgos y criterios de aceptación | No sustituir los gates automatizados |

## Protocolo de tarea

1. Leer `AGENTS.md` y la documentación del área.
2. Identificar archivos, módulos, contratos y pruebas afectados.
3. Definir alcance y criterios de aceptación antes de editar.
4. Implementar el cambio mínimo cohesivo.
5. Ejecutar pruebas y controles relacionados.
6. Revisar el diff completo y comprobar la política de cero comentarios.
7. Informar resultados, limitaciones y riesgos remanentes.

## Reglas de ejecución

- Ningún agente trabaja directamente sobre `main`.
- No se permiten cambios fuera del alcance acordado.
- No se agregan dependencias por conveniencia sin evaluación.
- No se crean capas o crates sin una frontera de responsabilidad justificable.
- No se ocultan fallos ni se debilitan tests.
- No se consideran terminadas tareas que no tengan validación observable.
- La documentación externa se actualiza junto con el cambio que la vuelve obsoleta.

## Criterios de salida

Una tarea queda lista para revisión cuando el diff es autocontenido, el contrato es claro, las pruebas relevantes pasan, la documentación está alineada y no hay comentarios en el código modificado.
