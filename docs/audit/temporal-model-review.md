# Revisión del modelo temporal

## Alcance y estado

Revisión estática del modelo temporal actualizada en el head `0742e57e5d5c644c4303b12413110a9a0d48a7cf`. Evalúa representación de instantes, intervalos, calendario, presentación y responsabilidades de las bibliotecas de fechas. No incorpora una dependencia ni cambia el comportamiento temporal.

La decisión de biblioteca sigue abierta según ADR-0004. Jiff es la candidata propuesta para evaluación; `time` y Chrono/`chrono-tz` permanecen como alternativas comparables. La selección no debe basarse solo en popularidad o microbenchmarks.

## Modelo actual observado

- Los timestamps de mercado y gráfico son `i64` en milisegundos Unix.
- `TimePoint` encapsula apertura, cierre e intervalo nominal; sus campos son privados y se consultan mediante getters.
- `MarketCandle` conserva apertura y cierre como `i64` públicos. La conversión calcula el span con `checked_sub` y rechaza duraciones no positivas.
- `TimeScale` mantiene timestamps ordenados, deduplicados y enteros; la indexación e interpolación no requieren objetos de fecha.
- El formateo de etiquetas convierte epoch a calendario gregoriano mediante una implementación propia de `civil_from_days` y presenta UTC explícitamente.
- La selección de granularidad de etiquetas usa umbrales de 28 días por mes y 365 días por año.
- El modelo no incluye una zona horaria de visualización configurable ni identidad de calendario/venue.

## Hallazgos

### TM-001 — La unidad temporal está implícita en enteros sin tipo semántico

**Severidad:** alta.

**Evidencia:** `TimePoint`, `MarketCandle`, `TimeScale` y contratos de proveedor utilizan `i64` con sufijo de campo `_ms`, pero no un tipo que impida mezclar milisegundos con segundos, microsegundos u otra escala.

**Riesgo:** errores de escala pueden compilar y producir fechas válidas pero incorrectas. El límite del proveedor tampoco expresa precisión inferior al milisegundo.

**Dirección:** definir un tipo de instante canónico con unidad documentada y conversiones explícitas checked. Antes de introducirlo, inventariar precisión y rango de los proveedores objetivo. No migrar todos los campos a una biblioteca de calendario: el núcleo de orden/indexación puede continuar en enteros.

### TM-002 — El intervalo nominal y el span real no tienen contrato uniforme

**Severidad:** alta.

**Evidencia:** `TimePoint` almacena `interval_ms` separado de `close_time_ms - open_time_ms`. El constructor permite que sean distintos; la conversión de `MarketCandle` establece el intervalo igual al span; el mock realtime deriva su intervalo de la última vela.

**Riesgo:** una vela de timeframe nominal de un minuto puede tener un span de timestamps que depende de la convención del proveedor (cierre inclusivo/exclusivo), y no necesariamente representa la duración nominal. Reutilizar ese span como paso de scheduling puede desplazar la vela siguiente o producir gaps/overlaps.

**Dirección:** modelar por separado el timeframe nominal (duración fija o período de calendario), los timestamps de inicio/fin del evento y la convención de cierre. Especificar si los intervalos son [inicio, fin), [inicio, fin] u otra semántica por proveedor. No inferir el próximo inicio sumando el span de una vela hasta que el contrato lo autorice.

### TM-003 — La clasificación de etiquetas aproxima meses y años con duraciones fijas

**Severidad:** media.

**Evidencia:** `TimeScale::format_time_axis_label` utiliza `28 * DAY_MS` y `365 * DAY_MS` para decidir cuándo mostrar mes/año.

**Riesgo:** los meses tienen longitudes civiles distintas y los años pueden ser bisiestos. Aunque los valores se usan como umbrales visuales y no como conversión de timestamps, el comportamiento no expresa una granularidad de calendario fiable.

**Dirección:** separar la selección de ticks (granularidad deseada) del formateo civil. La selección de meses/años debe utilizar unidades de calendario, no una duración aproximada. Mantenerla fuera del hot path y cachear etiquetas cuando sus dependencias no cambien.

### TM-004 — La conversión de calendario está implementada manualmente y UTC es la única política visible

**Severidad:** media-alta para una aplicación multiplataforma de mercados globales.

**Evidencia:** `TimeScale` implementa `civil_from_days` y añade el sufijo UTC. No existe zona horaria configurable, ni resolución de horas locales ambiguas/inexistentes, ni pruebas de DST.

**Riesgo:** los usuarios no pueden interpretar sesiones y horarios en la zona del mercado o en una zona elegida. La conversión manual se convierte en código de calendario que el proyecto debe mantener, mientras que UTC fijo no cubre la presentación local o por venue.

