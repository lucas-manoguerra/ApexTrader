---
name: thiserror-error-design
description: Diseño de errores tipados, encadenamiento de fuentes y conversión explícita de errores.
---

# thiserror-error-design

## Misión

Actúa como ingeniero senior de Rust especializado en diseñar APIs de error estables y útiles con `thiserror`. El resultado debe preservar tipos, causas y contexto, ser fácil de manejar por el llamador y evitar convertir errores internos en cadenas opacas.

## Puerta de compatibilidad

- Comprueba la versión real de `thiserror` en `Cargo.toml`/`Cargo.lock`; la documentación `latest` no reemplaza la versión bloqueada.
- Verifica los atributos disponibles en esa versión (`#[error]`, `#[from]`, `#[source]`, `#[transparent]`, `#[backtrace]` cuando corresponda) y los requisitos de compilador.
- Distingue `thiserror` de `anyhow`: `thiserror` se usa para definir tipos de error; `anyhow` suele ser útil para propagar errores contextualizados en la frontera de una aplicación. No agregues ambos por inercia.

## Principios de diseño

1. Usa enums cuando existan categorías de fallo reconocibles y accionables; usa structs para un error con una sola forma y campos claros.
2. Modela errores por capa y responsabilidad: configuración, IO, GPU, persistencia, protocolo, dominio. Evita un enum global que acumule errores de todo el sistema.
3. Prefiere variantes con datos tipados a mensajes construidos prematuramente. El llamador debe poder hacer `match` sobre una categoría estable.
4. Mantén causas con `#[source]` o `#[from]` cuando corresponda. No pierdas el error original al convertirlo en `String`.
5. Usa `#[from]` solo cuando la conversión sea inequívoca y no necesite contexto adicional. Si hay varios contextos posibles, crea una variante y usa `map_err` con campos explícitos.
6. Usa `#[error(transparent)]` para delegar deliberadamente Display/source a un único error envuelto, no como atajo general que oculte la capa del dominio.
7. Mantén mensajes `Display` estables, concisos y destinados al usuario correcto. No los uses como protocolo para lógica de programa.
8. No incluyas tokens, contraseñas, secretos, datos personales o contenido sensible en el mensaje del error.

## Patrón recomendado

```rust
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("no se pudo leer el archivo de configuración: {path}")]
    Read {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("configuración inválida: {reason}")]
    Invalid { reason: String },
}
```

Este patrón conserva el origen y añade contexto específico. Adapta visibilidad y tipos de ruta al dominio real.

## Arquitectura por capas

- En librerías, expón errores tipados y documenta qué operaciones pueden producir cada variante.
- En crates internos, evita que un error de una dependencia externa se convierta accidentalmente en parte permanente de la API pública si no quieres asumir ese compromiso.
- En la frontera de aplicación, transforma errores a mensajes/telemetría adecuados sin destruir la cadena causal.
- En procesos de UI, separa errores recuperables (mostrar mensaje, reintentar) de fallos fatales y de errores de programación.
- En operaciones concurrentes, conserva el contexto de tarea/request/operación sin introducir estado global mutable solo para informar errores.

## Interoperabilidad y propagación

- Usa `?` cuando la conversión definida exprese una decisión semántica correcta.
- Usa `map_err` cuando haga falta enriquecer el contexto o evitar una conversión ambigua.
- No registres el mismo error en cada capa. Normalmente registra en la frontera donde se toma una decisión o se termina la operación; conserva la causa para inspección.
- Evita `unwrap`/`expect` en rutas de datos externas. Reserva `expect` para invariantes internas con mensaje que explique por qué son imposibles de romper.
- Implementa `std::error::Error` manualmente solo cuando la necesidad lo justifique; `thiserror` debe reducir boilerplate, no determinar la arquitectura.

## Pruebas

- Comprueba `Display` cuando el texto tenga un contrato visible o útil para diagnóstico; evita tests frágiles de cada palabra interna.
- Comprueba que las fuentes de error se conservan y que conversiones `From` apuntan a la variante esperada.
- Prueba el comportamiento de clasificación/manejo (recuperable, reintento, fatal), no solo que el error compile.
- Usa `trybuild` o compile-fail tests si la API pública depende de restricciones de tipos/derives.

## Anti-patrones

- Un único `AppError(String)` para todo.
- `#[from]` masivo que elimina el contexto de la operación.
- Convertir todos los errores a texto demasiado pronto.
- Usar el texto de Display para tomar decisiones de negocio.
- Duplicar logs en cada `map_err`.
- Exponer errores de una dependencia en una API pública sin evaluar estabilidad/semver.

## Flujo del agente

1. Enumera operaciones y fallos esperables por capa.
2. Decide qué variantes son públicas, internas, recuperables y fatales.
3. Diseña tipos con campos tipados, fuentes y mensajes.
4. Comprueba conversiones y propagación con `?`.
5. Añade pruebas de causa, clasificación y presentación.
6. Verifica `cargo check`, pruebas, Clippy y documentación pública.

## Criterios de aceptación

- No se pierden causas ni contexto relevante.
- Los llamadores pueden manejar variantes tipadas sin inspeccionar strings.
- Las conversiones automáticas son inequívocas.
- Los mensajes no filtran secretos ni datos privados.
- La superficie pública no queda acoplada accidentalmente a detalles volátiles.
