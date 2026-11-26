# ADR-0003: WGPU como backend gráfico principal del desktop

## Estado

Aceptada

## Contexto

ApexTrader es una aplicación desktop para Windows, macOS y Linux. El motor gráfico de producto debe ser wgpu; Glow/OpenGL no es un backend alternativo de producto ni un fallback silencioso.

El estado inicial auditado no cumplía esa dirección:
- En el baseline, `Cargo.toml` habilitaba `eframe` con `glow`.
- En el baseline, `bootstrap::desktop::run` usaba las opciones nativas por defecto y no seleccionaba explícitamente el renderer.
- En el baseline, `renderer::create_renderer` creaba `GlowChartRenderer`.
- `ChartRenderer::draw` recibe `egui::Painter` y `egui::Rect`, por lo que el contrato de renderer está acoplado a la presentación.
- `WgpuChartRenderer` crea su propia instancia y device, limita los backends a DX12, no crea una surface y no presenta la escena.
- `WgpuPipeline` dibuja geometría de prueba y no proyecta la escena real del chart.

La aplicación ya utiliza eframe/egui para ventana, ciclo de eventos, controles y composición de UI. Mantener un segundo ciclo nativo independiente para el chart duplicaría ownership de ventana, surface, dispositivo, sincronización, resize y recuperación.

## Alternativas consideradas

### 1. eframe con renderer WGPU y callbacks personalizados

eframe conserva el ciclo nativo y la integración de ventana. Su renderer WGPU gestiona la instancia, adapter, device, queue y surface. El chart aporta pipelines, buffers y recursos propios mediante callbacks de egui-wgpu, integrados en el render pass de la UI.

### 2. Ventana y ciclo nativo propios con winit y wgpu

ApexTrader gestionaría directamente ventana, eventos, surface, adapter, device, queue, presentación, resize, suspensión y recuperación. Esta opción ofrece control completo, pero requiere reemplazar o separar la integración nativa de eframe y no justifica duplicar el ciclo de vida actual.

### 3. Mantener Glow como renderer principal o fallback

Se descarta porque contradice el requisito de wgpu como motor gráfico principal y mantiene dos implementaciones con semánticas, cachés y resultados potencialmente divergentes.

## Decisión

Se adopta **eframe con Renderer::Wgpu como backend gráfico principal** para la aplicación desktop.

1. eframe es owner del ciclo de ventana/eventos y de la instancia, adapter, device, queue y surface WGPU que utiliza su integración nativa.
2. El bootstrap selecciona explícitamente `eframe::Renderer::Wgpu`; las features de Cargo habilitan WGPU y las integraciones de ventana necesarias para Windows, macOS y Linux.
3. El chart se integra mediante callbacks personalizados de egui-wgpu. El callback prepara y actualiza recursos, y emite los comandos de dibujo en el pass coordinado por el renderer de eframe.
4. El subsistema de rendering es owner de sus pipelines, buffers, shaders, geometría derivada, cachés y políticas de invalidación. No crea una segunda instancia, device, queue ni surface.
5. La escena del chart y su geometría permanecen independientes de egui, eframe y wgpu. El adaptador de presentación convierte layout, clipping, DPI y coordenadas a los parámetros requeridos por el callback.
6. No existe fallback automático a Glow. Si la inicialización WGPU falla, el error se informa explícitamente; una futura modalidad de compatibilidad requeriría otra decisión documentada.
7. Las APIs directas de `winit`, `raw-window-handle` y `pollster` no se conservan por inercia. Se eliminan cuando la migración haya quitado todos sus consumidores reales. La dependencia directa de wgpu y bytemuck se mantiene si los pipelines propios la requieren.
8. El backend nativo debe habilitar las implementaciones apropiadas de cada plataforma. La matriz de CI verificará compilación en los tres sistemas; la ejecución real del renderer requiere validación adicional en hardware o runners gráficos adecuados.

## Plan de migración incremental

La migración se ejecuta en fases para preservar la UI y evitar que un cambio de backend se confunda con la implementación completa del renderer del chart.

### Fase 1: activar WGPU en eframe

