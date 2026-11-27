# Flujo de desarrollo

## Antes de implementar

1. Consultar tareas y PRs abiertas para evitar duplicar trabajo.
2. Leer `AGENTS.md` y los documentos de las áreas afectadas.
3. Acordar objetivo, criterios de aceptación, responsable y alcance.
4. Inspeccionar implementación, consumidores, pruebas y límites de dependencias.
5. Identificar invariantes, fallos posibles y efectos de rendimiento.

## Durante la implementación

- Mantener cambios pequeños y cohesivos.
- Evitar refactors no relacionados.
- No modificar contratos compartidos sin coordinación con sus consumidores y responsables.
- Actualizar tests y documentación en el mismo cambio.
- Registrar decisiones de arquitectura relevantes.
- Mantener la política de cero comentarios en código.
- Mantener commits enfocados y no incluir cambios ajenos.

## Antes de solicitar revisión

Ejecutar formato, check, Clippy, tests y los controles estáticos aplicables. Informar claramente controles no ejecutados y motivos. Revisar el diff completo, los archivos modificados y la base de la rama.

## Pull Request

El PR debe incluir objetivo, alcance, decisiones, pruebas, riesgos, impacto en rendimiento y documentación afectada. Los cambios que alteren contratos deben identificar consumidores, colaboradores afectados y plan de migración. Si depende de otra PR, debe declararlo y enlazarla.

## Integración

La integración respeta las reglas vigentes de CI y la secuencia de dependencias. Una PR verde no autoriza por sí sola un merge si existe una restricción temporal del proyecto. No se debe eliminar una matriz de plataforma para simular cobertura completa.
