# Mapa arquitectónico inicial

## Alcance y estado

Este documento registra el mapa estático inicial del repositorio sobre la rama `chore/engineering-governance-foundation`. Es una fotografía del código inspeccionado, no una arquitectura objetivo ni una certificación de calidad. La auditoría continúa por flujos y módulos; las responsabilidades inferidas deben contrastarse con comportamiento y pruebas antes de refactorizar.

Inventario obtenido del árbol Git: 165 archivos versionados, de los cuales 51 pertenecen a `src/`, 10 a `tests/` y 1 a `benches/`. El repositorio también contiene documentación, automatización, configuración de agentes, tareas y assets.

## Mapa de módulos actuales

| Módulo | Responsabilidad observada | Dependencias y observaciones |
|---|---|---|
| `src/main.rs` | Punto de entrada binario | Delegación directa a `apex_trader::application::run()` |
| `src/application/commands` | Casos de uso de carga/ingestión de velas | `LoadCandles` recibe un `MarketDataProvider` definido en `market`; la aplicación conoce la abstracción del subsistema de mercado |
| `src/application/events` | Traducción de input de egui a eventos del controlador de interacción | Importa `eframe::egui`, `ChartLayout` y `ChartState`; es un adaptador de UI alojado dentro de application |
| `src/application/services` | Arranque, UI, estado de pantalla, carga, realtime, escala, invalidación y composición de renderer/proveedores | Archivo de 865 líneas; implementa `eframe::App` y crea directamente proveedores mock |
| `src/chart/coordinate.rs`, `crosshair.rs`, `hit_testing`, `interaction`, `layout`, `visible_range` | Geometría, interacción, hit testing y layout del gráfico | El subsistema de interacción es mayormente independiente del framework en controller/events/options, pero el adaptador `application/events` depende de egui |
| `src/domain` | Tipos de vela, precio, cantidad, tiempo, estado del gráfico, viewport y transformaciones | El verificador actual prohíbe algunas dependencias de infraestructura aquí; la independencia debe verificarse con grafo completo |
| `src/market` | Modelo de vela de mercado, símbolo, errores, contratos de proveedores y proveedores mock históricos/realtime | Los contratos de proveedor viven en este módulo y son consumidos por application |
| `src/renderer` | Escena/representación intermedia, backend Glow y backend wgpu | `ChartRenderer` recibe `egui::Painter` y `egui::Rect`; el contrato no es neutral respecto de UI |
| `src/errors` | Errores de dominio/renderer | Separado por tipo de error |
| `src/logging` | Inicialización de tracing | Inicialización invocada desde la composición de aplicación |
| `tests` | Pruebas de arquitectura, dominio, transformaciones, escalas, interacción y propiedades | Debe contrastarse cobertura con riesgos y casos límite |
| `benches` | Benchmark de transformaciones del gráfico | Un benchmark identificado |

## Concentración de responsabilidades en application/services

La inspección de `src/application/services/mod.rs` confirma 865 líneas y los siguientes elementos de primer nivel:

- `LoadState`: estado de carga presentado por la UI.
- `ApexTraderApp`: estado de pantalla, dependencias de renderer/input, estado del viewport, control de seguimiento, reloj de realtime y rectángulo anterior.
- `eframe::App::ui`: composición de controles egui, modificación del viewport, entrada de símbolo, geometría de layout, procesamiento de input, autoscale, cálculo de invalidación, construcción de escena, render y footer de diagnóstico.
- `advance_realtime`: temporización, polling de proveedor realtime, conversión de vela, mutación de ChartState y solicitud de repaint.
- `calculate_invalidation_mask`: política de invalidación visual ligada a egui::Rect.
- `run`: inicialización de logging, ventana nativa, fuentes, estado inicial, proveedores mock y renderer.
- Funciones auxiliares: tick size por símbolo, timestamp de última vela cerrada, construcción del proveedor realtime, carga/fallback histórico, fuentes, compatibilidad de escalas y rangos de precios.
- Pruebas unitarias locales de tick size y autoscale.

Responsabilidades que hoy comparten el mismo módulo:

