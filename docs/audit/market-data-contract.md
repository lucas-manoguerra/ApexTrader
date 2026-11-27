# Contrato canónico de datos de mercado

## Estado

Borrador de arquitectura y calidad, versión 0.1. Define requisitos para los límites de ingestión y los modelos internos. No declara todavía una representación numérica final ni presupone reglas de un proveedor o venue. Los detalles dependientes del mercado deben proceder de metadatos autoritativos.

## Principios no negociables

1. Un dato no se considera confiable solo porque sus campos sean sintácticamente válidos.
2. La identidad de una serie debe ser inequívoca y conservarse desde la fuente hasta la visualización.
3. Ningún error de proveedor puede sustituir datos reales por datos sintéticos de forma implícita.
4. Una ingestión parcial debe ser representada como parcial; nunca como íntegra.
5. Los datos recibidos deben validarse completamente antes de mutar el estado publicado.
6. La misma entrada, versión de contrato y política de resolución debe producir el mismo resultado.
7. Las conversiones a tipos gráficos aproximados no pueden convertirse en la fuente autoritativa de cálculos financieros.
8. Todo rechazo, corrección, gap y cambio de procedencia debe ser trazable.
9. Los límites de capacidad deben ser explícitos y producir backpressure o errores controlados, nunca crecimiento ilimitado.
10. El modo de simulación debe ser explícito, aislado y visualmente distinguible del modo de mercado.

## Identidad de instrumento y serie

El texto visible del símbolo no es una identidad suficiente. El modelo debe distinguir:

- Instrumento canónico e identificador estable asignado por el mercado o catálogo de instrumentos.
- Venue/mercado y tipo de instrumento cuando corresponda.
- Activo base y cotizado cuando aplique.
- Proveedor y su identificador externo, conservando el mapeo al instrumento canónico.
- Timeframe o definición de agregación.
- Tipo de precio utilizado para construir la vela cuando el mercado lo distinga (trades, mark, index, bid/ask u otro).
- Sesión/calendario aplicable cuando afecte la agregación.

La clave lógica mínima de una vela será la identidad de serie más el inicio del período. No se debe colapsar por timestamp únicamente. La equivalencia entre alias, mayúsculas/minúsculas y símbolos de distintos proveedores debe definirse mediante catálogo y mapeo explícito; no mediante normalización textual asumida.

## Especificación de instrumento

Los metadatos autoritativos del instrumento deben incluir, cuando el mercado los defina:

- Tick size y reglas de precio, incluyendo incrementos variables o bandas.
- Cantidad mínima, step size y unidades de cantidad.
- Escala y precisión máxima de precio y cantidad.
- Moneda de cotización y unidad de volumen.
- Multiplicador/contract size y reglas de valoración para derivados.
- Calendario, sesiones, zona horaria y reglas de cierre.
- Vigencia temporal y versión de los metadatos.

La ausencia de un metadato requerido debe producir un estado explícito de configuración incompleta. No se debe inferir tick size, lot size ni precisión a partir del nombre del símbolo.

## Representación numérica

### Entrada

- Los adaptadores deben preservar el valor decimal recibido sin pasar primero por f32 o f64.
- Si un proveedor entrega números JSON, el adaptador debe preservar el token decimal original o utilizar una ruta de parseo que no introduzca una conversión binaria intermedia.
- Precio y cantidad deben validarse contra la escala, rango y reglas del instrumento.
- La validación debe rechazar NaN, infinitos, overflow, valores negativos no permitidos y valores no representables según el contrato del mercado.

### Modelo interno

La decisión entre decimal de precisión fija y unidades enteras escaladas queda pendiente de un ADR específico. Debe evaluarse con:

- Rango máximo y mínimo requerido por todos los instrumentos soportados.
- Escalas máximas y variabilidad de tick/lot.
- Exactitud de operaciones y reglas de redondeo.
- Overflow explícito y comportamiento de división.
- Coste medido en cargas reales y benchmarks reproducibles.
- Serialización, interoperabilidad y compatibilidad con proveedores.

