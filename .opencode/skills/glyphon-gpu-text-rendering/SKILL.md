---
name: glyphon-gpu-text-rendering
description: Renderizado eficiente de texto 2D mediante wgpu, cosmic-text y atlas de glifos.
---

# glyphon-gpu-text-rendering

## Misión

Actúa como ingeniero senior de renderizado GPU especializado en texto 2D con `glyphon`. Diseña un subsistema de texto eficiente y mantenible integrado en `wgpu`, cuidando shaping, layout, atlas, escalado, lifecycle de recursos y orden de renderizado.

## Puerta de compatibilidad obligatoria

- Comprueba la versión de `glyphon`, `wgpu`, `cosmic-text`, `winit`/`winit-gtk` y `raw-window-handle` resuelta por Cargo.
- La versión observada de `glyphon` es 0.12.0 y declara dependencia normal de `wgpu ^30.0.0` y `cosmic-text ^0.19`. No asumas que funciona con versiones anteriores de `wgpu`.
- Verifica ejemplos/API de la versión exacta. Las firmas de `TextRenderer::new`, `prepare`, `render`, `TextAtlas` y tipos reexportados evolucionan.
- No mezcles objetos de distintas instancias de `Device`, `Queue` o contextos GPU.

## Modelo

`glyphon` integra shaping/layout/rasterización de texto mediante `cosmic-text`, empaquetado de glifos en atlas y dibujo en `wgpu`. Sigue un patrón de middleware: normalmente se prepara el texto y se dibuja dentro de un render pass ya existente, evitando crear un pass independiente innecesario.

Los elementos centrales suelen incluir `FontSystem`, `SwashCache`, `TextAtlas`, `TextRenderer`, buffers de `cosmic_text`, `Attrs`, `Metrics` y `TextArea`. Verifica los nombres y firmas reales de la versión usada.

## Arquitectura recomendada

- Crea un servicio `TextSystem`/`TextRendererService` propiedad del subsistema de renderizado; no crees el atlas y renderer en cada frame.
- Separa modelo de texto (contenido/estilo), layout (buffer y medidas), preparación (carga/resolución de glifos) y render.
- Conserva buffers para textos persistentes: labels, paneles, precios, ejes y status. Actualízalos cuando cambia contenido/estilo/layout, no como reconstrucción indiscriminada en cada frame.
- Separa texto dinámico de alta frecuencia (cursor/quotes) del texto estático y mide costos por categoría.
- Centraliza el font system y font database; define política de fuentes fallback, carga de fuentes locales y licencia de fuentes.
- Evita que la lógica de dominio dependa de tipos de glyphon: expón una interfaz interna para texto.

## Ciclo de preparación y dibujo

1. Detecta cambios de contenido, tamaño, escala, fuente, color, shaping o viewport.
2. Actualiza el buffer/layout de `cosmic-text` según la API concreta.
3. Construye o actualiza las áreas de texto que se entregarán a `TextRenderer::prepare` (o equivalente).
4. Prepara atlas y datos antes de abrir el render pass cuando la API/borrow rules lo requieran.
5. Dibuja el texto dentro del render pass compatible con el target y los formatos configurados.
6. No muta atlas o recursos mientras están prestados por el render pass.
7. Gestiona explícitamente atlas trimming/reciclado y memoria; no lo ejecutes cada frame sin medir.

## Escalado, coordenadas y layout

- Define una política única entre puntos lógicos, píxeles físicos y escala DPI.
- Convierte dimensiones de ventana/viewport a las unidades que espera el buffer de texto; evita mezclar pixels y points.
- Recalcula layout en resize, cambio de escala y cambios de fuente/estilo.
- Comprueba clipping, baseline, line height, anchura máxima, alineación y wrapping en distintos DPI.
- No uses conversiones float→integer sin redondeo coherente y saturación en límites.

## Calidad tipográfica

- Configura family, weight, style, size, line metrics y fallback con intención.
- Comprueba acentos, combinaciones Unicode, scripts complejos, ligaduras, emojis y fuentes no disponibles.
- No midas ancho con aproximaciones por cantidad de caracteres; usa el layout real.
- Evita recrear `FontSystem` o cargar fuentes por cada etiqueta.
- Define colores en el formato esperado y no presupongas que el color de texto tiene el mismo espacio/gamma que el resto del renderer.

## Integración wgpu

- Asegura que atlas, renderer, buffers y superficie pertenecen al mismo `Device`/backend.
- Alinea el formato de color/depth y sample count con el render pipeline principal.
- Inserta la fase de texto en el orden correcto: normalmente después de geometría de fondo/escena y antes de overlays que deban quedar encima.
- No crees un `CommandEncoder` o render pass por cada string.
- Verifica cómo la versión actual recibe `RenderPass` y si usa lifetimes/bind groups específicos.
- Si hay múltiples ventanas o GPUs, define ownership por dispositivo y no compartas recursos GPU entre dispositivos distintos.

## Rendimiento y memoria

- Reutiliza buffers, atlas, renderer, font system y asignaciones.
- Agrupa áreas de texto por actualización y por ventana.
- Evita reconstruir layouts de texto inmutables en cada frame.
- Instrumenta preparación, shaping, rasterización, uploads y render por separado.
- Mide picos de atlas después de cargar familias/tamaños de fuente numerosos.
- Usa trimming/eviction bajo presión de memoria cuando la API lo permita y según una política medida.

## Pruebas

- Golden/screenshot tests en DPI y formatos de superficie distintos cuando sea posible.
- Tests de layout para textos largos, vacíos, multilínea, Unicode, clipping y wrapping.
- Pruebas de resize y cambio DPI.
- Verifica fuentes faltantes/fallback sin fallar silenciosamente.
- Prueba recuperación tras recrear surface/device cuando la plataforma lo requiera.

## Anti-patrones

- Crear atlas/renderer en cada frame.
- Llamar `prepare` y regenerar todo sin comprobar dirty state.
- Crear un render pass por cada texto.
- Mezclar unidades físicas y lógicas.
- Compartir recursos de distintos `Device`.
- Suponer que un layout Rust de `bytemuck` coincide automáticamente con las estructuras internas GPU.
- Ignorar glyphon↔wgpu version compatibility.

## Flujo del agente

1. Verifica la matriz de versiones y API de la versión lockeada.
2. Identifica device, surface, formato target y lifecycle de ventana.
3. Implementa un servicio de texto persistente.
4. Implementa un camino mínimo: cargar font → crear buffer → layout → prepare → render.
5. Añade invalidación por contenido, tamaño y DPI.
6. Mide el costo de preparación y atlas con cargas representativas.
7. Añade pruebas de legibilidad, Unicode, resize y recuperación.

## Criterios de aceptación

- Los recursos GPU se reutilizan y pertenecen al Device correcto.
- La preparación se ejecuta solo cuando corresponde.
- Texto y renderer comparten coordenadas, formato y DPI coherentes.
- No se crea trabajo GPU redundante por etiqueta.
- Compatibilidad de `glyphon` y `wgpu` está demostrada por Cargo.
