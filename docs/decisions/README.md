# Decisiones de arquitectura (ADR)

Esta carpeta contiene Architecture Decision Records (ADR).

Un ADR registra una decisión importante:

- Qué problema teníamos.
- Qué alternativas evaluamos.
- Qué decisión tomamos.
- Por qué la tomamos.
- Qué consecuencias tendrá.

## Propósito

Para no tener que discutir dentro de un año decisiones que ya analizamos.

## Cómo agregar un ADR

1. Creá un archivo `docs/decisions/NNNN-titulo-corto.md` (NNNN es un número secuencial).
2. Incluí: contexto, alternativas, decisión, justificación, consecuencias.
3. Referenciá el ADR desde el código o la documentación relacionada cuando sea útil.

## Ejemplo de decisión futura

Cuando decidamos usar `egui` para la interfaz general y `wgpu` para renderizado especializado, documentaremos los motivos y alternativas consideradas aquí.

## ADRs actuales

Ninguno todavía. Esta carpeta queda lista para la primera decisión.
