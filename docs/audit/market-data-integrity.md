# Auditoría de integridad de datos de mercado

## Estado y alcance

Auditoría estática incremental de los flujos actuales de datos históricos y realtime. Las conclusiones se basan en la implementación y las pruebas presentes en la rama `chore/engineering-governance-foundation`. No certifica todavía proveedores reales, cálculos de indicadores ni ejecución de órdenes.

El principio rector para una aplicación de análisis financiero es que ningún dato sintético, incompleto, fuera de contrato o numéricamente ambiguo debe presentarse como dato de mercado confiable. La procedencia, el instrumento, el intervalo, la calidad y el estado de cierre deben poder determinarse.

## Flujo actual observado

1. `MarketDataProvider::fetch_candles` devuelve `Vec<MarketCandle>`.
2. `LoadCandles::execute` valida símbolo, rango solicitado y orden ascendente estricto; convierte cada vela aceptada a `ChartDataPoint` y registra rechazos.
3. `ChartState::add_data_points` añade los puntos, ordena por `open_time_ms`, consolida timestamps repetidos y reconstruye `TimeScale`.
4. `RealtimeMarketDataProvider::next_update` devuelve una vela y un booleano `is_closed`.
5. `advance_realtime` convierte el update a punto y lo inserta mediante `ChartState::add_data_point`; el timestamp de apertura existente se reemplaza.

## Hallazgos confirmados

### MD-001 — La aplicación sustituye un fallo de carga por datos sintéticos

**Severidad:** crítica para un producto financiero conectado a datos reales.

**Evidencia histórica:** `src/application/services/mod.rs`, función `load_symbol_into_chart`, en una revisión anterior.

En el head actual, `load_symbol_into_chart` está en `src/presentation/desktop/app.rs`. Ante un error limpia `ChartState` y devuelve `LoadState::Error`; la prueba `failed_market_load_does_not_publish_synthetic_data` verifica que el gráfico permanece vacío. El fallback histórico sintético implícito está eliminado. Sin embargo, el arranque y la recarga de la aplicación siguen seleccionando explícitamente `MockMarketDataProvider` y `MockRealtimeMarketDataProvider` como flujo de demostración. No existe todavía una selección de modo de ejecución ni una procedencia de simulación integrada al estado de la serie.

**Riesgo:** un usuario puede analizar una serie que no procede del mercado esperado, y la continuidad visual/realtime puede hacer que parezca una serie válida. En una aplicación financiera, una recuperación nunca debe reemplazar silenciosamente datos reales por datos sintéticos.

**Dirección requerida:** eliminar el fallback sintético del flujo de producción. Los mocks deben estar confinados a un modo de simulación explícito, con identidad y procedencia visibles y aisladas. Ante un fallo de proveedor, conservar o limpiar el último conjunto válido según una política explícita, marcar los datos como obsoletos/no disponibles y bloquear cualquier afirmación de actualidad. No activar realtime si la serie base no está validada para el mismo instrumento y fuente.

### MD-002 — La ingestión parcial se presenta como carga exitosa

**Severidad:** alta.

**Evidencia:** `src/application/commands/load_candles.rs` y `src/presentation/desktop/app.rs`.

`LoadCandles::execute` devuelve puntos aceptados y una colección `rejected`. Si hay rechazos, `load_symbol_into_chart` en `src/presentation/desktop/app.rs` solo emite advertencias de tracing, agrega los puntos aceptados y devuelve `LoadState::Loaded { count }`. El pie de la UI muestra el estado y el número de puntos, pero no el número de rechazos ni una condición de calidad degradada.

**Riesgo:** huecos por velas rechazadas, datos parciales o inconsistencias del proveedor pueden confundirse con un histórico íntegro. La telemetría de logs no es suficiente para informar al usuario ni para que otros casos de uso tomen decisiones seguras.

**Dirección requerida:** representar explícitamente estados de calidad de ingestión (íntegra, parcial, rechazada, obsoleta, simulada, etc.), exponer recuentos y causas estructuradas y definir si un caso de uso puede continuar con datos parciales. Para análisis que requieran continuidad, permitir política fail-closed. La política debe ser configurable por caso de uso, no implícita.

### MD-003 — La representación autoritativa de precios y cantidades es binaria flotante

**Severidad:** alta; crítica si estos valores alimentan cálculos financieros, límites, señales u órdenes.

**Evidencia:** `src/market/candle.rs`, `src/domain/candle.rs`, `src/domain/price.rs`, `src/domain/quantity.rs` y `src/application/services/mod.rs`.

