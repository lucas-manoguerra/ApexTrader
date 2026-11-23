# Directrices de rendimiento

## Filosofía

**No optimices por intuición. Medí, identificá el cuello de botella y optimizá.**

## Tres principios

1. No introduzcas optimizaciones sin evidencia.
2. No sacrifiques claridad por microoptimizaciones irrelevantes.
3. No permitas que una decisión de arquitectura impida alcanzar los objetivos de rendimiento.

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

Hay un benchmark de línea base en `benches/chart_transform_benchmark.rs` que mide el costo de transformar coordenadas de datos a pantalla para N velas.

- Ejecutá con: `cargo bench --bench chart_transform_benchmark`
- Este benchmark es la **evidencia de referencia**. Cualquier cambio en `domain/viewport.rs` o `renderer/` tiene que mantener o mejorar estos números.
- Si un cambio degrada el tiempo por vela, no se integra hasta entender por qué y corregirlo.

## Profiling

- Usá `cargo flamegraph` o el profiler de Windows (PerfView / Visual Studio Profiler) para identificar puntos calientes reales.
- El profiling se ejecuta sobre builds de release (`--release`) para que los números sean representativos.
- No perfiles en debug: el código no optimizado distorsiona los resultados.

## Ejemplo

Si querés representar cientos de miles de velas, no redibujees innecesariamente toda la geometría cada vez que se mueve el cursor. Eso se resuelve con arquitectura, estructuras de datos y estrategias de renderizado, no solo escribiendo código más rápido.

## Cuándo medir

- Antes de cualquier cambio que afirme mejorar el rendimiento.
- Cuando un usuario reporta lentitud.
- Cuando se añade una nueva fuente de datos o indicador.
- Antes y después de cambiar el backend de renderizado (glow → wgpu).
