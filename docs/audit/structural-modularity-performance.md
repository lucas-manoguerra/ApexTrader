# Auditoría de estructura, modularidad y rendimiento

## Estado y alcance

Auditoría estática del árbol Git en `chore/engineering-governance-foundation`, commit `d9b825f04b8fe8b15fbb874a24760ff4e51eee35`. Se revisaron el árbol completo, manifiesto Cargo, módulos públicos, límites arquitectónicos, documentación, el módulo de composición de UI, el controlador de interacción, el modelo de viewport, el estado del gráfico, el IR de renderizado, el backend Glow, mocks y tests de arquitectura.

Este documento distingue:
- Hechos observados en el árbol o el código.
- Riesgos estructurales que requieren validación dinámica.
- Dirección objetivo y propuestas de reorganización.
- Cambios que no deben ejecutarse hasta tener pruebas y ownership definidos.

No es una certificación de rendimiento. No se dispone todavía de perfiles ni benchmarks suficientes para declarar cuellos de botella.

## Inventario del árbol

En el commit inspeccionado, el árbol contiene 169 archivos y 52 directorios (221 entradas Git):
- `src/`: 51 archivos.
- `tests/`: 10 archivos.
- `benches/`: 1 archivo.
- `docs/`: documentación de arquitectura, ingeniería, calidad, auditoría y decisiones.
- `.opencode/skills/`: recursos y referencias de ingeniería para agentes.
- `odd/tasks/`: especificaciones y tareas históricas/operativas.
- `.atl/`, `.github/`, `scripts/`, `assets/` y configuración raíz.

El inventario anterior de 165 archivos quedó desactualizado al evolucionar la rama; el mapa arquitectónico debe mantener una fecha/commit de referencia y actualizarse con cada auditoría estructural significativa.

## Hallazgos estructurales

### ST-001 — El punto de composición de la aplicación concentra demasiadas políticas

**Severidad:** crítica.

**Evidencia:** `src/application/services/mod.rs`, aproximadamente 32.7 KB y documentado previamente como 865 líneas.

El mismo archivo contiene:
- Implementación de `eframe::App` y composición de widgets.
- Estado de pantalla y coordinación del ciclo de vida.
- Selección y validación de símbolos.
- Construcción directa de proveedores mock históricos y realtime.
- Política de carga, fallback y mutación de `ChartState`.
- Temporización y aplicación de actualizaciones realtime.
- Cálculo de rangos y comportamiento de autoscale.
- Política de invalidación, preparación de escena y llamada al renderer.
- Inicialización de logging, fuentes y ventana nativa.
- Funciones auxiliares y tests locales de varias políticas.

El problema no es el tamaño aislado: son responsabilidades con razones de cambio distintas y dependencias de UI, mercado, dominio y renderer en el mismo módulo.

**Dirección:** convertir el arranque en un composition root pequeño; mover el adaptador eframe a presentación; separar políticas puras de viewport/escala; mantener carga y realtime como casos de uso coordinados por contratos; aislar mocks en modo demo/simulación explícito. No crear un módulo genérico `services` que vuelva a concentrar todo.

### ST-002 — El límite application incluye adaptadores de presentación y depende de contratos de mercado concretos

**Severidad:** alta.

**Evidencia:** `src/application/mod.rs` expone `run() -> eframe::Result`; `src/application/events/mod.rs` importa egui y `ChartState`; `src/application/services/mod.rs` importa tipos mock concretos y los traits de mercado.

**Impacto:** la capa de aplicación no puede ejecutarse como núcleo independiente del framework y la UI instancia detalles de infraestructura. El traductor de input es un adaptador de presentación, no un caso de uso.

**Dirección:** crear un límite de presentación explícito; definir puertos de aplicación para las operaciones que los casos de uso requieren; ubicar adaptadores egui/eframe fuera de application; ubicar composición de proveedores mock en un bootstrap de demo. Los puertos deben reflejar necesidades de casos de uso, no duplicar mecánicamente todos los traits existentes.

### ST-003 — El subsistema market mezcla contratos, modelo normalizado y mocks

**Severidad:** alta.

**Evidencia:** `src/market/` contiene `candle.rs`, `symbol.rs`, `errors.rs`, `provider.rs`, `mock.rs` y `mock_realtime.rs`.