OHLCV se representa como `f64` desde `MarketCandle` hasta `Candle`. Existen `Price` y `Quantity` basados en `rust_decimal::Decimal`, pero no constituyen la representación canónica de las velas ni gobiernan el flujo de ingestión. La UI convierte rangos desde `f64` mediante `Decimal::from_f64`, lo que no recupera necesariamente la representación decimal exacta que entregó el mercado.

**Riesgo:** los decimales financieros comunes no tienen representación binaria exacta. Comparaciones, cuantización a tick, agregaciones, porcentajes y límites pueden acumular diferencias o producir valores que no son múltiplos válidos del instrumento. La conversión posterior a Decimal no restaura la precisión original.

**Dirección requerida:** definir una política numérica por dominio y por instrumento antes de implementar cálculos financieros. Para valores autoritativos, evaluar Decimal o unidades enteras escaladas (ticks/lots) con escala y reglas de redondeo explícitas. Mantener tipos de punto flotante solo en fronteras de visualización/transformación, con conversión unidireccional, controlada y testeada. Definir precisión, escala, redondeo y overflow para cada operación; no aplicar una escala global arbitraria.

### MD-004 — La precisión de tick se infiere del nombre del símbolo

**Severidad:** alta para visualización financiera precisa.

**Evidencia:** `price_tick_size_for_symbol` en `src/application/services/mod.rs`.

La función asigna incrementos mediante heurísticas por nombre: ciertos futuros reciben 0.25, pares FX reciben 0.00001 o 0.001 y el resto recibe 0.01. No existe un contrato de metadatos de instrumento/proveedor/venue que determine tick size, cantidad mínima, escalas ni reglas de cotización.

**Riesgo:** el mismo símbolo puede tener reglas diferentes según mercado, broker, contrato o venue. Una inferencia incorrecta afecta la graduación del eje y puede inducir a interpretar niveles de precio con una precisión que el instrumento no admite.

**Dirección requerida:** obtener especificaciones de instrumento desde metadatos autoritativos asociados a venue y contrato. El símbolo textual por sí solo no identifica todas las reglas de negociación. La UI debe indicar cuándo los metadatos faltan y no inventar una precisión.

### MD-005 — La actualización realtime no aplica una política de finalización de velas

**Severidad:** alta.

**Evidencia:** `src/market/provider.rs`, `src/market/mock_realtime.rs`, `src/presentation/desktop/app.rs` y `src/domain/chart_state.rs`.

El contrato realtime expone `RealtimeMarketUpdate.is_closed`. `advance_realtime` convierte cada update a `ChartDataPoint` y lo aplica mediante `ChartState::add_data_point`, independientemente de que esté abierto o cerrado. `is_closed` solo condiciona un mensaje de tracing. `ChartState` reemplaza el punto existente con el mismo `open_time_ms`; `ChartDataPoint` no conserva el estado de cierre. Tampoco existe identidad de update, secuencia del proveedor, versión o procedencia que permita distinguir una retransmisión de una corrección.

**Riesgo:** una actualización tardía, duplicada o fuera de secuencia podría reescribir una vela finalizada sin distinguir una corrección legítima del proveedor de una regresión o un mensaje inválido. Tampoco hay una política explícita de reconciliación entre histórico y realtime.

**Dirección requerida:** modelar el ciclo de vida de la vela (provisional, cerrada, corregida) y la semántica de secuencia/versionado del proveedor. Definir reglas de idempotencia, correcciones tardías, duplicados, gaps y reconciliación histórico/realtime. Las actualizaciones deben validarse antes de mutar el estado y los rechazos deben ser observables. La política debe impedir que un update ordinario reemplace una vela cerrada; las correcciones posteriores requieren autoridad y versión explícitas.

### MD-006 — Invariantes temporales de TimePoint

**Severidad original:** alta.

**Estado:** encapsulación aplicada en el head actual; pendiente de validación CI.

**Evidencia:** `src/domain/chart/time_point.rs` ahora mantiene privados `open_time_ms`, `close_time_ms` e `interval_ms` y expone getters de solo lectura. Los consumidores se actualizaron para usar esos getters. La única construcción pública es `TimePoint::new`, que rechaza rangos invertidos e intervalos no positivos. `ChartState` recibe `ChartDataPoint` con un `TimePoint` ya validado, por lo que ya no puede recibir desde otros módulos un valor temporal fabricado mediante struct literal o mutación de campos.

