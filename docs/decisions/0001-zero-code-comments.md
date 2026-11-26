# ADR-0001: Prohibir comentarios en el código

## Estado

Aceptada

## Contexto

Los comentarios de implementación tienden a duplicar el comportamiento y pueden quedar obsoletos. La documentación de API y las decisiones necesitan un lugar estable, revisable y navegable fuera del código fuente.

## Alternativas consideradas

- Mantener comentarios de implementación con revisión manual.
- Permitir únicamente Rustdoc.
- Prohibir comentarios en el código mantenido y centralizar explicaciones en documentación externa.

## Decisión

No se permiten comentarios de línea, bloque, Rustdoc, TODO/FIXME ni código comentado en los archivos de código y configuración ejecutable mantenidos por el proyecto. La explicación de arquitectura, contratos, procedimientos y decisiones vive en Markdown.

Los artefactos generados y las dependencias externas o vendorizadas se tratan como material de terceros y no se modifican automáticamente.

## Consecuencias

- El código debe ser autodocumentado mediante nombres, tipos, módulos e interfaces.
- La documentación externa debe mantenerse alineada con la implementación.
- CI debe impedir que se agreguen comentarios en los cambios y, una vez completada la limpieza inicial, verificar el repositorio completo.
- La retirada de Rustdoc requiere conservar en documentación externa la información de diseño que siga siendo necesaria.

## Condiciones de revisión

Revisar esta decisión si cambia el alcance de los archivos mantenidos, se incorpora código generado o se demuestra una necesidad técnica que no pueda resolverse mediante tipos y documentación externa.