No se aceptará una conversión silenciosa que redondee o trunque. Cada operación que pueda redondear debe nombrar la regla aplicada y ofrecer tests de límites. El dominio financiero debe conservar la precisión autoritativa; el renderer puede recibir una proyección aproximada para coordenadas de pantalla.

## Tiempo y agregación

Cada dato debe distinguir, cuando estén disponibles:

- Tiempo de evento del mercado.
- Inicio y fin del período de la vela.
- Tiempo de recepción local.
- Tiempo de publicación/procesamiento local.

El contrato debe especificar epoch, unidad, semántica de apertura/cierre y límites inclusivos/exclusivos. Las duraciones fijas y los períodos de calendario no son intercambiables: día, semana, mes y sesiones deben derivarse del calendario/venue cuando corresponda, no de sumar una cantidad fija de milisegundos.

Los cálculos de duración deben usar operaciones checked. Toda vela aceptada debe poder convertirse a las representaciones derivadas requeridas sin overflow. Los intervalos nominales y los spans reales deben tener nombres y semánticas separadas.

## Comportamiento actual de la consulta histórica

La implementación existente tiene un comportamiento concreto que debe distinguirse del contrato objetivo para proveedores externos:

- `LoadCandles::execute` valida que `from_ms <= to_ms` y filtra por `open_time_ms` dentro de los límites inclusivos `[from_ms, to_ms]`.
- Los timestamps de apertura aceptados deben llegar en orden estrictamente ascendente. La capa de aplicación rechaza entradas duplicadas o desordenadas; no las ordena.
- `MockMarketDataProvider` devuelve aperturas alineadas a su intervalo nominal y trata ambos límites como inclusivos. Su comportamiento no demuestra que un proveedor externo use la misma convención.
- `MarketDataProvider::fetch_candles` todavía no expresa orden, unicidad, semántica de límites, precisión ni timestamp que gobierna el filtro.

Por tanto, no asumir que el rango inclusivo actual es el contrato canónico definitivo. Cada adaptador externo debe declarar su semántica nativa y convertirla explícitamente a un contrato interno acordado. La política de paginación debe evitar pérdida y duplicación incluso cuando el proveedor incluya ambos extremos. Un adaptador no debe ordenar silenciosamente dos eventos con la misma identidad lógica y contenido conflictivo.

## Ciclo de vida y calidad

El modelo debe poder distinguir como mínimo:

- Provisional: puede recibir nuevas actualizaciones dentro de la misma serie/período.
- Cerrada: el proveedor declara terminado el período; no se modifica sin evento explícito de corrección.
- Corregida: reemplazo autorizado de una versión previamente publicada.
- Parcial: la ingestión no contiene todos los datos esperados o hay rechazos/gaps.
- Obsoleta: no hay evidencia suficiente de actualidad.
- Simulada: generada por un motor de simulación, nunca confundible con datos externos.
- Rechazada: incumple contrato y no ingresa en la serie aceptada.

La calidad debe ser metadata estructurada, no solo texto de log. Debe registrar fuente, motivo, contadores y momento de evaluación.

## Orden, duplicados y reconciliación

- La identidad lógica de una vela debe ser única dentro de la serie.
- Repetir exactamente el mismo evento debe ser idempotente.
- Un evento con misma identidad pero distinto contenido no debe resolverse con “último recibido gana” sin política de versión/autoridad.
- Las actualizaciones provisionales deben respetar secuencia o versión del proveedor.
- Una vela cerrada solo puede cambiar mediante un evento de corrección identificable y auditable.
- Los mensajes fuera de orden deben almacenarse/reordenarse o rechazarse según una ventana y política explícitas.
- Los gaps deben detectarse respecto al calendario y timeframe; un gap de mercado cerrado no equivale a pérdida de datos.
- La reconciliación entre histórico y realtime debe comparar identidad, versión, estado y procedencia, y producir un resultado explícito.

## Atomicidad y capacidad