**Impacto:** al integrar proveedores reales será fácil mezclar formato externo, normalización, identidad de instrumento, validación, política de simulación y contratos de aplicación. Los mocks aparecen como módulos de primer nivel indistinguibles de una implementación de producción.

**Dirección:** separar el modelo canónico de mercado de los adaptadores de proveedor. Ubicar mocks en un submódulo `testing` o `simulation` con activación explícita. Mantener en el núcleo de mercado solo tipos y contratos que tengan ownership claro. Los formatos crudos de cada proveedor no deben escapar de su adaptador.

### ST-004 — El contrato de renderer y su geometría no tienen todavía una frontera totalmente neutral

**Severidad:** alta.

**Evidencia:** `src/renderer/mod.rs` declara `ChartRenderer::draw` con `egui::Painter` y `egui::Rect`; `src/renderer/ir.rs` define `ChartScene`, `RenderFrame`, capas y geometría, pero importa `ChartLayout` desde `chart`.

**Impacto:** el contrato del backend depende de UI y el IR depende de un módulo que también representa layout del chart. Las fronteras entre layout de interacción, layout visual y geometría de backend no están claramente expresadas.

**Dirección:** distinguir layout lógico del chart, escena visual y adaptadores de backend. Definir un contrato de render neutral solo si los consumidores reales (Glow y WGPU) pueden compartirlo sin introducir conversiones redundantes. El adaptador Glow puede recibir painter/rect externos y traducir comandos neutrales. No crear una abstracción de GPU que sea solo una copia de egui.

### ST-005 — Archivos grandes con responsabilidades internas que merecen revisión por cohesión

**Severidad:** media-alta; requiere análisis de ownership antes de dividir.

| Archivo | Tamaño aproximado | Observación |
|---|---:|---|
| `src/chart/interaction/controller.rs` | 41 KB | Controlador con estados de gesto y múltiples políticas de pan/zoom/escala; tests unitarios amplios conviven con implementación |
| `src/renderer/glow.rs` | 45.7 KB | Backend con ticks, formato de ejes, cache, construcción de shapes y dibujo de superficies/ejes |
| `src/domain/viewport.rs` | 39 KB | Estado de viewport, escalas lineal/log/porcentual, zoom, pan, fitting, límites y tests |
| `src/domain/chart_state.rs` | 15 KB | Estado de serie, mutaciones, índices temporales, viewport y revisión de datos |
| `src/domain/chart/time_scale.rs` | 13.4 KB | Escala temporal, interpolación, rangos y formato de etiquetas |

No se propone dividir automáticamente por tamaño o cantidad de funciones. Los candidatos de extracción deben representar conceptos independientes con invariantes y pruebas propias. Por ejemplo, el formato de ticks podría extraerse de Glow si es una política reutilizable y no una abstracción artificial; el viewport solo debe dividirse si sus políticas de escala y navegación pueden tener ownership independiente.

### ST-006 — Los tests no reflejan todavía la estructura de subsistemas

**Severidad:** media.

**Evidencia:** `tests/` contiene diez archivos mayormente planos; combina nombres con guiones y guiones bajos, y agrupa pruebas de chart, dominio, mercado y arquitectura en el mismo nivel. Hay tests locales dentro de módulos de producción, incluidos archivos grandes.

**Impacto:** cuesta descubrir la suite correspondiente a un subsistema, entender fixtures compartidos y medir cobertura por responsabilidad.

**Dirección:** mantener unit tests junto al módulo cuando prueban detalles privados; organizar integration tests por capacidades y límites públicos. Usar nombres consistentes en snake_case y fixtures compartidos solo cuando exista semántica común. Separar pruebas de arquitectura, dominio financiero, market data, chart, renderer y rendimiento. No mover tests sin revisar visibilidad y alcance de cada prueba.

### ST-007 — La documentación de trabajo y la documentación normativa necesitan taxonomía explícita

**Severidad:** media.

**Evidencia:** `docs/` tiene categorías claras, pero `odd/tasks/` aloja especificaciones de producto/ingeniería bajo una abreviatura no explicada en el índice documental. La raíz también contiene `chat-gpt.md`, recursos `.atl/` y `.opencode/` con propósitos distintos.

**Impacto:** un nuevo contribuyente o agente puede confundir tareas históricas, especificaciones vigentes, estándares normativos y material de referencia.

