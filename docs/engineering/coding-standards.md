# Estándares de código

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