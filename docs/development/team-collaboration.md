# Protocolo de trabajo en equipo

## Objetivo

Permitir que varias personas contribuyan en paralelo sin duplicar tareas, sobrescribir trabajo ajeno ni introducir cambios incompatibles. La coordinación se realiza mediante Issues, ramas y Pull Requests visibles en GitHub; los acuerdos importantes no deben quedar únicamente en mensajes privados.

## Fuente de verdad del trabajo

- Una Issue representa una tarea, bug, mejora o decisión pendiente.
- Una Issue debe tener un único responsable principal. Puede tener colaboradores, pero no dos responsables que editen el mismo alcance de forma independiente.
- Antes de empezar, buscar Issues y PRs abiertos con palabras clave, archivos y módulos relacionados.
- La Issue debe describir problema, resultado esperado, criterios de aceptación, límites y dependencias conocidas.
- El responsable actualiza la Issue cuando cambia el alcance, aparece un bloqueo o se descubre que otra tarea cubre el mismo problema.
- Si una tarea requiere coordinación entre dos personas, acordar explícitamente qué parte hace cada una y qué contrato compartido se considera estable.

No iniciar una segunda implementación de una tarea ya asignada sin hablar con su responsable. Si se descubre duplicación, detener la parte solapada y acordar cuál cambio continúa.

## Gestión de hallazgos fuera del alcance

Durante el análisis o la implementación pueden aparecer problemas y oportunidades que no pertenecen al objetivo de la PR actual. Para preservar PRs pequeñas y permitir que el equipo distribuya el trabajo:

- Si el hallazgo no es necesario para que la PR actual sea correcta, segura y completa según sus criterios de aceptación, no implementarlo dentro de esa PR.
- Buscar primero una Issue abierta equivalente. Si existe, añadir contexto y enlazarla en la PR actual; no crear duplicados.
- Si no existe, abrir una Issue independiente con problema observado, impacto, evidencia reproducible, resultado esperado, criterios de aceptación y módulos o archivos probablemente afectados.
- Añadir a la Issue una referencia a la PR donde se descubrió y explicar por qué quedó fuera de alcance.
- No asignar automáticamente una Issue a otra persona. Dejarla disponible para priorización y asignación coordinada.
- Si el hallazgo bloquea la corrección, la seguridad, una invariante o un criterio de aceptación de la PR actual, tratarlo como bloqueante y resolverlo antes de integrar.
- Si no está claro si es bloqueante, detener la ampliación del cambio y acordar el alcance con el responsable de la PR.

Clasificación recomendada para Issues:
- **Bug:** comportamiento incorrecto o regresión independiente.
- **Mejora:** capacidad o calidad adicional que no exige la PR actual.
- **Deuda técnica:** costo de mantenimiento, acoplamiento o riesgo acumulado.
- **Investigación:** pregunta técnica que requiere evidencia antes de decidir.
- **Bloqueante:** defecto que impide completar correctamente la tarea actual; normalmente se enlaza a la PR y se resuelve dentro de su alcance o mediante dependencia explícita.

La descripción de la Issue debe ser accionable, no una lista vaga de ideas. Si el alcance es grande, dividirlo en tareas pequeñas después de una investigación inicial.

## División del trabajo en paralelo

Priorizar tareas que puedan integrarse de forma independiente: documentación, tests de un contrato estable, correcciones en módulos distintos o mejoras aisladas.

Antes de dividir una tarea:

1. Identificar módulos, archivos, APIs, tipos y pruebas que probablemente cambien.
2. Detectar contratos compartidos y consumidores.
3. Definir interfaces y criterios de aceptación antes de repartir implementación.
4. Asignar un responsable por cada conjunto de archivos.
5. Acordar el orden de integración si una parte depende de otra.

No dividir trabajo solo por cantidad de archivos. Dos cambios en archivos diferentes pueden seguir chocando si alteran el mismo contrato o la misma invariante.

## Reserva temporal de archivos y contratos

La reserva es un acuerdo de coordinación, no un bloqueo técnico de GitHub.

- Anunciar en la Issue o PR los archivos, módulos o contratos principales que se van a modificar.
- Consultar si alguien ya está trabajando en ellos.
- Mientras una tarea esté activa, cualquier cambio concurrente en esos mismos archivos debe coordinarse con su responsable.
- La reserva termina al integrar o cerrar la tarea, o cuando el responsable comunica que libera el alcance.
- Si una tarea descubre que necesita modificar un archivo reservado por otra persona, detenerse y coordinar antes de editar.
- No mantener reservas indefinidas: actualizar el estado o liberar el trabajo si queda pausado.

## Ramas y sincronización

- Nunca trabajar directamente sobre `main`.
- Usar una rama por tarea, con prefijos `feat/`, `fix/`, `refactor/`, `perf/`, `test/`, `docs/`, `ci/` o `chore/`.
- Crear ramas desde la última versión de `origin/main` para tareas independientes.
- No reutilizar una rama para una tarea distinta.
- Mantener ramas de corta duración y PRs pequeñas.
- Sincronizar una rama con `main` mediante la estrategia acordada por el equipo; antes de rebase o force-push, confirmar que nadie más trabaja sobre esa rama.
- No hacer force-push a ramas compartidas ni sobrescribir commits de otra persona.
- No crear PRs apiladas salvo que exista una dependencia real. Si se apilan, declararlo en todas las PRs implicadas, explicar el orden de integración y evitar incluir cambios duplicados en la PR dependiente.
- La exclusión temporal de macOS actualmente obliga a conservar esa configuración en las ramas de trabajo que parten de PRs pendientes. Es una excepción operativa, no el modelo permanente de branching. No retirar ni duplicar esa modificación sin coordinarla.