**Dirección:** definir el propósito y ciclo de vida de ODD en documentación; decidir si las tareas activas pertenecen a `docs/specifications/`, `docs/roadmap/` o un sistema de issues. Mantener recursos propios de herramientas en sus directorios dedicados. No borrar ni mover `chat-gpt.md`, `odd/` o recursos de agentes sin confirmar si son inputs activos del usuario/proyecto.

### ST-008 — La API pública de módulos es más abierta de lo necesario

**Severidad:** media-alta.

**Evidencia:** `src/lib.rs` publica todos los módulos superiores; los módulos declaran muchos submódulos como `pub mod`. El verificador de arquitectura actual solo comprueba imports de crates prohibidos dentro de `domain`.

**Impacto:** detalles internos se convierten en API accidental; mover archivos o separar módulos genera cambios de compatibilidad innecesarios. Las pruebas de arquitectura no detectan dependencias entre capas ni ciclos.

**Dirección:** usar visibilidad privada por defecto, exponer APIs deliberadas mediante reexports mínimos y mantener implementaciones internas como `mod` o `pub(crate)`. Extender las reglas de arquitectura con una matriz de dependencias y pruebas del verificador. No reducir visibilidad hasta identificar consumidores internos/externos y ajustar tests.

## Dirección de estructura objetivo

Esta es una guía de ownership, no una instrucción para crear todos los directorios inmediatamente:

```text
src/
  main.rs
  lib.rs
  bootstrap/
    desktop.rs
  domain/
    instrument/
      instrument.rs
      identifiers.rs
      specification.rs
    market_data/
      candle.rs
      series.rs
      timeframe.rs
      quality.rs
    financial/
      price.rs
      quantity.rs
      numeric_policy.rs
    chart/
      chart_state.rs
      time_scale.rs
      viewport.rs
  application/
    ports/
      historical_market_data.rs
      realtime_market_data.rs
      clock.rs
    use_cases/
      load_series.rs
      apply_market_update.rs
    models/
  market_data/
    normalization/
    providers/
    simulation/
  chart/
    geometry/
    interaction/
    layout/
  presentation/
    desktop/
      app.rs
      panels/
      input/
  rendering/
    scene/
    cache/
    backends/
      glow/
      wgpu/
  infrastructure/
    clock/
    logging/
```

Reglas para adaptar esta propuesta:
- No crear carpetas de un solo archivo salvo que expresen un límite conceptual estable o permitan crecer en una dirección concreta.
- No mantener simultáneamente dos definiciones del mismo concepto en `domain`, `market_data` y `chart`; cada tipo tiene un owner y adaptadores explícitos.
- El dominio financiero no depende del chart ni del renderer.
- El chart consume modelos/proyecciones aptos para visualización; no se convierte en autoridad financiera.
- Los puertos viven hacia el interior; los adaptadores concretos viven hacia el exterior.
- `bootstrap` conecta implementaciones concretas; no contiene reglas de negocio.
- Los nombres describen conceptos y capacidades, no tecnologías salvo en adaptadores/backend.

## Política de archivos y abstracciones

1. Un archivo debe tener una razón principal de cambio.
2. El tamaño es una señal para investigar, no una regla automática de división.
3. Preferir nombres de dominio concretos a nombres vagos como `helpers`, `common`, `utils`, `services` o `manager`.
4. Una abstracción debe eliminar duplicación real, aislar una variación prevista, proteger una invariante o hacer posible una prueba aislada.
5. No crear traits por cada struct. Preferir tipos concretos y funciones simples cuando solo existe una implementación y no hay frontera de sustitución necesaria.
6. Usar enums para conjuntos cerrados de comportamientos; traits para extensiones abiertas o dependencias intercambiables.
7. Evitar wrappers que solo reenvían llamadas, capas de DTO sin transformación/contrato y factories sin variación real.
8. Usar newtypes para identidad, unidades e invariantes; no envolver tipos por estética.
9. Mantener funciones pequeñas por cohesión y legibilidad, no por un límite rígido de líneas.
10. La API pública es explícita y mínima; la visibilidad privada es el valor por defecto.
11. Los módulos de reexportación exponen una superficie coherente; no se usan como directorios de conveniencia.
12. Cero comentarios en código sigue siendo una regla global; la intención se expresa con nombres, tipos y documentación Markdown.

## Rendimiento y uso de recursos

### Buenas prácticas observadas