1. Adaptador de presentación egui/eframe.
2. Estado y coordinación del ciclo de vida de la pantalla.
3. Orquestación de carga histórica y política de fallback.
4. Composición de proveedores mock históricos y realtime.
5. Temporización y aplicación de actualizaciones realtime.
6. Política de viewport/autoscale y seguimiento de última vela.
7. Política de invalidación y preparación de escena.
8. Selección y composición del backend de renderizado.
9. Configuración de fuentes y logging.

La extracción debe conservar primero el comportamiento observable y las pruebas. No conviene separar por número de líneas: los límites candidatos son adaptador de presentación, coordinador/caso de uso de carga, sesión realtime, política de viewport y composición de dependencias.

## Flujo de datos observado

`MockMarketDataProvider` → `LoadCandles::execute` → `LoadCandlesResult` → conversión a puntos del gráfico → `ChartState` → cálculo de rango/viewport/visibilidad → `ChartScene` → `ChartRenderer::draw` → backend Glow.

El flujo realtime actual es:

`MockRealtimeMarketDataProvider::next_update` → `MarketCandle::to_chart_data_point` → `ChartState::add_data_point` → flag de seguimiento → siguiente ciclo de UI/repaint → recálculo de escena e invalidación.

Ambos flujos están coordinados directamente por `ApexTraderApp`. El proveedor histórico y el realtime son instanciados concretamente en el mismo módulo que implementa la UI. El fallback histórico también crea otro proveedor mock concreto.

## Dependencias y límites detectados

- `application/mod.rs` expone `run() -> eframe::Result`, por lo que el límite superior de application está acoplado al framework de presentación.
- `application/events` importa egui para traducir input; es un adaptador tecnológico, no lógica de aplicación neutral.
- `application/services` importa tipos concretos de mock de `market`, además de traits de proveedor.
- `application/commands/load_candles.rs` importa `MarketDataProvider` desde `market`; todavía no hay un puerto de entrada/salida propio de application para el caso de uso.
- `renderer::ChartRenderer` recibe tipos egui en su API pública. El renderer no puede sustituir el framework de UI sin adaptar su contrato.
- `renderer/ir.rs` contiene una representación intermedia y geometría de renderizado, pero también usa tipos de `chart::layout`; conviene evaluar si esa geometría es un modelo compartido o pertenece a un contrato neutral de render.
- `src/lib.rs` mantiene `#![allow(dead_code)]` global, reduciendo la señal de código sin uso durante la compilación.
- `Cargo.toml` reúne dependencias de UI, GPU, ventana, handles nativos y utilidades de async en un único paquete. No es prueba suficiente para crear crates; primero se medirán el grafo real, tiempos de compilación y fronteras de uso.

## Dirección objetivo provisional

La dirección a validar es una arquitectura de capas con dependencias hacia el núcleo:

- **Domain:** invariantes, tipos y reglas de negocio sin frameworks, IO ni adaptadores.
- **Application:** casos de uso y puertos requeridos, sin depender de egui/eframe, proveedores mock ni backends concretos.
- **Adapters:** proveedores de mercado, persistencia, reloj, logging y presentación.
- **Presentation:** adaptador egui/eframe que traduce interacción y estado de pantalla a comandos/consultas.
- **Rendering:** contrato neutral de escena/comandos/geometría; backend Glow y backend GPU implementan el contrato sin que la API pública dependa de egui.

La separación física en crates se decidirá únicamente si existen límites de compilación, despliegue, ownership o dependencias que la justifiquen.

## Próximas verificaciones

1. Construir el grafo de imports completo y detectar ciclos y dependencias no permitidas.
2. Rastrear todos los usos de `ApexTraderApp`, `LoadCandles`, `MarketDataProvider`, `ChartScene` y `ChartRenderer`.
3. Revisar invariantes y consistencia temporal de ingestión histórica y realtime, incluyendo huecos, duplicados, intervalos, velas parciales y cambios de símbolo.
4. Auditar precisión numérica y conversiones entre `f64`, `Decimal` y tipos del dominio.
5. Revisar ownership, sincronización y backpressure al incorporar proveedores realtime reales.
6. Obtener baseline reproducible de formato, compilación, Clippy, tests, documentación, seguridad y rendimiento.

## Validación de CI

La ejecución CI #259 se creó para el commit `b6dfc4b44f6fb8670f51a750edfbf187b4a6fcea` con evento `pull_request`. En la última consulta su estado era `queued` y aún no exponía jobs; no hay resultado de formato, compilación, lint ni tests confirmado.
