# ADR-0004: Modelo temporal y biblioteca de fechas

## Estado

Propuesta

## Contexto

ApexTrader procesa datos de mercado en una aplicación desktop multiplataforma (Windows, macOS y Linux). El tiempo interviene en dominios con semánticas distintas:

- Identificación y orden de eventos de mercado.
- Inicio, cierre y duración de velas.
- Conversión entre instantes UTC y fechas/horas civiles.
- Calendarios, sesiones, feriados y reglas horarias de mercados.
- Presentación de etiquetas temporales.
- Medición de latencias, FPS, timeouts y planificación de tareas locales.

El código actual usa milisegundos Unix en `i64` para `TimePoint` y `TimeScale`, `SystemTime` para obtener el reloj civil del sistema y `Instant` para programar actualizaciones realtime. `TimeScale` implementa manualmente conversión de días a calendario gregoriano y formateo UTC. La clasificación de intervalos mensuales/anuales usa aproximaciones de 28 y 365 días, que no representan meses ni años civiles reales.

El contrato de mercado ya establece que las duraciones fijas y los períodos de calendario no son intercambiables, y que sesiones y gaps deben evaluarse con el calendario/venue correspondiente.

## Alternativas consideradas

### 1. Mantener exclusivamente la biblioteca estándar

`SystemTime`, `Instant` y enteros de epoch son suficientes para obtener el reloj, medir duraciones monotónicas y ordenar instantes. No proporcionan por sí solos operaciones de calendario gregoriano, zonas horarias IANA, transiciones DST ni resolución de horas civiles ambiguas.

Esta opción evita dependencias, pero obliga a mantener y verificar lógica calendárica propia a medida que se incorporen sesiones y calendarios de mercado.

### 2. Incorporar `time`

La crate `time` ofrece tipos explícitos para fechas, horas, offsets, parsing y formatting, con features optativas. La integración con bases de zonas horarias IANA requiere componentes externos; `time-tz` es una alternativa del ecosistema. Esto implica ownership y actualización adicionales de la base horaria.

### 3. Incorporar `chrono`

Chrono ofrece `DateTime`, tipos civiles sin zona, offsets, parsing y formatting. La base de zonas horarias IANA completa requiere una dependencia como `chrono-tz`; el soporte de zona local y sus features también deben evaluarse en cada plataforma. No resuelve por sí solo el modelo de calendario/venue de los mercados.

### 4. Incorporar `jiff`

Jiff separa instantes absolutos (`Timestamp`), fechas/horas civiles y valores zonificados. Proporciona aritmética consciente de zonas horarias y DST, resolución explícita de tiempos civiles ambiguos e integración con la base IANA del sistema. En Windows puede incorporar una copia de la base IANA al binario; en Unix suele utilizar la base instalada del sistema.

Jiff prioriza APIs difíciles de usar incorrectamente y rendimiento razonable, no el máximo rendimiento en todos los escenarios. La versión 1.0 es reciente y su ecosistema es menos antiguo que el de Chrono o time; debe considerarse el coste de adopción, compatibilidad y mantenimiento.

## Decisión propuesta

Adoptar un modelo temporal híbrido y **evaluar Jiff como biblioteca de calendario y zonas horarias**, sin incorporarla todavía como dependencia hasta implementar el primer caso real que la necesite y validar la matriz multiplataforma.

