# Estándares de código

## Regla inquebrantable: sin comentarios en el código (obligatorio)

No se escriben comentarios en el código fuente. El código debe ser autodocumentado mediante:

- Nombres de tipos, funciones, variables, módulos y campos que expresen intención.
- Estructura clara: un archivo, una responsabilidad; archivos pequeños y enfocados.
- Tests que pinchen el comportamiento observable (documentan el contrato por ejemplo).
- Tipos que hagan imposibles los estados inválidos (ej. `Price` en lugar de `f64` crudo).

Excepciones permitidas (y solo estas):

- Documentación de APIs públicas (`///` en items públicos) que describa el contrato, no la implementación.
- Justificación de `unsafe` requerida por el guardrail de `unsafe`.

## Principios generales

- Un archivo, una responsabilidad.
- Un componente, un archivo.
- Archivos pequeños y enfocados; nunca código espagueti.
- Código legible sobre código ingenioso.

## Nombres

- Tipos, traits, módulos: `PascalCase`.
- Funciones, variables, campos: `snake_case`.
- Constantes: `SCREAMING_SNAKE_CASE`.
- Crates: `snake_case`, nombres cortos y descriptivos.

## Errores

- Usar `Result` y `Option` para hacer explícitos los errores y ausencias.
- Evitar `unwrap()` y `expect()` en caminos de producción normales.
- Evitar `panic!()` para errores recuperables.
- Usar `thiserror` para errores tipados con contexto.

## `unsafe`

- Solo con justificación técnica documentada.
- Encapsulado en módulos seguros con pruebas.
- Nunca en caminos de negocio cotidianos.

## Concurrencia

- No usar `Arc<Mutex<T>>` automáticamente para todo problema de concurrencia.
- Preferir tipos que hagan imposibles los estados inválidos.
- Medir contención antes de añadir locks.

## APIs públicas

- Pequeñas, explícitas y estables.
- Documentar invariantes y comportamiento.
- No exponer detalles internos de implementación.

## Clonados

- No clonar datos grandes sin entender el costo.
- Preferir préstamos (`&T`) sobre copias cuando sea posible.