**Dirección:** conservar UTC como representación canónica de instantes; introducir un servicio de calendario/presentación en el límite de UI con zona IANA explícita y política de locale. Probar horas inexistentes y duplicadas en DST. No convertir a hora local dentro del dominio ni en cada vértice/frame.

### TM-005 — No existe calendario de mercado asociado a la serie

**Severidad:** alta antes de soportar instrumentos con sesiones.

**Evidencia:** el contrato de proveedor y `MarketCandle` no incluyen venue, timeframe, calendario ni zona horaria de sesión. El contrato canónico ya requiere estos metadatos, pero el modelo ejecutable aún no los representa.

**Riesgo:** agregar velas por día/semana/mes mediante duraciones fijas ignora sesiones, feriados, cierres extraordinarios, cambios históricos y mercados 24/7. La zona IANA sola tampoco define las reglas comerciales.

**Dirección:** incorporar una identidad de serie que refiera a instrumento, venue, timeframe y calendario/versionado. Resolver sesiones mediante datos autoritativos del venue. La biblioteca de fechas debe resolver calendario civil y zonas; el calendario comercial sigue siendo responsabilidad del dominio/infraestructura de mercado.

### TM-006 — Las fronteras temporales de proveedores no están normalizadas

**Severidad:** alta.

**Evidencia:** `MarketDataProvider::fetch_candles` recibe rangos inclusivos expresados como `from_ms` y `to_ms`, mientras que la semántica de apertura/cierre de cada vela no está declarada en el contrato del proveedor.

**Riesgo:** proveedores con límites inclusivos/exclusivos, timestamps de apertura o cierre y distintas precisiones pueden devolver duplicados o huecos en paginación/backfill. El contrato actual no permite validar esas diferencias.

**Dirección:** definir contratos de adaptador por proveedor y normalizar a un intervalo interno explícito. Documentar precisión, epoch, unidad, timestamp de evento y semántica de los límites de consulta. Añadir fixtures de paginación sin solapamiento ni pérdida.

### TM-007 — El tipo temporal de chart admite duración cero aunque la vela de mercado la rechaza

**Severidad:** media; requiere decisión semántica antes de endurecer el constructor.

**Evidencia:** `TimePoint::new` rechaza `close_time_ms < open_time_ms`, por lo que acepta apertura y cierre iguales si `interval_ms` es positivo. En cambio, `MarketCandle::is_valid` y `to_chart_data_point` exigen `close_time_ms - open_time_ms > 0`.

**Riesgo:** la misma estructura de chart puede construirse con una duración temporal que el adaptador de velas de mercado considera inválida. Un consumidor podría asumir que todo `TimePoint` representa una vela con span positivo.

**Dirección:** decidir si `TimePoint` representa exclusivamente el intervalo de una vela o un punto/evento temporal más general. Si representa una vela, exigir span positivo y añadir pruebas de igualdad; si es general, separar los tipos y nombrar las invariantes para que el consumidor no infiera una duración que no existe.

### TM-009 — El orden de respuesta del proveedor no está declarado y la capa de aplicación rechaza desorden en lugar de normalizarlo

**Severidad:** alta para proveedores reales y backfills paginados.

**Evidencia:** `MarketDataProvider::fetch_candles` no declara un orden de respuesta. `LoadCandles::execute` acepta únicamente timestamps de apertura estrictamente crecientes respecto del último dato aceptado; un dato duplicado o fuera de orden se agrega a `rejected` en lugar de ordenarse. El mock entrega los datos en orden ascendente, por lo que los tests actuales no ejercitan respuestas reales desordenadas.

**Riesgo:** un proveedor que entregue páginas solapadas, respuestas fuera de orden o reintentos puede producir rechazos parciales. La aplicación no distingue entre una respuesta contractual inválida y un adaptador que todavía no normalizó la respuesta externa.

**Dirección:** cada adaptador debe normalizar y validar orden, duplicados y paginación antes de entregar una secuencia al caso de uso. El contrato debe decidir si la salida interna exige orden ascendente estricto y unicidad por identidad lógica; si lo exige, debe rechazar conflictos y duplicados con política explícita, no ordenar silenciosamente datos ambiguos.

### TM-008 — Los rangos de selección de TimeScale son inclusivos, pero el contrato no lo declara

**Severidad:** media.

**Evidencia:** `TimeScale::indices_in_time_range` selecciona timestamps con `time >= start_time_ms` y `time <= end_time_ms`; las pruebas confirman que un rango `[t, t]` incluye el timestamp `t`. Por separado, `MarketDataProvider::fetch_candles` acepta `from_ms` y `to_ms` sin declarar la inclusión de los extremos.

