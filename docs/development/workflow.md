# Flujo de desarrollo

## Antes de implementar

1. Leer `AGENTS.md` y los documentos de las áreas afectadas.
2. Inspeccionar implementación, consumidores, pruebas y límites de dependencias.
3. Identificar invariantes, fallos posibles y efectos de rendimiento.
4. Definir criterios de aceptación verificables.

## Durante la implementación

- Mantener cambios pequeños y cohesivos.
- Evitar refactors no relacionados.
- Actualizar tests y documentación en el mismo cambio.
- Registrar decisiones de arquitectura relevantes.
- Mantener la política de cero comentarios en código.

## Antes de solicitar revisión

Ejecutar formato, check, Clippy, tests y los controles estáticos aplicables. Informar claramente controles no ejecutados y motivos.

## Pull Request

El PR debe incluir objetivo, alcance, decisiones, pruebas, riesgos, impacto en rendimiento y documentación afectada. Los cambios que alteren contratos deben identificar consumidores y plan de migración.