- `ChartScene` toma prestado `&[ChartDataPoint]` en lugar de clonar la historia.
- `RenderLayer` reserva capacidad para velas y barras visibles.
- `GlowChartRenderer` mantiene caché de layer y shapes y dispone de tests para reutilización e invalidación.
- El estado del gráfico mantiene una revisión de datos y tests que verifican que actualizar la última vela no reconstruye innecesariamente la escala temporal.

Estas decisiones deben conservarse y medirse durante cualquier refactor.

### Oportunidades que requieren medición

- El ciclo de UI calcula rangos de precio visibles en más de una etapa y calcula volumen máximo sobre el rango visible. Evaluar una única pasada de agregación que produzca min/max y max_volume para el mismo snapshot, evitando cálculos duplicados cuando la semántica de autoscale lo permita.
- Medir asignaciones de escena, labels, strings y shapes por frame; conservar caches solo cuando la invalidación sea correcta y su coste de mantenimiento menor que la recomputación.
- Medir coste del controlador de interacción y viewport en escenarios con series grandes e irregularidad temporal.
- Medir la escala temporal en append, corrección de vela, inserción fuera de orden y reconstrucción.
- Evaluar memoria retenida por histórico, backfill y buffers realtime; definir límites, retención y backpressure antes de conectar feeds reales.
- Separar tiempo de CPU, tiempo de GPU, latencia de ingestión y latencia de presentación.
- Medir tiempo de compilación y tamaño del árbol de dependencias antes de dividir crates o cambiar features.

### Reglas de optimización

- Establecer benchmarks reproducibles con tamaños representativos (p. ej. 1k, 10k, 100k y 1M puntos cuando la capacidad objetivo lo justifique).
- Medir latencia p50/p95/p99 para ingestión y actualización realtime; medir frame time y asignaciones para rendering.
- Comparar antes/después en build release y misma máquina/toolchain.
- Mantener tests de equivalencia para que una optimización no cambie resultados numéricos, orden temporal, visibilidad o invalidación.
- Evitar clones de colecciones completas; preferir borrowing, slices, iteradores y ownership explícito.
- Evitar recalcular derivaciones si existe una revisión/versionado confiable para invalidar caché.
- No introducir paralelismo hasta que el perfil muestre trabajo independiente suficiente y el coste de coordinación esté justificado.
- No optimizar con pérdida de precisión financiera ni suavizar errores de integridad.

## Secuencia de refactor propuesta

1. **Mapa de dependencias y ownership:** generar imports/ciclos, registrar dependencias permitidas, identificar consumidores de cada API pública.
2. **Composition root y presentación:** aislar eframe/egui y sacar mocks de la UI sin alterar comportamiento visible.
3. **Puertos y casos de uso:** definir contratos mínimos para histórico/realtime/reloj y probarlos sin UI.
4. **Modelo canónico de mercado:** identidad, serie, timeframe, calidad y representación numérica, según los ADR pendientes.
5. **Chart y rendering:** separar layout/escena/backend cuando exista contrato común demostrado.
6. **Tests y benchmarks:** reflejar la estructura por subsistema, añadir fixtures y baseline de memoria/latencia/frame time.
7. **Crates/features:** evaluar solo después de probar límites estables, beneficios de compilación y ownership independiente.

Cada etapa debe ser un PR/commit coherente, con tests, CI y revisión del impacto. No hacer una reorganización masiva de carpetas en una sola operación.

## Criterios de aceptación estructural

- Cada módulo y carpeta tiene propósito, owner y dirección de dependencia documentados.
- Ningún módulo de aplicación depende de egui/eframe ni de mocks concretos.
- Ningún tipo de proveedor externo se filtra al modelo canónico.
- El dominio financiero no depende de chart, UI, renderer ni infraestructura.
- Los contratos de renderer no dependen de tipos de egui.
- Los imports no permitidos y ciclos son detectados por CI.
- La API pública se limita a contratos intencionales.
- Las pruebas están organizadas por comportamiento y límite.
- El rendimiento cuenta con benchmarks y perfiles reproducibles.
- La política de cero comentarios se mantiene en todos los cambios.

## Decisiones abiertas

- Definir si `odd/tasks` es un backlog vigente, archivo histórico o documentación de especificaciones.
- Identificar los mercados y proveedores iniciales para fijar ownership de instrumentos/series.
- Definir qué backend es objetivo de producto y cuál permanece experimental.
- Definir objetivos cuantitativos de memoria, latencia y frame time según el hardware y tamaño de series soportados.