## Pull Requests

Cada PR debe:

- Tener un objetivo único y un alcance revisable.
- Enlazar la Issue correspondiente.
- Declarar responsable principal, colaboradores, archivos/contratos compartidos y PRs dependientes.
- Explicar qué cambia y qué queda explícitamente fuera.
- Identificar cambios en APIs, invariantes, estado, errores, recursos y ciclo de vida.
- Incluir tests relevantes y documentación actualizada.
- Informar comandos ejecutados y controles que no pudieron ejecutarse.
- No incluir refactors, formato masivo ni cambios ajenos al objetivo.
- Mantenerse actualizada con las observaciones de revisión.

Una PR no está lista por el mero hecho de compilar localmente. La CI y la revisión deben verificar el cambio contra la base real de integración.

## Revisión y decisiones

- El autor no debe aprobar su propio cambio como única revisión.
- La revisión se enfoca primero en corrección, contratos, ownership, seguridad, exactitud de datos y regresiones; luego en estilo.
- Los cambios de arquitectura, contratos compartidos, dependencias, seguridad o persistencia requieren revisión explícita del responsable técnico del área.
- Las discrepancias se resuelven con evidencia, tests y decisiones documentadas, no con ediciones silenciosas.
- Las decisiones arquitectónicas duraderas se registran en ADR. Una conversación o comentario de PR no reemplaza un ADR cuando el cambio afecta límites estructurales.

## Integración y conflictos

- Integrar una PR solo cuando esté revisada, sus checks requeridos estén verdes y sus dependencias estén resueltas.
- Integrar PRs dependientes en el orden declarado.
- Después de integrar, actualizar las ramas independientes desde `main` y revisar si la nueva base vuelve obsoletos sus cambios.
- Si aparece un conflicto, resolverlo con conocimiento de ambos cambios y comunicarlo a sus autores.
- No descartar ni aceptar automáticamente un lado de un conflicto sin revisar su intención.
- El responsable de la integración verifica el estado final de CI sobre el commit que se va a integrar.

### Restricción temporal de CI

Mientras los runners macOS de GitHub Actions no estén disponibles de forma estable, ninguna PR debe integrarse. La exclusión temporal de macOS debe mantenerse explícita y no debe afirmarse cobertura completa de Windows/macOS/Linux.

Cuando el servicio se restablezca:

1. Restaurar macOS en todas las matrices afectadas.
2. Actualizar las ramas/PRs pendientes con esa configuración.
3. Ejecutar CI completa en Windows, macOS y Linux.
4. Resolver fallos antes de integrar.
5. Integrar las PRs pendientes en orden de dependencia, una por una.
6. Verificar CI de `main` después de cada integración.

## Comunicación de bloqueos

Comunicar en la Issue o PR:

- Qué está bloqueado.
- Qué decisión o cambio se necesita.
- Qué archivos, contratos o personas están afectados.
- Si el bloqueo impide continuar o solo afecta una parte.
- Qué trabajo independiente puede avanzar mientras tanto.

No dejar ramas abiertas sin contexto ni mantener cambios locales sin publicar cuando otro colaborador dependa de ellos.

## Incorporación de una persona nueva

Antes de asignar tareas de implementación:

1. Dar acceso al repositorio con el nivel mínimo de permisos necesario.
2. Pedir que lea `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, [arquitectura](../architecture/README.md), [estándares](../engineering/standards.md) y esta guía.
3. Explicar el estado actual de CI, las PRs abiertas y las limitaciones conocidas.
4. Empezar con una Issue pequeña, independiente y bien delimitada.
5. Revisar juntos el primer PR para validar el flujo, los controles y la coordinación.
6. Ampliar gradualmente el ownership de módulos cuando haya experiencia suficiente.

El archivo `.github/CODEOWNERS` identifica actualmente al mantenedor del repositorio como propietario global para sugerencias de revisión. Cuando se conozca el usuario GitHub del nuevo colaborador y sus áreas de responsabilidad, revisar CODEOWNERS para reflejar ownership real por rutas. No asignar ownership por anticipado ni usarlo como sustituto de la coordinación en Issues.

## Checklist antes de empezar una tarea

- [ ] Busqué trabajo duplicado en Issues y PRs.
- [ ] La tarea tiene responsable principal y criterios de aceptación.
- [ ] Identifiqué módulos, archivos y contratos afectados.
- [ ] Coordiné cualquier archivo o contrato compartido.
- [ ] Conozco las dependencias y el orden de integración.
- [ ] Creé una rama propia desde la base correcta.
- [ ] Leí las reglas de arquitectura y calidad aplicables.

## Checklist antes de cerrar una tarea

- [ ] El PR contiene solo cambios del alcance acordado.
- [ ] La documentación y los tests reflejan el comportamiento.
- [ ] Informé resultados reales de validación y limitaciones.
- [ ] Revisé los conflictos y cambios de otros colaboradores.
- [ ] La Issue y la reserva de archivos quedaron actualizadas.
- [ ] La integración respeta las restricciones actuales del proyecto.
