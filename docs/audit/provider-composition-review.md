# Auditoría de composición de proveedores y ownership

## Alcance y referencia

Revisión estática del código en la rama `chore/engineering-governance-foundation`, HEAD `ce7a020f84544dd025d546f69b122298b8376a6f`. Se inspeccionaron el bootstrap desktop, la aplicación de presentación, el caso de uso de carga histórica, los contratos y mocks de mercado, y los documentos de ownership y dependencias.

Esta revisión registra el comportamiento existente y fija límites de destino. No modifica código Rust ni declara validada una arquitectura que todavía no está implementada.

## Estado observado

### PC-001 — El bootstrap no compone las dependencias de la aplicación

**Evidencia:**
- `src/bootstrap/desktop.rs` configura `eframe::Renderer::Wgpu` y construye `ApexTraderApp::new(creation_context)`.
- No recibe ni construye dependencias de mercado, reloj o servicios de aplicación.
- `ApexTraderApp::new` crea directamente `MockMarketDataProvider`, carga histórico y crea `MockRealtimeMarketDataProvider`.

**Impacto:** el bootstrap actual solo inicia el runtime de escritorio. La composición concreta de proveedores está desplazada a la presentación, por lo que el punto de entrada no puede seleccionar de forma explícita una configuración de simulación o una configuración conectada.

**Dirección:** el composition root debe construir la configuración concreta y entregarla a la aplicación de presentación mediante dependencias explícitas. La presentación no debe elegir ni instanciar proveedores concretos.

### PC-002 — La presentación construye proveedores concretos durante la interacción

**Evidencia:**
- `src/presentation/desktop/app.rs` importa `MockMarketDataProvider`, `MockRealtimeMarketDataProvider`, `MarketDataProvider` y `RealtimeMarketDataProvider`.
- La inicialización de `ApexTraderApp` crea un mock histórico.
- La acción de recargar símbolo crea otro `MockMarketDataProvider`, calcula el rango temporal y ejecuta `load_symbol_into_chart`.
- `create_realtime_provider` reconstruye un `MarketCandle` desde el último `ChartDataPoint`, deriva el cierre nominal y crea el mock realtime.
- `advance_realtime` consulta el proveedor, convierte la vela a `ChartDataPoint` y muta `ChartState`.

**Impacto:** la presentación asume decisiones de infraestructura, composición, continuidad histórico/realtime, temporización y proyección de datos. La sustitución de simulación por un proveedor real obligaría a modificar código de UI y aumenta el riesgo de que una selección visual determine el origen de los datos.

**Dirección:** la presentación debe emitir una intención de carga y consumir resultados/estado observable. La selección de modo y la construcción de adapters pertenecen al composition root. La coordinación histórica/realtime y sus políticas de continuidad deben pertenecer a una capacidad de aplicación explícita, no a widgets ni al renderer.

### PC-003 — El caso de uso depende de contratos alojados en market

**Evidencia:**
- `src/application/commands/load_candles.rs` importa `MarketDataProvider`, `MarketError` y `Symbol` desde `crate::market`.
- `MarketDataProvider` está definido en `src/market/provider.rs`.
- El caso de uso devuelve `ChartDataPoint` y llama `MarketCandle::to_chart_data_point()`.

**Impacto:** el contrato requerido por el caso de uso no tiene ownership en application y el resultado del caso de uso queda acoplado a una representación de presentación/chart. La separación de capas es nominal, no completa.

**Dirección:** definir únicamente el puerto mínimo que el caso de uso necesite bajo ownership de application, sin duplicar traits por reflejo. El resultado de aplicación debe expresar datos normalizados y contexto de ingestión, no un tipo de dibujo. La proyección a chart debe tener un mapper explícito en el límite que consume esos datos.

### PC-004 — Los mocks están publicados como módulos ordinarios de market

**Evidencia:**
- `src/market/mod.rs` expone públicamente `mock` y `mock_realtime`.
- Los mocks implementan los contratos de `market/provider.rs`.
- La presentación importa y construye directamente ambos mocks.

**Impacto:** el árbol no expresa que esos adapters son exclusivamente simulación/testing, y permite que consumidores de producción dependan de ellos accidentalmente.

**Dirección:** aislar los adapters simulados en un módulo de simulación o testing, con visibilidad y activación explícitas. No definir todavía un crate separado: el límite de módulo es suficiente hasta que haya evidencia de independencia de compilación, despliegue o ownership.

### PC-005 — El estado del chart y la coordinación de datos están concentrados en la aplicación de presentación

**Evidencia:**
- `ApexTraderApp` posee `ChartState`, renderer, input handler, estado de carga y proveedor realtime.
- La misma implementación realiza mutaciones de chart, autoscale, invalidación, interacción, render y temporización realtime.
- La carga histórica vacía el chart ante error y evita publicar datos sintéticos, comportamiento cubierto por `failed_market_load_does_not_publish_synthetic_data`.

**Impacto:** el estado visual del chart es también el lugar donde se coordinan y retienen datos de mercado. Esto dificulta aislar las políticas de ingestión, continuidad, correcciones y reconexión de las políticas de viewport y render.

**Dirección:** mantener el estado de viewport e interacción bajo ownership de chart/presentation según su naturaleza visual. La serie canónica y su ciclo de vida deben tener un owner de aplicación/mercado claramente definido. El chart recibe una proyección; no decide la autoridad, continuidad o reconciliación de mercado.