**Riesgo:** los consumidores pueden asumir que el rango de consulta del proveedor comparte la semántica inclusiva de la selección local. Al paginar o unir backfills, esa suposición puede duplicar o perder eventos.

**Dirección:** documentar cada contrato por separado. Mantener la semántica inclusiva de selección de `TimeScale` mientras sus consumidores la requieran; no trasladarla automáticamente a la API del proveedor. La normalización debe ocurrir en el adaptador de cada fuente con fixtures de frontera.

## Comportamiento observado de consulta histórica

La implementación actual permite describir el comportamiento de la capa de aplicación, pero no convertirlo todavía en contrato universal de proveedores:

- `LoadCandles::execute` valida `from_ms <= to_ms` y acepta velas cuyo `open_time_ms` cae dentro del rango inclusivo `[from_ms, to_ms]`.
- La capa de aplicación exige que los timestamps de apertura aceptados sean estrictamente crecientes en el orden recibido; no ordena la respuesta.
- `MockMarketDataProvider` alinea el primer timestamp de apertura al siguiente múltiplo de `interval_ms` que no sea menor que `from_ms`, y el último al múltiplo no mayor que `to_ms`.
- La interfaz `MarketDataProvider` no documenta ninguna de estas garantías. El mock no sustituye fixtures de proveedores reales.

No cambiar la semántica actual ni generalizarla a todos los proveedores sin decidir antes cómo se paginan rangos solapados y cómo se representan duplicados conflictivos. La semántica de `TimeScale::indices_in_time_range` es otra operación y no define el contrato de consulta histórica.

## Responsabilidades temporales propuestas

| Responsabilidad | Tipo de solución |
|---|---|
| Orden, igualdad, identidad y búsquedas de eventos | Enteros/tipo de instante del dominio |
| Medición de latencia, FPS, timeout y scheduling | `std::time::Instant` |
| Lectura del reloj civil del sistema | `std::time::SystemTime` en el borde |
| Fechas civiles, zonas IANA, DST, parsing y formatting | Biblioteca temporal evaluada |
| Timeframe, sesiones, feriados y cierres de venue | Modelo/calendario de mercado |
| Etiquetas de ejes y zona de visualización | Servicio de presentación con política explícita |

## Matriz para elegir biblioteca

Evaluar Jiff, `time` más su solución IANA y Chrono más `chrono-tz` usando el mismo conjunto de pruebas y criterios:

1. Semántica y seguridad de API para instantes, fechas civiles, offsets, zonas y DST.
2. Fuente y actualización de tzdb en Windows, macOS y Linux; reproducibilidad y política de actualización.
3. Resolución explícita de horas ambiguas e inexistentes.
4. MSRV, compatibilidad con toolchain fijada, licencia y dependencias transitivas.
5. Tamaño del binario, tiempo de compilación y coste de actualización.
6. Parsing/formatting y locale requerido por la UI.
7. Conversión checked desde/hacia el epoch y límites negativos/extremos.
8. Madurez, mantenimiento, documentación y coste de integración.

No medir la biblioteca dentro del hot path de indexación; medir solo rutas que conviertan o formateen fechas. No añadir una dependencia hasta que exista un consumidor real y una prueba de plataforma.

## Plan de pruebas requerido antes de adoptar

- Epoch cero, timestamps negativos y límites representables.
- Conversión round-trip con precisión declarada.
- Años bisiestos, fin de mes, cambio de año y períodos de calendario.
- DST: hora local inexistente y hora local repetida.
- Sesiones que cruzan medianoche, fines de semana, feriados y cierres extraordinarios.
- Paginación de proveedor en fronteras inclusivas/exclusivas y backfill solapado.
- UTC, zona IANA del venue y zona configurable del usuario.
- Windows, macOS y Linux, incluyendo disponibilidad/actualización de tzdb.
- Verificación de que la conversión/formateo no se ejecuta por vela/frame sin necesidad y que etiquetas cacheadas se invalidan al cambiar zona/locale.

## Secuencia recomendada

1. Cerrar la semántica de timestamp y timeframe con al menos un proveedor real objetivo.
2. Definir el tipo canónico de instante/unidad y los contratos de frontera del proveedor.
3. Elegir biblioteca con la matriz anterior y añadirla solo en el límite de calendario/presentación.
4. Implementar política de zona horaria configurable y tests DST.
5. Diseñar identidad de calendario/venue y fixtures de sesiones.
6. Sustituir aproximaciones de granularidad mensual/anual por selección basada en calendario.
7. Medir y cachear conversiones de presentación; preservar el hot path entero.

## Relación con ADR

Este documento complementa ADR-0004 y el apartado Tiempo y agregación de `market-data-contract.md`. No elige aún una biblioteca ni modifica el modelo de ejecución.
