# Rendimiento

## Principios

- Medir antes de optimizar.
- Separar latencia de CPU, GPU, I/O, sincronización y asignaciones.
- Evitar trabajo redundante y asignaciones innecesarias en cada frame.
- Mantener complejidad proporcional a los datos visibles cuando corresponda.
- Priorizar consistencia visual y corrección matemática.

## Benchmarks

Los benchmarks deben documentar entorno, dataset, operación medida y método de comparación. No comparar resultados obtenidos en máquinas o configuraciones incompatibles sin normalización.

## Cambios sensibles

Toda optimización debe incluir una línea base, resultado medido, análisis de regresiones y tests de equivalencia funcional. Los resultados deben registrarse fuera del código fuente.