## Límite objetivo de composición

1. **Bootstrap/composition root:** selecciona explícitamente el modo de ejecución y construye los adapters concretos requeridos. El modo Simulation debe ser explícito y visible; Connected solo podrá habilitarse cuando exista un adapter real configurado. No se debe inferir un fallback.
2. **Presentation:** posee widgets, input egui, estado de interacción y proyección visual. Recibe capacidades ya compuestas y no importa mocks ni SDKs de proveedores.
3. **Application:** coordina casos de uso, política de carga y ciclo histórico/realtime, continuidad, errores y publicación. Define puertos estrechos según las operaciones que realmente necesita.
4. **Market/domain:** conserva los modelos normalizados, identidad, unidades e invariantes de mercado. No depende de chart ni de UI.
5. **Simulation adapters:** generan datos deterministas y cumplen los mismos contratos requeridos por application; no se usan como recuperación silenciosa de fallos reales.
6. **Chart/rendering:** consume datos proyectados y mantiene sus responsabilidades visuales. No crea proveedores ni reconstruye datos de mercado desde puntos dibujados.

El flujo conceptual es:

```text
bootstrap / composition root
    |-- selects Simulation or Connected
    |-- constructs provider and clock adapters
    v
application use cases and lifecycle orchestration
    |-- application-owned ports
    |-- normalized market data and quality context
    v
presentation projection --> chart state --> rendering backend
```

Este flujo no obliga a crear un módulo por cada caja ni a extraer crates. Los límites deben materializarse en el código existente mediante cambios pequeños y pruebas de comportamiento.

## Secuencia de remediación recomendada

1. Introducir una configuración de ejecución explícita y composición de adapters en bootstrap, manteniendo únicamente Simulation disponible hasta que exista un proveedor real.
2. Pasar las dependencias compuestas a presentación y eliminar de ella las construcciones de mocks.
3. Definir el puerto histórico mínimo bajo application y adaptar el proveedor existente sin duplicar tipos de mercado.
4. Separar el resultado normalizado de la proyección `ChartDataPoint`.
5. Mover la coordinación de histórico/realtime fuera de la UI, una vez definidos identidad de serie, continuidad y lifecycle según los contratos de datos ya documentados.
6. Reducir la superficie pública de mocks y adapters después de revisar consumidores y tests.
7. Añadir enforcement de dependencias por capas cuando las fronteras hayan sido materializadas.

Cada paso debe ser acotado, conservar el comportamiento observable y contar con tests. No combinar esta refactorización con cambios numéricos financieros ni con una reorganización masiva del árbol.


## Remediación incremental aplicada

En el HEAD posterior a esta auditoría se aplicó un primer paso de composición:

- `bootstrap::desktop::run` construye el proveedor histórico de simulación y la factory del proveedor realtime.
- `ApexTraderApp::new` recibe ambos como dependencias inyectadas.
- La recarga de símbolo reutiliza el proveedor histórico inyectado y la factory realtime inyectada.
- `presentation::desktop::app` ya no importa ni instancia `MockMarketDataProvider` ni `MockRealtimeMarketDataProvider`; tampoco contiene configuración específica de esos mocks.
- Los parámetros específicos de simulación (precio base, intervalo, volumen y ticks por vela) están concentrados en bootstrap.
- El backend WGPU permanece seleccionado explícitamente en bootstrap.

Este paso mueve la construcción de adapters concretos al composition root, pero **no completa todavía** la separación de capas:

- Presentation aún depende de los traits definidos en `market::provider` y del modelo `MarketCandle`.
- Presentation conserva la política de rango histórico por defecto y la coordinación de carga/realtime.
- El último punto de `ChartState` todavía se reconstruye como `MarketCandle` para inicializar realtime; esto mantiene el riesgo de perder lifecycle/provenance identificado en la auditoría realtime.
- La factory realtime es un contrato de inyección transitorio declarado junto a la aplicación desktop, no el puerto definitivo de Application.
- Solo existe composición Simulation; no se anuncia ni se simula un modo Connected.

La CI debe validar el cambio antes de continuar. No se considera esta extracción una solución al ownership de histórico/realtime ni a la continuidad de serie.

## Criterios de salida

- Presentation no importa ni construye proveedores concretos.
- Bootstrap es el único punto que selecciona y construye adapters concretos.
- Application depende de puertos propios y no de egui/eframe ni de mocks.
- Los resultados de application no están acoplados a `ChartDataPoint`.
- La simulación se selecciona explícitamente y se identifica como tal.
- Un error del proveedor Connected nunca activa datos sintéticos.
- La continuidad histórico/realtime se valida con identidad de serie y lifecycle, no reconstruyendo autoridad desde el último punto del chart.
- Tests de carga fallida, orden, duplicados, correcciones y transición histórico/realtime permanecen verdes.
- No se crean crates ni abstracciones sin una necesidad demostrada.

## Relación con auditorías existentes

Esta revisión profundiza ST-001, ST-002 y ST-003 de `structural-modularity-performance.md`, y DG-002, DG-005, DG-006 y DG-007 de `dependency-graph.md`. Los hallazgos anteriores deben considerarse refinados por esta evidencia actual; no se marcan como resueltos por el mero hecho de que el bootstrap desktop ya exista.
