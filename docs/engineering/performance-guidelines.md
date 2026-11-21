# Directrices de rendimiento

## Filosofía

**No optimizaremos por intuición. Mediremos, identificaremos el cuello de botella y optimizaremos.**

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

## Ejemplo

Si queremos representar cientos de miles de velas, no redibujamos innecesariamente toda la geometría cada vez que se mueve el cursor. Eso se resuelve con arquitectura, estructuras de datos y estrategias de renderizado, no solo escribiendo código más rápido.

## Cuándo medir

- Antes de cualquier cambio que afirme mejorar el rendimiento.
- Cuando un usuario reporta lentitud.
- Cuando se añade una nueva fuente de datos o indicador.