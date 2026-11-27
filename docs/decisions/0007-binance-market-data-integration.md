# ADR-0007: Estrategia de integración de datos de mercado de Binance

## Estado

Propuesta para revisión. La decisión de producto es soportar Binance Spot y Binance USDⓈ-M Futures. La entrega se realizará incrementalmente: Spot como primer adaptador ejecutable y USDⓈ-M Futures como segundo adaptador sobre los mismos límites internos. Esta ADR no autoriza todavía I/O de red en la UI ni ejecución de órdenes.

## Contexto

ApexTrader tiene una visión híbrida: terminal de análisis, trading asistido y evolución posterior hacia capacidades algorítmicas. El proveedor inicial elegido es Binance y el alcance de mercado solicitado comprende Spot y USDⓈ-M Futures.

El estado actual contiene los puertos síncronos `MarketDataProvider::fetch_candles` y `RealtimeMarketDataProvider::next_update` en `src/market/provider.rs`. `ApexTraderApp::ui` llama a `advance_realtime` desde el hilo de interfaz. El bootstrap desktop instancia proveedores mock. `MarketCandle` conserva símbolo, OHLCV y timestamps como `f64/i64`; `Symbol` solo representa BASE/QUOTE. Por tanto, hoy no existe identidad suficiente para distinguir, por ejemplo, BTCUSDT Spot de BTCUSDT USDⓈ-M Futures, ni para identificar timeframe/proveedor en una serie.

Las Issues #25 y #34 requieren resolver ejecución fuera del hilo UI, ciclo de vida, cancelación, backpressure y descarte de respuestas obsoletas. Esta ADR define el contrato de integración de Binance, pero la implementación de red depende de esas decisiones.

## Decisión

### 1. Mercados y secuencia de entrega

- El diseño interno soportará dos segmentos explícitos: `Spot` y `UsdMarginedFutures`.
- La primera integración funcional será Binance Spot, por ser el camino más sencillo para validar histórico, streaming, normalización y ciclo de vida sin añadir de entrada la semántica de contratos.
- USDⓈ-M Futures será el segundo adaptador. No se simulará como Spot ni se inferirá el segmento a partir del texto del símbolo.
- La interfaz de selección de mercado y la identidad de series deberán distinguir ambos segmentos antes de permitir cargar datos de ambos en una misma sesión.
- La arquitectura no se extenderá automáticamente a COIN-M, Options, Margin ni otros productos Binance; requerirán decisión propia.

### 2. Interfaces públicas oficiales

**Spot**
- Histórico: `GET https://data-api.binance.vision/api/v3/klines` (alternativamente `https://api.binance.com/api/v3/klines`).
- Metadatos: `GET https://data-api.binance.vision/api/v3/exchangeInfo`.
- Streaming: `wss://data-stream.binance.vision/ws/<symbol>@kline_<interval>` (alternativamente el host Spot documentado por Binance).
- Klines se identifican por símbolo e instante de apertura. El endpoint REST devuelve arrays posicionales; OHLCV numérico se transmite como strings decimales y los tiempos como timestamps.

**USDⓈ-M Futures**
- Histórico: `GET https://fapi.binance.com/fapi/v1/klines`.
- Metadatos: `GET https://fapi.binance.com/fapi/v1/exchangeInfo`.
- Streaming: `wss://fstream.binance.com/ws/<symbol>@kline_<interval>`.
- El endpoint de klines admite hasta 1500 registros por petición; Spot admite hasta 1000. La paginación debe respetar los límites propios de cada mercado y no asumir un tamaño compartido.

La documentación oficial es la fuente de verdad para paths, hosts, parámetros, intervalos, pesos y límites operativos. Esos valores pueden cambiar; deben verificarse al implementar y no codificarse como garantías permanentes en esta ADR.

### 3. Metadatos e identidad de instrumento

Definir una identidad explícita de instrumento/serie independiente del símbolo de presentación:

- `venue = Binance`
- `market_segment = Spot | UsdMarginedFutures`
- `base_asset`, `quote_asset` y, para futuros, `contract_type` y `margin_asset` cuando corresponda
- símbolo nativo del proveedor
- timeframe/intervalo
- zona temporal de la serie (UTC por defecto)
- fuente/tipo de kline (trade price; no mezclar mark-price, index-price ni continuous-contract klines con velas de símbolo sin identificarlo)

El `Symbol` actual BASE/QUOTE puede seguir siendo una representación de UI, pero no es una clave primaria suficiente. La evolución del modelo debe evitar romper la compatibilidad del chart durante la migración.