**Límite deliberado:** `interval_ms` representa el intervalo nominal y no se fuerza a coincidir con `close_time_ms - open_time_ms`; esa semántica requiere una decisión de contrato de mercado/timeframe. La igualdad de timestamps sigue siendo aceptada por `TimePoint::new` por ahora, mientras que el adaptador de mercado rechaza velas de duración cero.

**Validación requerida:** CI multiplataforma verde en el head que contiene la encapsulación.

### MD-007 — La política de duplicados depende del camino de ingestión

**Severidad:** media-alta.

**Evidencia:** `src/application/commands/load_candles.rs`, `src/domain/chart_state.rs` y `src/presentation/desktop/app.rs`.

La carga histórica rechaza timestamps duplicados y fuera de orden durante `LoadCandles::execute`. En cambio, `ChartState::add_data_points` extiende el vector recibido y `rebuild_time_scale` lo ordena y consolida timestamps repetidos conservando el último punto según el orden del lote. La inserción individual busca por `open_time_ms` y reemplaza el punto existente. El flujo de presentación actual llama primero a `LoadCandles`, registra los rechazos, limpia el chart y luego agrega el lote aceptado. Por tanto, hoy el lote histórico llega ya filtrado, pero `ChartState` conserva una política de resolución distinta que también puede ser invocada por otros caminos.

**Riesgo:** el resultado depende del camino de ingestión. La regla de “último recibido gana” no distingue retransmisión idéntica, orden de llegada, tiempo de evento, versión del proveedor ni corrección autorizada. Además, el estado del gráfico no dispone de identidad de serie ni procedencia para determinar si dos puntos con igual apertura son comparables.

**Dirección requerida:** mantener la resolución de duplicados/conflictos en la frontera de normalización y en la política de aplicación, antes de proyectar los puntos al gráfico. `ChartState` debe recibir una secuencia validada y no decidir autoridad de mercado mediante ordenamiento/deduplicación implícitos. Definir identidad de vela por serie más período y diferenciar duplicado idéntico, actualización provisional, corrección autorizada y conflicto irreconciliable.

## Observaciones sobre pruebas existentes

- Hay pruebas de determinismo del proveedor mock, continuidad de su serie, alineación de rangos y reemplazo de timestamps duplicados.
- Hay pruebas de ordenamiento/deduplicación de lotes en `ChartState`.
- Las pruebas de propiedad de `MarketCandle` no cubren de forma suficiente timestamps extremos, igualdad de timestamps, desbordamiento de duración, ni la propiedad de que toda vela declarada válida pueda convertirse a todos los tipos derivados.
- Las pruebas actuales no definen una matriz de reconciliación histórico/realtime, la inmutabilidad de velas cerradas, el tratamiento de mensajes fuera de secuencia ni los límites de calidad de una ingestión parcial.
- Las pruebas de la serie mock no validan precisión de mercado, tick size ni exactitud decimal; son pruebas de comportamiento del generador sintético.

## Contrato de datos que debe definirse antes de refactorizar

1. **Identidad:** venue, instrumento canónico, timeframe, apertura, fuente y, cuando exista, identificador/versionado del proveedor.
2. **Tiempo:** unidad, zona/epoch, semántica inclusiva/exclusiva de cierre, calendario de mercado, sesiones, DST, gaps y timestamps de evento/recepción.
3. **Precisión:** escala de precio/cantidad por instrumento, tick/lot, redondeo, overflow y representación autoritativa.
4. **Calidad:** válido, parcial, obsoleto, provisional, cerrado, corregido, simulado y rechazado; procedencia y motivo.
5. **Orden y conflictos:** idempotencia, duplicados, out-of-order, backfill, correcciones y reconciliación histórico/realtime.
6. **Atomicidad:** validación previa del lote y mutación transaccional o explícitamente parcial.
7. **Capacidad:** límites de lote, retención, backpressure y política de memoria para históricos extensos y streams prolongados.

## Próximas verificaciones

1. Revisar el ciclo de cambio de símbolo, especialmente el comportamiento cuando falla la carga y cuando el símbolo es inválido.
2. Definir fixtures y pruebas de propiedades para extremos temporales, duplicados conflictivos, datos fuera de orden, gaps y actualizaciones tardías.
3. Diseñar el modelo de instrumento y la representación numérica autoritativa antes de cambiar tipos en masa.
4. Separar modo simulación del modo de datos de mercado y eliminar fallback sintético implícito.
5. Diseñar una ingestión validada y transaccional con estados de calidad explícitos.

## Restricción de implementación