La ingestión por lote debe validar identidad, secuencia, rangos, precisión y temporalidad antes de publicar cambios. Si la política admite rechazos parciales, el resultado debe separar claramente accepted/rejected y no publicar un estado íntegro. Las mutaciones deben ser atómicas por lote o declarar explícitamente su granularidad transaccional.

Deben existir límites configurables y medibles para velas por respuesta/lote, tamaño de payload, backfill simultáneo, retención por serie, número de series activas, cola y retraso realtime, memoria y backpressure. El sistema debe exponer métricas de retraso, gaps, rechazos, correcciones, duplicados, tamaño de buffers y estado de conexión.

## Contratos de adaptadores

El adaptador histórico debe declarar proveedor, instrumento canónico, timeframe, rango efectivo, paginación y calidad de respuesta.

El adaptador realtime debe exponer identidad de serie, secuencia/versionado, tiempo de evento, tiempo de recepción, estado de vela y semántica de corrección. Un booleano aislado is_closed no es suficiente para reconciliación robusta.

Los errores de red, autenticación, rate limit, símbolo no soportado, respuesta malformada, datos parciales, datos obsoletos y conflicto de versión deben ser distinguibles.

## Requisitos de pruebas

La suite deberá incluir:

- Pruebas de parseo decimal exacto y round-trip.
- Límites de escala, rango, tick, lot y overflow.
- Timestamps iguales, invertidos y extremos de i64.
- Intervalos fijos y de calendario, sesiones y DST.
- Identidad de serie y aislamiento entre venue/proveedor/timeframe.
- Alias de símbolo y reglas de canonicalización.
- Duplicados idénticos y conflictivos.
- Actualizaciones provisionales, cierre, corrección tardía y mensajes fuera de secuencia.
- Gaps esperados por calendario frente a gaps de datos.
- Ingestión transaccional y política fail-closed/fail-open explícita.
- Reconciliación histórico/realtime y reintentos idempotentes.
- Fallos de proveedor sin fallback sintético.
- Límites de memoria, backpressure y recuperación de streams.
- Pruebas de propiedades y fuzzing de adaptadores/serialización.

Las pruebas deben comprobar propiedades observables del sistema, no identidades tautológicas ni solo la consistencia interna de un generador mock.

## Contrato interno de consulta histórica pendiente

Antes de conectar un proveedor real, especificar en el trait o en un tipo de consulta explícito:

- Qué timestamp se filtra: apertura del período, cierre, timestamp de evento u otro identificador temporal.
- Unidad, precisión y epoch del timestamp.
- Si los límites de consulta son inclusivos o exclusivos y cómo se transforma la convención nativa del proveedor.
- Orden de salida requerido y si la unicidad se evalúa por instrumento/serie más apertura.
- Política para duplicados idénticos, duplicados conflictivos y respuestas desordenadas.
- Continuación/paginación sin pérdida ni solapamiento, incluyendo la precisión mínima del timestamp.
- Si el proveedor puede devolver velas parciales, abiertas o fuera de la ventana solicitada.

La semántica de consulta debe ser independiente de la semántica del intervalo de una vela. El límite de una consulta selecciona registros; el intervalo de una vela describe un período de mercado. No se debe inferir uno del otro.

## Decisiones abiertas que requieren ADR

1. Representación numérica autoritativa y estrategia de redondeo.
2. Modelo de instrumento y catálogo/metadatos.
3. Semántica de timeframe y calendario.
4. Modelo de versión, secuencia y correcciones realtime.
5. Política de ingestión parcial y publicación transaccional.
6. Retención, límites y backpressure.

Ninguna de estas decisiones debe cerrarse con constantes arbitrarias. Deben apoyarse en los mercados que ApexTrader pretende soportar, contratos reales de proveedores, pruebas de límites y mediciones.

## Criterio de aceptación

No se habilitará análisis financiero sobre una serie si no puede determinarse su identidad, procedencia, timeframe, precisión y estado de calidad. Si falta información esencial, el sistema debe degradarse de forma visible y segura, o rechazar el conjunto de datos.
