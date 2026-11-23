# Conectar datos de mercado reales — Etapa 4

Objetivo: incorporar una capa de datos de mercado que provea velas (OHLCV)
reales al chart engine, separando la obtención de datos del renderizado y del
estado del gráfico. Sigue el flujo MarketDataProvider → Chart State → Chart
Engine → Renderer.

Branch: `feature/conectar-datos-mercado` (se crea al iniciar la primera tarea).
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (proveedor de mercado): `MarketDataProvider` trait con método para obtener
velas de un símbolo en un rango. Implementación mock/sintética para pruebas sin red.

S2 (símbolo): tipo `Symbol` que represente un par de trading (ej. BTC/USDT),
validado y serializable.

S3 (vela con timestamp): `MarketCandle` o extensión de `Candle` con timestamp
y símbolo, para ordenar y dedup velas del mercado.

S4 (ingestión en ChartState): `ChartState` recibe velas de un proveedor y las
almacena, ordenadas por tiempo, sin duplicados.

S5 (errores de mercado): errores tipados para red, auth, rate limit, símbolo
inválido; sin panic ante fallos de red.

S6 (comandos de aplicación): comandos para cargar datos de un símbolo y
mostrar estado de carga en la UI.

## Tasks

- T1: Crear módulo `market/` con `Symbol`, `MarketCandle` y trait
  `MarketDataProvider`. Route: inline. Status: done. Commit: 9c36532.
- T2: Implementar proveedor mock/sintético (`MockMarketDataProvider`) para
  pruebas sin red. Route: inline. Status: done. Commit: 9c36532 (incluido en T1).
- T3: Conectar `ChartState` con método para ingestar velas de un proveedor,
  ordenadas y sin duplicados (S4). Route: inline. Status: done. Commit: f8669b4.
  `ChartState::add_candles(&[Candle])` agrega múltiples velas. El
  ordenamiento/dedup por timestamp se hace en la capa de aplicación con
  `MarketCandle` (evita dependencia circular domain → market).
- T4: Errores tipados de mercado (red, auth, rate limit, símbolo inválido)
  sin panic (S5). Route: inline. Status: done. Commit: 9c36532 (MarketError
  en src/market/errors.rs).
- T5: Comandos de aplicación para cargar datos de un símbolo (S6). Route:
  inline. Status: done. Commit: 66b6114.
  `LoadCandles::execute(provider, symbol, from_ms, to_ms)` obtiene velas
  de un `MarketDataProvider`, las convierte a `domain::Candle` y las devuelve
  ordenadas, listas para `ChartState::add_candles`.
- T6: Pruebas de propiedades para ordenamiento, dedup y validación de
  MarketCandle. Route: inline. Status: done. Commit: 0aed654.
  `tests/market_proptest.rs` con 4 pruebas de propiedades (is_valid,
  to_domain_candle, ordenamiento, dedup de timestamps).
- T7: Verificación final: cargo test --workspace (79 tests), clippy -D
  warnings, cargo fmt --check, tests/architecture.rs todos verdes.
  Status: done. Etapa 4 completa.

## Log

L1: Etapa 4 aprobada por el usuario tras completar la Etapa 3 (rama
feature/profesionalizar-chart-engine mergeada vía PR #2).


## Extensión: histórico sintético y stream realtime

La capa mock se amplía para probar el ciclo completo de actualización del chart
sin conectar un exchange real.

- T8: Reemplazar la curva sinusoidal simplista por un histórico determinista con
  tendencias de varias escalas, volatilidad variable, mechas asimétricas y
  volumen correlacionado con el retorno absoluto. Route: inline. Status: done
  (pendiente de CI). Archivo: src/market/mock.rs.
- T9: Crear el contrato RealtimeMarketDataProvider y una implementación mock
  que emita snapshots OHLCV de la vela activa, indique su cierre y abra velas
  contiguas conservando continuidad de precio y tiempo. Route: inline.
  Status: done (pendiente de CI). Archivos: src/market/provider.rs y
  src/market/mock_realtime.rs.
- T10: Conectar el stream a la UI con un reloj de simulación, actualizaciones
  periódicas y control de seguimiento de la última vela. El seguimiento solo
  mueve el eje X si el usuario permanece cerca del borde derecho. Route: inline.
  Status: done (pendiente de CI). Archivo: src/application/services/mod.rs.
- T11: Optimizar la ingestión incremental: reemplazar una vela existente sin
  reconstruir TimeScale y anexar timestamps crecientes en O(1) amortizado.
  Route: inline. Status: done (pendiente de CI). Archivos:
  src/domain/chart_state.rs y src/domain/chart/time_scale.rs.

Criterios de aceptación:
- Las consultas históricas repetidas y los rangos solapados generan exactamente
  las mismas velas.
- Cada vela conserva OHLC válido, timestamps alineados y continuidad open/close.
- El volumen varía y aumenta estadísticamente con los movimientos absolutos fuertes.
- El stream actualiza la misma vela hasta cerrarla; el siguiente timestamp empieza
  en el cierre de la anterior y abre al último precio.
- El histórico y el realtime se integran mediante el mismo ChartState.
- Los ticks de una vela activa no reconstruyen el histórico de TimeScale.
- El seguimiento realtime no interfiere con el pan/zoom manual cuando el usuario
  abandona el extremo derecho.
