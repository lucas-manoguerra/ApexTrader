# Integrar datos de mercado en la UI del chart — Etapa 5

Objetivo: que la aplicación de escritorio cargue velas de mercado desde un
`MockMarketDataProvider` a través del comando `LoadCandles` y las muestre en el
renderer glow, reemplazando las velas sintéticas hardcodeadas en
`services/mod.rs`. Sigue el flujo:
`MockMarketDataProvider` → `LoadCandles` → `ChartState::add_candles` → `Renderer`.

Branch: `feature/integrar-datos-mercado-ui` (se crea al iniciar la primera tarea).
Delivery strategy: `ask-on-risk` (default), sin push ni PR hasta que el usuario lo decida.
Test runner: `cargo test --workspace`

## Specs

S1 (carga automática al iniciar): la aplicación carga velas del proveedor mock
para un símbolo por defecto (ej. BTC/USDT) al iniciar y las muestra en el chart.

S2 (símbolo configurable): el símbolo se puede cambiar desde la UI (campo de texto
o selector) y se recargan los datos.

S3 (estado de carga): la UI muestra si los datos se están cargando, se cargaron o
hubo un error (sin panic ante fallos del proveedor).

S4 (velas en pantalla): el renderer glow dibuja las velas cargadas desde el
proveedor, usando el eje Y para precio y el eje X para tiempo/índice.

S5 (auto_scale tras carga): después de cargar velas, el viewport aplica
auto_scale para que todas las velas quepan con margen 5%.

## Tasks

- T1: Reemplazar las velas sintéticas hardcodeadas en `services/mod.rs` por una
  carga a través de `LoadCandles` + `MockMarketDataProvider` al iniciar.
  Route: inline. Status: done. Commit: 0076717.
  `services/mod.rs` ahora crea un `MockMarketDataProvider`, llama
  `LoadCandles::execute`, agrega las velas a `ChartState` y aplica
  `auto_scale`. Incluye fallback a velas sintéticas si falla el proveedor.
- T2: Mostrar estado de carga / error en la UI (S3). Route: inline. Status: done. Commit: 436a0c5.
  `ApexTraderApp` tiene un campo `load_state: LoadState` (Idle/Loading/Loaded/Error).
  `services/mod.rs` lo establece al cargar (`Loaded` con count o `Error` con mensaje).
  La UI muestra `Carga de mercado: {:?}` con el estado actual.
- T3: Permitir cambiar el símbolo desde la UI y recargar (S2). Route: inline.
  Status: done. Commit: (pendiente).
  `ApexTraderApp` tiene `symbol_input: String`. La UI muestra un campo de texto
  y botón "Recargar" (Enter también dispara la recarga). Al cambiar el símbolo,
  se limpia el chart, se cargan las nuevas velas vía `load_symbol_into_chart` y
  se aplica `auto_scale`. Errores de símbolo inválido se muestran en `load_state`.
- T4: Llamar `auto_scale` tras cargar velas para que quepan en pantalla (S5).
  Route: inline. Status: done. Commit: 0076717 (ya incluido en T1).
- T5: Pruebas de integración para carga end-to-end (provider → command →
  ChartState → renderer dibuja). Route: inline. Status: done. Commit: (pendiente).
  `tests/market_proptest.rs` agrega `test_carga_end_to_end_btc_usdt` (mock →
  LoadCandles → ChartState → auto_scale) y
  `test_cambiar_simbolo_no_acumula_velas` (clear + recarga de símbolo distinto).
- T6: Verificación final: cargo test --workspace, clippy -D warnings, cargo fmt.
  Route: inline. Status: pending.

## Log

L1: Etapa 5 aprobada por el usuario tras completar la Etapa 4 (rama
feature/conectar-datos-mercado mergeada vía PR #3).
L2: T1 completado y commiteado. services/mod.rs ahora carga velas desde
MockMarketDataProvider a través de LoadCandles, las agrega a ChartState y
aplica auto_scale. Incluye fallback a velas sintéticas si falla el proveedor.
L3: T2 completado y commiteado. ApexTraderApp tiene campo load_state: LoadState
(Idle/Loading/Loaded/Error). La UI muestra "Carga de mercado: {:?}"
con el estado actual. Fallback a velas sintéticas si falla el proveedor.
L4: T3 completado y commiteado. Agregado symbol_input a ApexTraderApp. La UI
muestra campo de texto y botón "Recargar" (Enter también dispara la recarga).
Al cambiar el símbolo, se limpia el chart, se cargan las nuevas velas vía
load_symbol_into_chart y se aplica auto_scale. Agregado ChartState::clear() y
función helper load_symbol_into_chart().
L5: T5 completado y commiteado. Pruebas de integración end-to-end:
test_carga_end_to_end_btc_usdt y test_cambiar_simbolo_no_acumula_velas en
tests/market_proptest.rs.