Esta fase registra riesgos y contratos pendientes. No se modifican todavía tipos ni comportamiento de mercado hasta acordar semánticas y agregar pruebas que hagan verificables las reglas. Todo código nuevo o modificado debe mantener la política de cero comentarios.


### MD-008 — Un símbolo inválido deja visible la serie del símbolo anterior

**Severidad:** media-alta.

**Evidencia:** `src/application/services/mod.rs`, flujo de recarga del símbolo.

Cuando el texto ingresado no puede convertirse a `Symbol`, la aplicación detiene el proveedor realtime y cambia el estado a error, pero no limpia ni marca como anterior el contenido existente de `ChartState`. La pantalla conserva la serie previa mientras el campo de símbolo contiene el texto inválido.

**Riesgo:** la identidad del dato visible deja de estar sincronizada con el instrumento solicitado. Aunque el error de validación se registra en el estado de carga, la serie anterior continúa ocupando el gráfico y no existe una identidad de instrumento adjunta a cada conjunto de puntos que permita verificar la correspondencia desde la presentación.

**Dirección requerida:** separar el texto de búsqueda del instrumento activo. Mostrar siempre el instrumento/procedencia asociado a la serie actualmente renderizada. Ante una solicitud inválida, conservar la serie previa solo si queda claramente identificada como último instrumento válido; no presentar el texto inválido como identidad de la serie. Añadir pruebas de transición entre símbolo válido, inválido y válido distinto.



### MD-009 — La identidad de vela no incluye timeframe ni mercado

**Severidad:** crítica para series múltiples.

**Evidencia:** `src/market/provider.rs`, `src/market/candle.rs`, `src/domain/chart_state.rs` y `src/domain/chart/chart_data.rs`.

El contrato histórico recibe símbolo y rango temporal, pero no timeframe ni venue. `MarketCandle` conserva símbolo y timestamps, pero no timeframe, fuente, tipo de precio ni identificador de instrumento canónico. `ChartState` ordena y deduplica exclusivamente por `open_time_ms`.

**Riesgo:** dos series del mismo símbolo con distinta granularidad, venue, proveedor o tipo de precio pueden colisionar si llegan al mismo estado. La deduplicación podría reemplazar una vela de una serie con una vela perteneciente a otra, sin que el modelo pueda detectar el conflicto.

**Dirección requerida:** introducir una identidad de serie explícita y una identidad lógica de vela compuesta por serie más período. Incluir timeframe/definición de agregación y los metadatos de mercado necesarios. Separar estados por serie o impedir que un estado de gráfico mezcle identidades incompatibles. El contrato propuesto está en `market-data-contract.md`.

### MD-010 — La identidad del símbolo no tiene política de canonicalización

**Severidad:** media-alta.

**Evidencia:** `src/market/symbol.rs`.

El parser acepta tanto `BTC/USDT` como `btc/usdt`, pero conserva literalmente las cadenas y la igualdad/hash de `Symbol` distinguen ambos valores. El generador mock también deriva su semilla de la representación textual. No existe un catálogo ni un mapeo explícito de alias a instrumento canónico.

**Riesgo:** identificadores que un proveedor considere equivalentes pueden crear series separadas, fallar la validación de símbolo o generar datos sintéticos distintos. No es correcto imponer uppercase universalmente sin considerar los mercados que pueden tener identificadores sensibles a mayúsculas.

**Dirección requerida:** resolver símbolos mediante metadatos del proveedor y un identificador canónico de instrumento. Mantener el símbolo externo como alias con ámbito de proveedor/venue. Definir equivalencia y normalización por mercado, no por una transformación textual global.

### MD-011 — Parte de las pruebas de propiedades no verifica la propiedad que su nombre anuncia

**Severidad:** alta para confianza de validación.

**Evidencia:** `tests/market_proptest.rs`.

La propiedad `prop_market_candles_unique_timestamps` ordena timestamps, los deduplica y comprueba que el número deduplicado sea menor o igual al original. Esa desigualdad es cierta por construcción y no ejercita `ChartState`, no valida unicidad en el estado final y no comprueba la política de conflictos. Además, `prop_market_candle_to_domain_candle` solo prueba conversión a `Candle`, no conversión completa a `ChartDataPoint`; la estrategia genera timestamps positivos pequeños y no alcanza extremos ni overflow.

**Riesgo:** el nombre de las pruebas puede transmitir una cobertura mayor que la realmente existente. Los casos límite temporales y la reconciliación de datos siguen sin protección basada en propiedades.

