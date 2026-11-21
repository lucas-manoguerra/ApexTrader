# Directrices de rendimiento

## Filosofía

**No optimizaremos por intuición. Mediremos, identificamos el cuello de botella y optimizamos.**

## Tres principios

1. No introducir optimizaciones sin evidencia.
2. No sacrificar claridad por microoptimizaciones irrelevantes.
3. No permitir que una decisión de arquitectura impida alcanzar los objetivos de rendimiento.

## Áreas de vigilancia

| Área | Qué vigilamos |
|------|---------------|
| CPU | Costo de cálculos y procesamiento |
| RAM | Asignaciones, copias y estructuras de datos |
| GPU | Transferencias, buffers, shaders y draw calls |
| Concurrencia | Contención, bloqueos y sincronización |
| Market Data | Throughput, latencia y pérdida de eventos |
| Chart Engine | Cantidad de datos y geometría procesada |
| Indicadores | Cálculo incremental y reutilización de resultados |

## Benchmarks y línea base

Existe un benchmark de línea base en `benches/chart_transform_benchmark.rs` que mide el costo de transformar coordenadas de datos a pantalla para N velas.

- Ejecutar con: `cargo bench --bench chart_transform_benchmark`
- Este benchmark es la **evidencia de referencia**. Cualquier cambio en `domain/viewport.rs` o `renderer/` debe mantener o mejorar estos números.
- Si un cambio degrada el tiempo por vela, no se integra hasta entender por qué y corregirlo.

## Profiling

- Usar `cargo flamegraph` o el profiler de Windows (PerfView / Visual Studio Profiler) para identificar puntos calientes reales.
- El profiling se ejecuta sobre builds de release (`--release`) para que los números sean representativos.
- Nunca perfilar en debug: el código no optimizado distorsiona los resultados.

## Ejemplo

Si queremos representar cientos de miles de velas, no redibujamos innecesariamente toda la geometría cada vez que se mueve el cursor. Eso se resuelve con arquitectura, estructuras de datos y estrategias de renderizado, no solo escribiendo código más rápido.

## Cuándo medir

- Antes de cualquier cambio que afirme mejorar el rendimiento.
- Cuando un usuario reporta lentitud.
- Cuando se añade una nueva fuente de datos o indicador.
- Antes y después de cambiar el backend de renderizado (glow → wgpu).