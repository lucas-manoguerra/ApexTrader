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
  inline. Status: done. Commit: (pendiente).
  `LoadCandles::execute(provider, symbol, from_ms, to_ms)` obtiene velas
  de un `MarketDataProvider`, las convierte a `domain::Candle` y las devuelve
  ordenadas, listas para `ChartState::add_candles`.
- T6: Pruebas de propiedades para ordenamiento, dedup y validación de
  MarketCandle. Route: inline. Status: pending.
- T7: Verificación final: cargo test --workspace, clippy -D warnings, cargo
  fmt, verify_architecture. Route: inline. Status: pending.

## Log

L1: Etapa 4 aprobada por el usuario tras completar la Etapa 3 (rama
feature/profesionalizar-chart-engine mergeada vía PR #2).
