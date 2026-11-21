---
name: tracing-instrumentation
description: Instrumentación estructurada con spans, events, campos y propagación contextual async.
---

# tracing-instrumentation

## Misión

Actúa como ingeniero senior de diagnóstico Rust especializado en `tracing`. Instrumenta el sistema con spans y events estructurados que permitan comprender causalidad, latencia y estado sin crear ruido, overhead innecesario o filtraciones de datos.

## Puerta de compatibilidad

- Comprueba versión y features de `tracing` en Cargo.lock. La versión observada es 0.1.44.
- Verifica que la feature `attributes` está habilitada antes de usar `#[instrument]`; comprueba features para `log` u otras integraciones.
- Revisa API de macros, fields, `Span`, `Event`, `Instrument` y `instrument` en la versión efectiva.
- Recuerda que `tracing` emite información; no configura por sí mismo un destino. La aplicación necesita un subscriber (normalmente `tracing-subscriber`).

## Span frente a Event

- Un `Span` representa una unidad de trabajo con duración y contexto: request, conexión, inicialización GPU, carga de workspace, frame o transacción.
- Un `Event` representa algo que ocurre en un instante: operación completada, valor actualizado, reintento, fallo o cambio de estado.
- Anida spans para reflejar causalidad y fases. No crees spans por cada instrucción o elemento de una colección.
- Usa campos tipados (`user_id` solo si es seguro, `request_id`, `operation`, `duration_ms`, `result`) con nombres estables.

## Diseño de instrumentación

- Instrumenta fronteras importantes: APIs públicas, operaciones I/O, tareas async, procesamiento de datos, pipeline GPU, lifecycle de ventana y comandos de aplicación.
- Define una convención de targets y nombres. Evita generar nombres de span dinámicos con cardinalidad ilimitada.
- Usa `#[instrument]` en funciones donde el span automático aporta contexto. Ajusta nombre/level y omite campos grandes o sensibles con `skip`/atributos equivalentes verificados.
- No registres buffers completos, imágenes, estructuras gigantes ni contraseñas. Registra tamaños, IDs no sensibles, contadores y estados.
- Para campos que se conocen después de crear un span, utiliza recording APIs si la versión lo permite; declara el campo con la semántica correcta.
- Prefiere campos estructurados sobre mensajes formateados manualmente. Mantén los mensajes para contexto legible, no como único dato consultable.

## Async y causalidad

- En funciones async, asegura que el span acompaña la ejecución del future a través de los awaits. `#[instrument]` está diseñado para crear y entrar al span asociado a la llamada; confirma el comportamiento de macros/trait en versión concreta.
- Cuando construyas futures manualmente o uses combinadores, usa el mecanismo `Instrument`/`Future::instrument` compatible para adjuntar el span.
- No uses `Span::enter` manteniendo un guard a través de `.await` si eso puede asociar eventos de otras tareas al contexto equivocado. Sigue las advertencias de la documentación.
- Propaga contexto explícitamente a threads/tareas cuando el flujo no lo preserve automáticamente.

## Niveles y volumen

- `ERROR`: operación fallida que requiere atención.
- `WARN`: degradación o situación anómala recuperable.
- `INFO`: transiciones de alto nivel y eventos operativos relevantes.
- `DEBUG`: diagnóstico accionable.
- `TRACE`: detalle intensivo que debe quedar desactivado normalmente.
- No conviertas cada frame o tick en `INFO`; agrega métricas/resúmenes o usa `TRACE` con muestreo cuando corresponda.
- Evita logs duplicados al propagar un mismo error por varias capas.

## Rendimiento y seguridad

- Comprueba si un span/event está habilitado cuando construir su campo cuesta mucho, usando APIs/macros apropiadas.
- No construyas strings costosos anticipadamente si el evento puede estar filtrado.
- Mantén baja cardinalidad para campos usados como dimensiones en sistemas de observabilidad.
- No registres tokens, credenciales, secretos, datos de usuario sensibles ni payloads privados.
- No uses `tracing` como mecanismo de control del flujo del programa.
- Las librerías deben emitir spans/events y no instalar subscribers globales.

## Convenciones útiles en aplicaciones GUI/GPU

- Spans: `app.startup`, `window.create`, `gpu.adapter.request`, `gpu.device.request`, `renderer.frame`, `text.prepare`, `workspace.load`, `market_data.update`.
- Events: resize, device lost, surface timeout, recurso recreado, frame degradado, tarea completada.
- Registra duración, tamaño de batch y estado de recurso, pero no instrumentes indiscriminadamente cada draw call si el costo o volumen es alto.
- Asocia el ID de ventana/sesión como campo cuando no sea sensible.

## Pruebas

- Usa subscribers de captura/scoped en tests, no inicialices globalmente uno distinto en cada test paralelo.
- Verifica que se emitan spans/eventos significativos y campos importantes.
- Comprueba que los campos sensibles quedan omitidos.
- Mide el overhead con subscriber activo e inactivo en rutas calientes.

## Anti-patrones

- Usar `info!` como `println!` para cada paso.
- Crear un span por elemento sin necesidad.
- Mantener guard de `enter()` a través de `.await`.
- Instrumentar argumentos grandes o sensibles automáticamente.
- Instalar el subscriber dentro de una librería.
- Construir mensajes/strings costosos aunque el nivel esté deshabilitado.

## Flujo del agente

1. Dibuja el flujo causal y elige spans por unidad de trabajo.
2. Elige events para hitos puntuales.
3. Define campos estables y política de datos sensibles.
4. Aplica macros/Instrument con APIs verificadas.
5. Comprueba propagación en async y tareas paralelas.
6. Añade pruebas de captura y mide overhead.
7. Documenta cómo activar niveles y filtros con el subscriber del binario.

## Criterios de aceptación

- La traza muestra causalidad y duración, no solo una secuencia plana de mensajes.
- Async conserva contexto de forma correcta.
- Campos son estructurados, estables y seguros.
- El volumen está controlado y no penaliza rutas calientes sin necesidad.
- El crate de librería no configura el subscriber global.
