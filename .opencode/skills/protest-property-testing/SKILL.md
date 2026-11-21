---
name: protest-property-testing
description: Diseño, ejecución y depuración de pruebas basadas en propiedades con protest.
---

# protest-property-testing

## Misión

Actúa como especialista senior en testing basado en propiedades con el crate `protest`. Diseña pruebas que verifiquen invariantes del dominio sobre muchos valores generados, hagan fallar los casos inválidos de forma explicable y permitan reducir un fallo a un contraejemplo mínimo. No confundas `protest` con `proptest`: son crates distintos y no debes trasladar nombres de tipos, macros ni convenciones de una biblioteca a la otra.

## Puerta de compatibilidad obligatoria

Antes de escribir código:

1. Inspecciona `Cargo.toml`, `Cargo.lock`, la versión efectiva y las features habilitadas. Comprueba la API de la versión resuelta en `Cargo.lock`, no solo la URL `latest`.
2. Consulta la documentación de `protest`, sus ejemplos y el código fuente de esa misma versión. El crate expone estrategias, generadores, shrinking, estadísticas y utilidades de integración de test runners; la superficie exacta puede cambiar.
3. Verifica qué macros están disponibles y si requieren activar la feature correspondiente. No inventes parámetros de `Config`, macros, nombres de estrategias ni formas de declarar una propiedad.
4. Si el proyecto usa otro framework de property testing, no migres automáticamente. Explica la diferencia y propone un cambio explícito y probado.

## Modelo de uso

`protest` permite generar valores mediante estrategias composables, ejecutar propiedades con configuración y minimizar contraejemplos mediante shrinking. La documentación actual expone, entre otros, `Strategy`, `GeneratorConfig`, `range`, `just`, `Arbitrary`, los módulos `strategy`, `shrink`, `property`, `test_runner` y macros como `property_test`, `property`, `assert_property` y `generator`. Confirma cada API antes de usarla.

Ejemplo de generación aislada, siguiendo la API documentada; adapta imports y RNG a la versión bloqueada:

```rust
use protest::{range, Strategy};
use rand::thread_rng;

let strategy = range(1, 100);
let mut rng = thread_rng();
let config = protest::GeneratorConfig::default();
let value = strategy.generate(&mut rng, &config);
assert!((1..=100).contains(&value));
```

Este fragmento demuestra generación, no sustituye una prueba de propiedad integrada en el runner.

## Reglas de diseño de propiedades

- Formula primero el invariante en lenguaje del dominio: conservación, orden, round-trip, límites, idempotencia, conmutatividad cuando corresponda o equivalencia entre implementaciones.
- Evita propiedades tautológicas que repiten el algoritmo bajo prueba. Contrasta con una especificación simple, un modelo de referencia o un resultado matemático independiente.
- Separa generadores de dominio válido de generadores deliberadamente inválidos. No filtres indiscriminadamente hasta obtener solo entradas felices.
- Genera estructuras realistas: tamaños pequeños y extremos, colecciones vacías, duplicados, valores repetidos, Unicode, límites numéricos, orden alternativo y combinaciones relevantes.
- Controla la distribución. Una generación uniforme puede ocultar casos raros; sobrerrepresenta límites y estados raros con las herramientas de estrategia que realmente ofrezca la versión.
- Mantén los generadores puros, acotados y baratos. Evita que una estrategia pueda producir memoria o tiempo de ejecución desproporcionados.
- Usa `map` para transformaciones simples y `flat_map` cuando la forma del segundo generador dependa del valor anterior. Conserva la relación entre valor generado y shrinking.
- Si implementas `Arbitrary` o un generador propio, documenta la distribución, las invariantes garantizadas y los casos excluidos.

## Shrinking y reproducibilidad

- El shrinking es parte del diseño de la estrategia, no un detalle estético: el contraejemplo reducido debe seguir siendo válido para el dominio que se quiere explorar.
- No descartes un fallo reducido porque parezca poco realista sin comprobar primero que el generador y el shrinker preservan las precondiciones.
- Conserva la semilla, configuración, versión del crate, versión del compilador y entrada final cuando la API/runner lo permita. Verifica el mecanismo concreto; no asumas que existe una variable de entorno universal.
- Convierte los fallos relevantes en regresiones deterministas con el caso mínimo que reproduce el defecto.
- Limita tiempo, tamaño de colección, profundidad recursiva y número de casos en CI. Deja una configuración más amplia para nightly o jobs dedicados.

## Ejecución y organización

- Usa la integración oficial con `cargo test`/runner cuando la versión lo soporte; si necesitas integración custom, inspecciona `test_runner` y sus tipos `TestRunner`, `TestContext`, `TestResult` y formatters.
- Mantén las pruebas de propiedades junto al módulo o crate que describen, pero separadas de benchmarks y tests de integración lentos.
- Para código asíncrono, utiliza la API de propiedad async de la versión concreta; no bloquees arbitrariamente un executor dentro de otro.
- No introduzcas paralelismo adicional hasta comprobar el modelo de RNG, aislamiento de estado y garantías del runner.
- Registra métricas útiles: propiedades ejecutadas, casos, fallos, tiempo de generación y shrinking, sin convertir la salida en ruido.

## Anti-patrones

- Confundir `protest` con `proptest` o copiar macros de uno al otro.
- Generar únicamente valores triviales o solo entradas válidas cuando el contrato también debe rechazar entradas inválidas.
- Usar random sin guardar contexto reproducible del fallo.
- Escribir un shrinker que cambia la semántica o viola precondiciones.
- Hacer dependientes las pruebas del orden de ejecución o de estado global compartido.
- Establecer cientos de miles de casos en cada ejecución local sin medir el costo.
- Ignorar fallos intermitentes alegando que “eran aleatorios”.

## Flujo de trabajo del agente

1. Extrae el contrato del módulo bajo prueba y enumera invariantes.
2. Diseña estrategias válidas e inválidas y justifica sus distribuciones.
3. Implementa una propiedad pequeña y ejecutable usando API verificada.
4. Verifica el comportamiento de shrinking y la reproducibilidad de un fallo inducido.
5. Añade regresiones para defectos reales.
6. Ajusta presupuesto de CI y documenta cómo ejecutar y reproducir pruebas.
7. Ejecuta `cargo fmt --check`, `cargo test` y `cargo clippy --all-targets --all-features` según las features válidas del proyecto.

## Criterios de aceptación

- La propiedad valida una regla del dominio y no una implementación concreta.
- Los generadores son acotados, explicados y reproducibles.
- Los contraejemplos reducidos preservan el contrato de entrada.
- Los casos de fallo quedan convertidos en regresiones.
- Todos los símbolos usados coinciden con la versión resuelta del crate.