Consultar `exchangeInfo` para validar que el instrumento está disponible y obtener filtros/metadatos. Preservar como strings decimales los valores de filtros tales como `PRICE_FILTER.tickSize`, `LOT_SIZE.stepSize`, límites y precisiones. En la fase de market data estos metadatos sirven para describir instrumentos y formatear precios; la validación de órdenes queda fuera de alcance.

### 4. Normalización y precisión

- Parsear los campos decimales de Binance desde strings, nunca pasando primero por `f64` si se necesita conservar el valor original.
- Los DTOs del adaptador son tipos de infraestructura; no filtrar JSON, nombres de campos Binance ni tipos del SDK a dominio/aplicación.
- Definir una política de precisión financiera antes de usar los datos para cálculo, señales o trading. La conversión al chart actual basado en `f64` debe ser explícita y quedar en el borde de visualización.
- Validar OHLC, volumen, timestamps, símbolo, segmento e intervalo antes de publicar datos.
- No inferir tick size, step size ni precisión a partir de la cantidad de decimales observada en una vela.

### 5. Velas históricas y realtime

- Normalizar todos los tiempos a Unix epoch en milisegundos UTC en el contrato interno; el intervalo solicitado forma parte de la identidad de la serie.
- Spot y USDⓈ-M Futures publican actualizaciones de kline durante la vida de la vela. El campo de cierre del evento (`x`) determina si la vela está cerrada; no asumir que cada mensaje WebSocket representa una vela final.
- Mantener una vela abierta mutable por identidad de serie + open time. Al recibir actualización de la misma vela, reemplazar su snapshot con el más reciente; al recibir `x=true`, marcarla cerrada e inmutable salvo política explícita de correcciones.
- Deduplicar por identidad de serie + open time. Rechazar o aislar actualizaciones con identidad incorrecta y detectar timestamps fuera de orden.
- Tras reconexión, recuperar desde REST una ventana solapada que cubra al menos la última vela cerrada y la vela en curso; reconciliar por open time y preferir el snapshot REST para velas cerradas si hay discrepancia, registrando la anomalía.
- Detectar gaps comparando aperturas esperadas para el intervalo. No inventar velas faltantes ni interpolar OHLCV. Solicitar backfill REST y exponer estado incompleto/error si no puede repararse.
- La semántica de intervalos mensuales y de zonas horarias debe estar definida por el adaptador; no calcular todos los intervalos como múltiplos fijos de milisegundos.

### 6. Concurrencia, backpressure y ciclo de vida

- Ninguna petición REST, conexión WebSocket, espera de red, backoff ni parseo costoso se ejecutará en el hilo de egui.
- Resolver #25 y #34 antes de integrar red. Evaluar worker thread + canales acotados frente a runtime async existente; no introducir Tokio ni otra dependencia hasta justificarla.
- El worker no compartirá mutablemente `ChartState`. Entregará mensajes/DTOs inmutables a la UI.
- Cada carga/subscripción tendrá un identificador de generación/sesión. Al cambiar instrumento, timeframe o mercado, cancelar la operación anterior o descartar toda respuesta cuyo identificador ya no sea vigente.
- Definir backpressure: coalescing de snapshots de una vela abierta por clave, preservando siempre el evento de cierre y los errores/estados de conexión. La cola tendrá capacidad acotada y métricas/logs de overflow.
- Cierre de ventana y cambio de instrumento deben cancelar y liberar workers/conexiones de forma determinista.
- Implementar timeout, reconexión con backoff exponencial acotado y jitter, y rotación preventiva de conexiones según límites oficiales (las sesiones WebSocket tienen duración limitada).

### 7. Rate limits y resiliencia

- Leer y respetar los pesos de endpoints y límites publicados por `exchangeInfo`/documentación para cada segmento; usar paginación y presupuesto de peticiones compartido por IP cuando corresponda.
- Tratar HTTP 429 como rate limit: detener solicitudes y respetar `Retry-After` cuando esté presente; tratar 418 como bloqueo temporal y no reintentar agresivamente.
- Aplicar backoff exponencial con jitter para fallos transitorios de red/servidor; no reintentar errores permanentes de símbolo/intervalo.
- WebSocket debe responder a ping/pong según el protocolo, controlar límites de mensajes de control/subscripción y rotar/reconectar antes de alcanzar el máximo de vida de la conexión.
- Los límites exactos (peso por límite de klines, mensajes por segundo, streams por conexión y duración de sesión) se verificarán en documentación vigente durante la implementación y se cubrirán con pruebas de configuración.

### 8. Configuración y bootstrap