**Dirección requerida:** reemplazar las propiedades tautológicas por pruebas sobre comportamiento observable del sistema: inserción en `ChartState`, número de series/puntos, identidad de vela, conflicto y resultado determinista. Generar valores en límites de enteros, escalas decimales, rangos temporales y estados realtime; comprobar que toda vela declarada válida pueda convertirse a cada tipo derivado requerido.


## Resolución parcial — MD-007 temporal

La implementación de `MarketCandle` ahora requiere una duración estrictamente positiva y calcula la diferencia temporal mediante `checked_sub`. `is_valid` y `to_chart_data_point` comparten la misma condición efectiva: los timestamps iguales, invertidos o cuya resta desborda `i64` se rechazan. Se añadieron pruebas para los tres casos y para la conversión de un intervalo positivo. La semántica del intervalo sigue siendo el span entre apertura y cierre al crear `TimePoint`; no se impone ninguna equivalencia adicional sobre intervalos nominales en otros caminos. La corrección queda pendiente de CI en el head actualizado.


## Resolución — MD-001 fallback sintético implícito

Se eliminó la generación de un histórico sintético de respaldo cuando falla `LoadCandles::execute`. El fallo ahora deja el `ChartState` vacío y devuelve `LoadState::Error`; no se publican velas mock como sustituto de una carga fallida. Se añadió una prueba con un proveedor que devuelve error para verificar que el estado permanece sin puntos y conserva el estado de error.

Los proveedores mock siguen formando parte del arranque actual de demostración; esta corrección no los convierte en proveedores reales ni implementa todavía una separación completa entre modo simulación y modo de mercado. La procedencia y el modo de ejecución deben hacerse explícitos en una etapa posterior. La resolución queda pendiente de CI en el nuevo head.


### MD-012 — La aplicación inicia realtime sin un contrato de continuidad verificable

**Severidad:** alta.

**Evidencia:** `src/presentation/desktop/app.rs`, funciones de recarga, `create_realtime_provider` y `advance_realtime`.

Tras la carga histórica, la aplicación intenta crear el proveedor realtime a partir del último punto visible en `ChartState`. La construcción reconstruye un `MarketCandle` desde los valores de presentación y deriva el intervalo nominal de `TimePoint`. Luego inicializa el mock realtime con ese punto. No existe una identidad de serie completa ni metadatos de proveedor, venue, timeframe o procedencia que permitan demostrar que el flujo realtime continúa exactamente la misma serie histórica. Si la carga histórica fue parcial pero aceptada, también puede iniciarse realtime a partir del último punto aceptado, aunque la continuidad del histórico no esté certificada.

**Riesgo:** el gráfico puede presentar una secuencia visualmente continua sin que exista una garantía de continuidad de identidad, calidad o autoridad entre histórico y realtime. Reconstruir el objeto de mercado desde un modelo de presentación también pierde cualquier metadato que no esté representado en `ChartDataPoint`.

**Dirección requerida:** el punto de continuidad debe proceder de un resultado de ingestión validado que conserve identidad de serie, procedencia, calidad y lifecycle. El inicio realtime debe requerir una política explícita sobre completitud/aceptación del histórico y coincidencia de identidad entre ambos flujos. No inferir equivalencia de serie solo por símbolo y timestamp.

### MD-013 — El modo mock está seleccionado directamente desde la presentación

**Severidad:** alta para la futura integración de proveedores reales.

**Evidencia:** `src/presentation/desktop/app.rs`, inicialización de la aplicación y acción de recarga.

La presentación instancia directamente `MockMarketDataProvider` y `MockRealtimeMarketDataProvider`, configura sus parámetros y calcula el rango histórico. Los mocks son el flujo de demostración activo, no un fallback ante error; sin embargo, no existe todavía una selección explícita entre modo simulación y modo conectado ni una identidad de origen expuesta al estado de la serie.

**Riesgo:** la presentación controla la elección de infraestructura y no puede expresar de manera uniforme qué modo y procedencia alimentan el gráfico. Al incorporar proveedores reales, esta construcción directa aumenta el acoplamiento y el riesgo de mezclar datos de simulación con datos de mercado.

**Dirección requerida:** introducir una decisión de composición en bootstrap o en un límite de aplicación dedicado. El modo de simulación y el modo conectado deben seleccionarse explícitamente, y la procedencia debe acompañar al resultado de ingestión hasta la presentación. No convertir los mocks en un proveedor por defecto silencioso ni ocultar su naturaleza.