- Estado: completada en el head `5afa0a2b1dab150414b625bc9af13018bf0828d0`.
- `Cargo.toml` habilita `wgpu`, `wayland` y `x11` en eframe; no habilita Glow ni `default_fonts`.
- `bootstrap::desktop` selecciona explícitamente `eframe::Renderer::Wgpu`.
- El renderer basado en `egui::Painter` fue renombrado a `EguiPainterChartRenderer`, reflejando su responsabilidad real y sin atribuirle Glow.
- `Cargo.lock` fue regenerado mediante `cargo check --workspace --all-targets` y quedó versionado. Se retiró `default_fonts` al detectar que añadía licencias no incluidas en la política vigente; no se amplió la allowlist.
- En esta fase, egui dibuja las primitivas del chart y eframe las presenta mediante WGPU. Esto activa WGPU como backend gráfico de la aplicación, pero todavía no constituye un renderer de chart con pipelines propios.
- No se habilita Glow como fallback.
- CI #321 pasó en Windows, macOS y Linux, incluidos build debug/release, tests, Clippy, documentación y auditoría de dependencias. La validación de runtime gráfico con hardware real sigue pendiente.

### Fase 2: renderer WGPU propio del chart

- Implementar un callback `egui-wgpu` que consuma una escena neutral y convierta geometría visible en buffers GPU.
- Inicializar y mantener pipelines y buffers en los recursos persistentes del callback, usando el device y queue proporcionados por eframe.
- Implementar velas, mechas, volumen y overlays con clipping y coordenadas físicas/lógicas correctas.
- Definir invalidación y actualización incremental de buffers sin reconstruir geometría completa en cada frame.
- Probar resize, cambios de DPI, clipping, pérdida/recreación del estado de dispositivo y escenas vacías.

### Fase 3: retirar infraestructura paralela

- Eliminar el renderer WGPU manual que crea su propia instancia, adapter y device, así como su surface stub y pipeline de geometría de prueba.
- Retirar dependencias directas de `winit`, `raw-window-handle`, `pollster` y tipos WGPU que ya no tengan consumidores reales.
- Mantener una dependencia directa de `wgpu` solo si el backend del chart la necesita para implementar callbacks y pipelines.
- Actualizar Cargo.lock mediante Cargo y verificar que una compilación con lockfile estricto sea reproducible.

### Evidencia por fase

La compilación multiplataforma valida integración y compatibilidad de tipos, no ejecución gráfica. Cada fase debe tener CI verde sobre su head. La Fase 2 requiere pruebas de geometría y callback; el soporte gráfico completo requiere ejecución real en Windows, macOS y Linux con validación de drivers y ciclo de vida.

## Consecuencias

### Positivas

- Un único owner del ciclo de vida de ventana y GPU.
- WGPU es el backend efectivo de UI y chart, no una implementación paralela inactiva.
- La escena del chart puede evolucionar sin depender del backend gráfico.
- Se evita duplicar surface, device, queue y mecanismos de presentación.
- El renderizado del chart puede compartir sincronización y presentación con egui.

### Costes y riesgos

- La integración de callbacks tiene requisitos de concurrencia y ciclo de vida: el callback implementa Send + Sync y los recursos GPU persistentes se alojan en los recursos del renderer.
- Los recursos deben reconstruirse cuando cambia el device o se invalida el estado asociado; no se deben retener handles obsoletos.
- La coordinación del render pass, clipping, DPI y espacios de coordenadas debe probarse explícitamente.
- El CI de compilación no demuestra rendimiento, compatibilidad de drivers ni funcionamiento en hardware real.
- La migración puede requerir reescribir el contrato actual de `ChartRenderer`; no debe simularse un backend neutral que siga exponiendo `egui::Painter`.

## Condiciones de aceptación

- eframe está compilado con WGPU y selecciona `Renderer::Wgpu` explícitamente.
- La aplicación no crea ni selecciona Glow para el chart.
- El chart real, no geometría de prueba, se presenta mediante un callback WGPU.
- La escena no contiene tipos de egui, eframe ni wgpu.
- Resize, clipping, DPI, invalidación y recreación de recursos tienen ownership y pruebas definidos.
- Los errores de inicialización o pérdida de recursos se propagan o registran de forma explícita; nunca se ocultan cambiando a Glow.
- CI compila en Windows, macOS y Linux.
- Se valida el renderer en runtime en al menos un entorno soportado por cada plataforma antes de declarar soporte gráfico completo.

## Condiciones de revisión

Revisar esta decisión si eframe deja de ofrecer control suficiente sobre el ciclo de renderizado, si la aplicación necesita un event loop nativo independiente, o si los requisitos de múltiples ventanas/superficies exigen ownership propio. La revisión debe comparar el coste total de ciclo de vida, mantenimiento, rendimiento y portabilidad; no basta con preferir una API de menor nivel.


