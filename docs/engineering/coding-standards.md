# Estándares de código

## Regla inquebrantable: sin comentarios en el código (obligatorio)

No se escriben comentarios en el código fuente. El código tiene que autodocumentarse mediante:

- Nombres de tipos, funciones, variables, módulos y campos que expresen intención.
- Estructura clara: un archivo, una responsabilidad; archivos pequeños y enfocados.
- Tests que pinchen el comportamiento observable (documentan el contrato por ejemplo).
- Tipos que hagan imposibles los estados inválidos (ej. `Price` en lugar de `f64` crudo).

Excepciones permitidas (y solo estas):

- Documentación de APIs públicas (`///` en items públicos) que describa el contrato, no la implementación.
- Justificación de `unsafe` requerida por el guardrail de `unsafe`.

## Documentación pública obligatoria: `///` y `//!` (obligatorio)

Todo item público (struct, enum, trait, función, método, constante, módulo) **debe** tener documentación `///` o `//!` profesional, actualizada y precisa.

Requisitos:

- **`///` en items públicos:** describir el contrato, el propósito y el comportamiento observable, no la implementación interna. Incluir parámetros, retorno, errores, panics e invariantes cuando aplique.
- **`//!` en módulos:** describir el propósito del módulo, su responsabilidad, cómo encaja en la arquitectura y las reglas de uso.
- **Mantener actualizado:** si cambia el comportamiento, la firma o el propósito de un item público, actualizar su documentación en el mismo commit. Nunca dejar documentación desactualizada.
- **Profesional:** escribir en español argentino (voseo), claro y conciso. Sin jargon innecesario, sin comentarios de implementación, sin "TODO", "FIXME" o notas internas.
- **No documentar lo obvio:** si el nombre ya expresa la intención, la documentación puede ser breve, pero nunca ausente en items públicos.

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

- Usá `Result` y `Option` para hacer explícitos los errores y ausencias.
- Evité `unwrap()` y `expect()` en caminos de producción normales.
- Evité `panic!()` para errores recuperables.
- Usá `thiserror` para errores tipados con contexto.

## `unsafe`

- Solo con justificación técnica documentada.
- Encapsulado en módulos seguros con pruebas.
- Nunca en caminos de negocio cotidianos.

## Concurrencia

- No uses `Arc<Mutex<T>>` automáticamente para todo problema de concurrencia.
- Preferí tipos que hagan imposibles los estados inválidos.
- Medí la contención antes de agregar locks.

## APIs públicas

- Pequeñas, explícitas y estables.
- Documentá invariantes y comportamiento.
- No expongas detalles internos de implementación.

## Clonados

- No clones datos grandes sin entender el costo.
- Preferí préstamos (`&T`) sobre copias cuando sea posible.
