# Documentación de ApexTrader

Esta documentación define el diseño, los estándares y los procedimientos del proyecto. La implementación es la fuente de verdad para el comportamiento ejecutable; cuando una diferencia sea detectada, debe corregirse el código o actualizarse la documentación mediante un cambio revisable.

## Índice

| Área | Contenido |
|---|---|
| [Arquitectura](architecture/README.md) | Capas, dependencias, límites y evolución |
| [Ingeniería](engineering/standards.md) | Estándares Rust, diseño, errores y política de comentarios |
| [Desarrollo](development/workflow.md) | Flujo de trabajo técnico |
| [Trabajo en equipo](development/team-collaboration.md) | Coordinación, asignación de tareas, ramas y prevención de conflictos |
| [Agentes](agents/README.md) | Roles, harness y protocolo de ejecución |
| [Calidad](quality/README.md) | Gates, pruebas y CI |
| [Seguridad](security/README.md) | Dependencias, unsafe y gestión de riesgos |
| [Rendimiento](performance/README.md) | Benchmarks, profiling y presupuestos |
| [Decisiones](decisions/README.md) | Architecture Decision Records |

## Gobierno documental

- Los documentos deben tener un responsable lógico y un alcance claro.
- Las decisiones permanentes se registran como ADR.
- Las instrucciones para agentes no sustituyen los estándares técnicos.
- No duplicar una regla en múltiples documentos si puede enlazarse a una fuente canónica.
- Los ejemplos deben compilar o identificarse expresamente como pseudocódigo.
- La documentación debe distinguir capacidades implementadas, experimentales y planificadas.
- Cuando cambie un procedimiento, actualizar la fuente canónica y sus enlaces; no mantener copias divergentes.