## Fase 2 — preparación del renderer de chart mediante callbacks

La implementación comienza con la eliminación de la ruta WGPU manual e incompleta. El renderer de chart no debe crear una segunda instancia, adapter, device, queue, surface ni ciclo de vida de ventana.

### Ownership y ciclo de vida

- eframe/winit posee ventana, eventos, surface y ciclo de vida de la presentación.
- eframe/egui-wgpu posee instancia, adapter, device, queue, surface y render pass de egui.
- ApexTrader posee únicamente recursos específicos del chart: pipeline(s), buffers, layouts y datos de geometría.
- Los recursos GPU persistentes se registran de forma diferida en RenderState.renderer.callback_resources durante el primer prepare del callback, usando el device y el formato de destino suministrados por eframe.
- Cada callback recibe una instantánea inmutable y acotada de la geometría visible. No retiene referencias a estado mutable del dominio ni a datos de mercado.
- prepare actualiza buffers y recursos antes del render pass; paint solo enlaza recursos y emite draw calls en el render pass compartido de egui.
- El rectángulo del callback delimita el viewport lógico del chart; el clip rect de egui limita la región efectiva. Las conversiones entre puntos lógicos y píxeles físicos deben usar pixels_per_point.
- Resize y DPI no recrean device ni surface. Solo invalidan geometría dependiente del tamaño y actualizan uniformes/viewport.
- La pérdida o recreación del device invalida todos los recursos GPU del chart y requiere reconstrucción desde la escena neutral.

### Contrato de geometría

La geometría de chart debe ser independiente de wgpu y egui. Se construye desde la escena neutral y contiene primitivas trianguladas para mechas, cuerpos y volumen, con colores explícitos y coordenadas locales al rectángulo del chart. El backend convierte esa geometría a buffers GPU; no recalcula precios, escalas financieras ni identidad de velas.

La conversión a coordenadas NDC debe rechazar dimensiones no finitas o no positivas y evitar castings que produzcan NaN/Infinity. Los valores financieros conservan su representación canónica fuera del renderer; los floats se limitan a la presentación.

### Invalidación y rendimiento

- Cambios de cursor/crosshair no regeneran la geometría de serie.
- Cambios de datos regeneran únicamente la geometría de serie.
- Cambios de viewport, escala, layout, tamaño o DPI invalidan la geometría que depende de esas dimensiones.
- Los buffers mantienen capacidad y se amplían solo cuando sea necesario.
- Una geometría sin cambios no se vuelve a transferir a GPU.
- No se agregan draw calls por vela: cuerpos, mechas y volumen se agrupan por pipeline y estado compatible.
- Las optimizaciones se aceptan con mediciones; primero se preserva la equivalencia visual y determinismo.

### Evidencia y alcance

La eliminación de la ruta manual solo elimina código inoperante; no representa por sí misma un renderer WGPU de chart. La fase 2 se considera implementada cuando callbacks reales dibujen cuerpos, mechas y volumen usando recursos persistentes y existan tests de geometría/invalidación más CI multiplataforma verde. La ejecución con GPU/driver real sigue requiriendo validación de runtime.


## Estado de implementación de la Fase 2

El primer recorrido WGPU del chart está integrado en el head actual:

- La geometría neutral CPU genera triángulos para mechas, cuerpos y barras de volumen.
- `ChartGpuCallback` implementa `egui_wgpu::CallbackTrait` y consume un snapshot inmutable compartido mediante `Arc`.
- El shader WGSL y el pipeline usan el formato de destino que eframe expone en `CreationContext::wgpu_render_state`.
- `prepare` crea de forma diferida los recursos dentro de `CallbackResources`, amplía el buffer si hace falta y evita volver a subir un snapshot sin cambios.
- `paint` solo enlaza pipeline/buffer y ejecuta el draw call.
- La serie del chart utiliza el callback cuando hay target WGPU; paneles, ejes, etiquetas y crosshair siguen en egui.
- Se añadieron pruebas de conversión clip-space, dimensiones inválidas y coordenadas no finitas.

Esta implementación todavía requiere CI verde sobre el head que la contiene y validación de runtime en Windows, macOS y Linux con hardware/driver real. Resize/DPI, recreación de device, múltiples viewports y recuperación de errores GPU necesitan pruebas dedicadas antes de declarar la fase completa.
