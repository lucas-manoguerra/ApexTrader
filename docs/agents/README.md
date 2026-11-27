# Harness y agentes

## Modelo de coordinación

El trabajo automatizado utiliza un agente coordinador que define alcance, asigna responsabilidades y verifica la integración. El mismo protocolo de coordinación se aplica a colaboradores humanos: consultar tareas abiertas, declarar ownership y evitar ediciones simultáneas de contratos compartidos. La guía canónica es [trabajo en equipo](../development/team-collaboration.md).

Los agentes especializados no modifican contratos fuera de su ámbito sin aprobación del coordinador. Ningún agente trabaja directamente sobre `main`.

## Roles

| Rol | Responsabilidad | Límites |
|---|---|---|
| Coordinación | Definir alcance, responsables, dependencias y secuencia de integración | No asignar el mismo archivo o contrato a dos responsables sin acuerdo |
| Arquitectura | Límites de módulos, dependencias y ADR | No implementar cambios funcionales amplios |
| Rust | Implementación idiomática y contratos | No redefinir arquitectura sin ADR o aprobación |
| Testing | Unit, integration, property y regression tests | No debilitar assertions para hacer pasar CI |
| Performance | Benchmarks, profiling y análisis de asignaciones | No optimizar sin línea base |
| Seguridad | Dependencias, unsafe, validación y exposición de datos | No aprobar excepciones sin evidencia |
| Documentación | README, guías, ADR e instrucciones | No describir capacidades no verificadas |
| Revisión | Analizar diff, riesgos y criterios de aceptación | No sustituir los gates automatizados |

Una persona puede desempeñar varios roles. Los roles describen responsabilidades, no requieren agentes o equipos separados.

## Protocolo de tarea

1. Leer `AGENTS.md` y la documentación del área.
2. Consultar Issues y PRs para detectar trabajo existente.
3. Identificar archivos, módulos, contratos y pruebas afectados.
4. Definir alcance y criterios de aceptación antes de editar.
5. Confirmar ownership de archivos y contratos compartidos.
6. Implementar el cambio mínimo cohesivo.
7. Ejecutar pruebas y controles relacionados.
8. Revisar el diff completo y comprobar la política de cero comentarios.
9. Informar resultados, limitaciones y riesgos remanentes.

## Reglas de ejecución

- Ningún agente trabaja directamente sobre `main`.
- No se permiten cambios fuera del alcance acordado.
- No se agregan dependencias por conveniencia sin evaluación.
- No se crean capas o crates sin una frontera de responsabilidad justificable.
- No se ocultan fallos ni se debilitan tests.
- No se consideran terminadas tareas que no tengan validación observable.
- No se modifican contratos compartidos sin coordinación con sus consumidores.
- No se sobrescriben ramas, commits o archivos de otro responsable.
- La documentación externa se actualiza junto con el cambio que la vuelve obsoleta.

## Criterios de salida

Una tarea queda lista para revisión cuando el diff es autocontenido, el contrato es claro, las pruebas relevantes pasan, la documentación está alineada y no hay comentarios en el código modificado.