- Mantener el mock como proveedor predeterminado para desarrollo offline y tests hasta que el adaptador real esté probado.
- Inyectar el proveedor desde bootstrap mediante configuración explícita: proveedor, segmento de mercado, símbolo e intervalo.
- Market data público no requiere API key ni firma. No introducir secretos ni variables de credenciales para la primera integración.
- No seleccionar automáticamente Spot/Futures por heurísticas del símbolo.
- La configuración debe permitir cambiar entre Mock, Binance Spot y Binance USDⓈ-M Futures sin que el dominio o la UI conozcan endpoints o payloads.

### 9. Pruebas

- Fixtures JSON versionados y reducidos para respuestas REST de Spot/Futures, eventos WebSocket de vela abierta/cerrada, metadatos y errores.
- Servidor HTTP/WebSocket simulado para pruebas deterministas de paginación, timeouts, 429/418, reconexión, mensajes duplicados/fuera de orden, gaps y cierre.
- Tests de contrato comunes ejecutados contra ambos adaptadores.
- Tests específicos por mercado para símbolos, intervalos, metadatos y formatos.
- Prohibido depender de Binance público en CI; pruebas live opcionales deben ser manuales/no bloqueantes.
- Probar que la UI permanece reactiva mientras el proveedor está lento o desconectado y que respuestas tardías de una generación anterior se descartan.

### 10. Alcance excluido

- Crear, cancelar o modificar órdenes.
- API keys, firmas, balances, posiciones, user data streams o gestión de cuentas.
- COIN-M Futures, Options, Margin y otros productos.
- Decisiones de estrategia, backtesting y ejecución algorítmica.
- Cambios de UI/workspace no necesarios para seleccionar mercado/proveedor.

La evolución algorítmica futura deberá construir sobre instrumentos, velas y series identificadas, datos históricos reproducibles y límites de riesgo separados. Esta ADR no habilita trading real.

## Alternativas consideradas

1. **Solo Spot:** menor complejidad inicial, pero no satisface la decisión de producto de soportar también derivados.
2. **Implementar Spot y USDⓈ-M Futures simultáneamente:** ofrece cobertura inmediata, pero duplica superficie de validación antes de probar el contrato común.
3. **Diseño común, Spot primero y USDⓈ-M después (elegida):** valida primero el flujo completo y luego añade diferencias de futuros con suites de contrato compartidas.
4. **Un SDK de terceros como abstracción principal:** se descarta por ahora; se prefiere controlar DTOs, semántica, límites y actualización de documentación con adaptadores pequeños detrás de puertos propios.

## Plan de implementación posterior

1. Resolver y aceptar las decisiones de ejecución/ciclo de vida de #25 y #34.
2. Evolucionar la identidad de instrumento/serie y la política de precisión sin filtrar tipos Binance al dominio.
3. Implementar adaptador Spot: exchangeInfo, klines REST, kline WebSocket, backfill y reconciliación.
4. Validar el flujo Spot con fixtures y servidor simulado; conservar Mock.
5. Implementar USDⓈ-M Futures reutilizando contratos, añadiendo metadatos y semánticas propias.
6. Añadir selección explícita de proveedor/mercado en bootstrap/configuración.
7. Solo después de estabilidad de market data, abrir trabajo de indicadores, paper trading, riesgo y herramientas algorítmicas.

## Consecuencias

### Positivas
- El producto podrá representar Spot y derivados sin confundir instrumentos con símbolos iguales.
- Se evita acoplar el dominio a Binance o a un SDK.
- La secuencia incremental reduce riesgo y mantiene el mock útil.
- La reconciliación y el control de identidad preparan el flujo para datos reproducibles y estrategias futuras.

### Costes y riesgos
- Se necesita ampliar el modelo actual de identidad de mercado/serie.
- Spot y USDⓈ-M Futures tienen diferencias que deben permanecer explícitas.
- La precisión actual basada en `f64` no es suficiente como representación financiera autoritativa.
- La integración depende de resolver primero I/O fuera del hilo UI y ciclo de vida.
- La documentación y los límites de Binance cambian; deben verificarse antes de cada implementación.

## Referencias oficiales

- [Binance Spot REST API](https://developers.binance.com/docs/binance-spot-api-docs/rest-api)
- [Binance Spot WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Binance Spot Market Data Only](https://github.com/binance/binance-spot-api-docs/blob/master/faqs/market_data_only.md)
- [Binance Spot Filters](https://github.com/binance/binance-spot-api-docs/blob/master/filters.md)
- [Binance USDⓈ-M Futures Market Data REST API](https://developers.binance.com/en/docs/catalog/core-trading-derivatives-trading-usd-s-m-futures/api/rest-api/market-data)
- [Binance USDⓈ-M Futures WebSocket Streams](https://developers.binance.com/en/docs/catalog/derivatives-trading-usds-margined-futures/websocket-market-streams)
- [Binance API General Information and Rate Limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information)