1. Los timestamps de mercado conservan una representación canónica de epoch con unidad explícita. En el estado actual se mantiene `i64` en milisegundos Unix para compatibilidad con proveedores y estructuras del chart.
2. Las operaciones de orden, comparación, búsqueda, indexación e interpolación de la escala temporal operan sobre enteros y no convierten timestamps a objetos de fecha.
3. La conversión a `jiff::Timestamp` se realiza en los límites donde se necesitan calendario, zona horaria, parsing o formatting.
4. El tiempo de evento del mercado, el tiempo de recepción local y el tiempo de procesamiento/publicación deben seguir siendo campos semánticamente distintos cuando el contrato de ingestión los incorpore.
5. `std::time::Instant` sigue siendo la opción para medir duraciones monotónicas, latencia, FPS, timeouts y scheduling. No se reemplaza por un reloj civil.
6. `std::time::SystemTime` puede seguir utilizándose para obtener el instante del reloj del sistema, convirtiéndolo explícitamente a epoch cuando sea necesario.
7. Los calendarios y sesiones de mercado deben modelarse por instrumento/venue y versión de calendario. Una zona horaria IANA es un dato necesario, pero no sustituye las reglas de sesión, feriados, cierres extraordinarios ni cambios históricos del mercado.
8. Las etiquetas de ejes y presentación deben usar una política explícita de zona horaria y locale. UTC puede ser el default técnico inicial, pero no debe asumirse como la zona de visualización universal.
9. No se permite construir calendarios mediante constantes aproximadas de milisegundos por mes o año. Los intervalos de calendario se calculan con tipos de calendario.
10. No se introducen conversiones de fecha/hora en el hot path de renderizado ni por vela/frame si una etiqueta puede calcularse una vez y cachearse según sus dependencias.

## Consecuencias

### Positivas

- Conserva operaciones temporales de mercado rápidas y deterministas basadas en enteros.
- Evita mezclar tiempo monotónico con reloj civil.
- Prepara soporte correcto de sesiones, zonas horarias y DST en los tres sistemas operativos.
- Evita mantener algoritmos calendáricos manuales en el dominio y presentación.
- La integración de calendario queda en los límites que realmente la necesitan.

### Costes y riesgos

- Jiff es una dependencia nueva y relativamente reciente; se deben validar MSRV, dependencias transitivas, licencia, seguridad, tamaño del binario y compatibilidad con Windows/macOS/Linux antes de adoptarla.
- El bundle de tzdb puede incrementar el tamaño del ejecutable en Windows y requiere actualizar la dependencia para incorporar cambios de la base horaria.
- Las zonas horarias y calendarios de mercado requieren versionado y trazabilidad; la biblioteca no aporta reglas comerciales de cada venue.
- El modelo actual en milisegundos puede no representar proveedores con precisión inferior al milisegundo; la unidad y precisión canónica deben revisarse al incorporar proveedores reales.
- La política de visualización necesita una zona horaria configurable y pruebas de cambios DST.

## Plan de validación antes de añadir la dependencia

- Probar conversión epoch-millis a/desde `jiff::Timestamp`, incluidos valores negativos y límites representables.
- Probar fechas civiles ambiguas e inexistentes durante transiciones DST.
- Probar sesiones que cruzan medianoche, fines de semana, feriados y cierres extraordinarios con calendarios de mercado de referencia.
- Verificar el comportamiento de detección de zona horaria y disponibilidad de tzdb en Windows, macOS y Linux.
- Medir impacto en tiempo de compilación y tamaño de binario.
- Verificar MSRV, dependencias transitivas y auditoría de seguridad.
- Comparar formatting de etiquetas frente a los tests actuales y eliminar conversiones por frame.
- Añadir benchmarks solo para rutas que vayan a ejecutarse con frecuencia; no asumir que la biblioteca de fechas debe intervenir en el indexado temporal.

## Auditoría de implementación

La revisión estática de los tipos y consumidores actuales se registra en [temporal-model-review.md](../audit/temporal-model-review.md). Identifica seis hallazgos sobre unidad implícita, semántica de intervalos, umbrales de calendario aproximados, UTC fijo, ausencia de calendario de mercado y normalización de fronteras de proveedor. La auditoría no añade una biblioteca ni cambia el modelo de ejecución.

## Condiciones de revisión

Revisar la selección si Jiff no satisface los requisitos de MSRV, tamaño, licencia, seguridad, actualización de tzdb o compatibilidad multiplataforma; comparar entonces `time` más su solución de zonas horarias y `chrono` más `chrono-tz` con los mismos casos de prueba.

## Referencias

- [Jiff — documentación](https://docs.rs/jiff/latest/jiff/)
- [Jiff — soporte de plataformas y zonas horarias](https://docs.rs/jiff/latest/jiff/_documentation/platform/index.html)
- [Jiff — comparación con otras bibliotecas](https://docs.rs/jiff/latest/jiff/_documentation/comparison/index.html)
- [time — documentación](https://docs.rs/time/latest/time/)
- [Chrono — documentación](https://docs.rs/chrono/latest/chrono/)
