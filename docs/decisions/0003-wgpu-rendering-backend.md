# ADR-0003: WGPU como backend gráfico principal del desktop

## Estado

Aceptada

## Contexto

ApexTrader es una aplicación desktop para Windows, macOS y Linux. El motor gráfico de producto debe ser wgpu; Glow/OpenGL no es un backend alternativo de producto ni un fallback silencioso.

El estado actual no cumple esa dirección:
- `Cargo.toml` habilita `eframe` con `glow`.
- `bootstrap::desktop::run` usa las opciones nativas por defecto y no selecciona explícitamente el renderer.
- `renderer::create_renderer` crea `GlowChartRenderer`.
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
