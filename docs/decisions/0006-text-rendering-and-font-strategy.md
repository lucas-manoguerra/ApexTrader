# ADR-0006: Estrategia de tipografía y renderizado de texto

## Estado

Propuesta aceptada para orientar la implementación de tipografía. La selección definitiva de archivos de fuentes queda pendiente de validar cobertura, licencia, tamaño y distribución.

## Contexto

ApexTrader utiliza eframe/egui 0.37 con WGPU 30. El renderer del gráfico dibuja velas, mechas y volumen mediante un callback WGPU integrado en el render pass coordinado por eframe. Ejes, etiquetas, crosshair y controles continúan utilizando egui.

La aplicación mostró glifos tofu en Ubuntu incluso después de retirar la selección de fuentes mediante rutas absolutas de Windows. La causa confirmada es que eframe se compila con `default-features = false` y no habilita `default_fonts`. La ADR-0003 retiró esa feature porque incorpora Ubuntu Light bajo Ubuntu Font Licence 1.0, que no está permitida por la política actual de licencias.

La solución debe garantizar fuentes redistribuibles, reproducibles y disponibles en instalaciones limpias. La biblioteca de renderizado de texto y los archivos tipográficos son responsabilidades distintas: adoptar un renderer GPU no proporciona por sí mismo fuentes ni garantiza su cobertura Unicode.

La revisión de egui 0.37 confirmó que `eframe` expone la feature `default_fonts`, que activa las fuentes embebidas de epaint. `FontDefinitions::default()` queda vacío cuando esa feature no está activa; cuando sí lo está, registra Ubuntu Light para la familia proporcional, Hack para monospace y los iconos propios de egui. El paquete `epaint_default_fonts` declara `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0`. Ubuntu Font Licence 1.0 permite embedding y redistribución bajo sus condiciones, incluida la entrega de avisos y texto de licencia. Por tanto, la licencia no es una incompatibilidad intrínseca: requiere una aceptación explícita en la política de dependencias y una distribución correcta de notices.

## Alternativas consideradas

### 1. egui/epaint con fuentes redistribuibles explícitas

egui mantiene la responsabilidad del texto de interfaz y de las etiquetas actuales del gráfico. Las fuentes se incorporan explícitamente y se registran mediante `FontDefinitions`, con familias proporcional y monoespaciada configuradas de forma deliberada.

La primera opción a probar es habilitar `eframe/default_fonts`, que utiliza los recursos de `epaint_default_fonts` ya alineados con la versión de egui del proyecto. Esto proporciona Ubuntu Light para texto proporcional, Hack para monospace y los iconos propios de egui, sin incorporar archivos TTF manualmente. La alternativa de incluir fuentes propias (por ejemplo Noto Sans y Noto Sans Mono) permanece disponible si la cobertura o la apariencia de las fuentes integradas no satisface los criterios.

Ventajas:
- Conserva el modelo actual de UI y el ownership del texto.
- No agrega un segundo subsistema de renderizado GPU.
- Las fuentes son embebidas por la dependencia y no dependen del sistema operativo.
- Reutiliza los recursos y la configuración tipográfica previstos por egui.
- Permite resolver primero el defecto visual sin mezclarlo con una migración del renderer.

Costes:
- La selección, redistribución y atribución de las fuentes son responsabilidad del proyecto.
- La política de licencias debe permitir explícitamente `OFL-1.1` y `Ubuntu-font-1.0`, según la metadata de `epaint_default_fonts`.
- Deben incluirse los avisos y textos de licencia de las fuentes efectivamente embebidas.
- La cobertura y el fallback deben verificarse explícitamente.
- La configuración debe cubrir controles egui y texto dibujado por `egui::Painter`.

### 2. glyphon

glyphon 0.12 utiliza cosmic-text para shaping, layout y rasterización, etagere para el atlas de glifos y wgpu para dibujar en un render pass existente. Su dependencia wgpu 30 coincide con la línea principal usada por ApexTrader.

Ventajas:
- Soporta shaping y layout de texto más completos mediante cosmic-text.
- Gestiona atlas y caché de glifos.
- Puede integrarse en el render pass existente sin crear otra instancia, device, queue o surface.
- Es el candidato más adecuado si el texto del gráfico requiere capacidades o rendimiento que egui no cubre.

Costes:
- Introduce otro subsistema de layout, caché, recursos GPU e invalidación.
- Requiere definir con precisión ownership y sincronización dentro del callback WGPU.
- No sustituye la necesidad de suministrar y configurar fuentes redistribuibles.
- Puede duplicar responsabilidades si se usa para todos los controles de egui.

### 3. wgpu_text

wgpu_text 30.0.1 es un wrapper sobre glyph-brush y wgpu 30.

Ventajas:
- API directa para dibujar texto en WGPU.
- Acepta fuentes TTF/OTF suministradas por la aplicación.
- Su línea 30 coincide con la versión principal de wgpu del proyecto.

Costes:
- Añade glyph-brush y su modelo de composición/caché.
- No ofrece la misma integración de shaping avanzado basada en cosmic-text que glyphon.
- Duplica el subsistema de texto para una necesidad que egui ya cubre actualmente.

Se conserva como alternativa para un caso de texto simple que justifique una prueba comparativa; no es la opción preferida.

### 4. Vello

Vello es un renderer vectorial 2D general, no una biblioteca específica para resolver la cobertura tipográfica de egui o para añadir etiquetas a un chart WGPU existente.

Ventajas:
- Puede ser pertinente si ApexTrader decide adoptar un renderer vectorial de escena completo.
- Abarca problemas de composición y dibujo más amplios que el texto.

Costes:
- Cambiaría el alcance desde tipografía hacia la arquitectura de renderizado de toda la escena.
- Introduce solapamiento con el renderer WGPU de chart y con la composición de egui.
- No está justificado por el defecto actual de glifos.

No se incorpora para el manejo aislado de texto.

### 5. Fuentes embebidas mediante una dependencia de assets

Se evaluaron paquetes que distribuyen Noto con licencia SIL Open Font License 1.1. `oxifont-bundled` 0.2.2 ofrece Noto Sans y Noto Sans Mono como datos embebidos opcionales, pero está diseñado para el ecosistema OxiFont e incorpora dependencias `oxifont-core` y `oxifont-parser`. `notosans` 0.1.0 expone bytes TTF de Noto Sans, pero no ofrece una familia monoespaciada equivalente y su publicación es antigua.

No se adopta una dependencia de fuentes hasta comparar el coste completo de dependencias, tamaño del binario, API, mantenimiento y avisos de licencia frente a incorporar directamente los archivos tipográficos necesarios al repositorio.

## Decisión

1. egui/epaint seguirá siendo el renderer de texto de la UI y de las etiquetas actuales del gráfico.
2. La corrección de la Issue #27 probará primero `eframe/default_fonts`, con aceptación explícita de `OFL-1.1` y `Ubuntu-font-1.0` en `cargo-deny.toml` y distribución de los avisos/licencias requeridos por las fuentes incluidas. La prueba debe confirmar cobertura y legibilidad de todos los caracteres de aceptación en Ubuntu, Windows y macOS. Si las fuentes de egui no cumplen, se evaluarán archivos externos explícitos (por ejemplo Noto Sans y Noto Sans Mono) con sus licencias y atribuciones.
3. La configuración tipográfica será centralizada y se realizará una sola vez durante la inicialización de la UI. No se priorizarán fuentes del sistema ni rutas absolutas específicas de plataforma; el objetivo es una misma cadena de fuentes embebidas en todos los sistemas.
4. Se configurarán las familias proporcional y monoespaciada con fuentes adecuadas; no se asignará una fuente proporcional a la familia monoespaciada sin validar las consecuencias visuales.
5. La verificación cubrirá caracteres españoles, puntuación, signos, números, decimales y símbolos financieros usados realmente por la aplicación. También cubrirá los textos dibujados mediante `egui::Painter`.
6. glyphon es el único candidato aprobado para un prototipo técnico posterior, limitado inicialmente al texto del gráfico. No se agrega como dependencia de producción en esta fase.
7. Un prototipo de glyphon reutilizará exclusivamente los recursos WGPU proporcionados por eframe y se integrará mediante el callback existente. No creará una instancia, adapter, device, queue, surface ni ciclo de presentación independiente.
8. wgpu_text no se incorpora salvo que una medición o requisito concreto justifique su API más simple frente a glyphon.
9. Vello queda fuera de alcance mientras ApexTrader no evalúe reemplazar el renderer vectorial de la escena completa.
10. Ninguna decisión de renderer de texto puede introducir dependencias desde el dominio o la aplicación hacia egui, wgpu o las bibliotecas de presentación.

## Plan de validación

### Corrección de fuentes

- Verificar la cobertura real de cada archivo tipográfico elegido.
- Conservar y auditar los avisos SIL OFL 1.1 y la información de copyright.
- Verificar que las fuentes se cargan desde el binario y no desde rutas del sistema.
- Probar controles egui, ejes, etiquetas y crosshair en una instalación limpia.
- Mantener CI de Windows y Ubuntu; restaurar macOS cuando GitHub estabilice sus runners.
- Realizar comprobación visual real en Windows, macOS y Linux antes de cerrar la Issue #27.

### Prototipo glyphon

- Aislarlo en una PR independiente y sin migrar los controles de egui.
- Probar shaping, fallback, alineación, clipping, DPI y resize.
- Medir tiempo de preparación, tiempo de render, memoria del atlas, número de actualizaciones y coste de caché.
- Comparar contra egui/epaint usando exactamente los mismos textos, tamaños y fuentes.
- Probar invalidación y recreación de recursos GPU sin retener handles obsoletos.
- Mantenerlo solo si existe una mejora medible o una capacidad necesaria que egui no proporcione.

## Consecuencias

### Positivas

- La corrección del problema de fuentes no queda acoplada a una migración del renderer.
- Se conserva un único owner del ciclo de vida WGPU.
- Se evita duplicar el renderizado de controles de egui.
- glyphon queda disponible como evolución incremental y medible para etiquetas densas del chart.
- La distribución de fuentes es explícita y reproducible en instalaciones limpias.

### Costes y riesgos

- El proyecto debe mantener avisos de licencia y verificar los archivos de fuentes distribuidos.
- Las fuentes embebidas aumentan el tamaño del ejecutable.
- Un prototipo glyphon añade complejidad de recursos, caché y ciclo de vida que debe justificarse con mediciones.
- La compilación multiplataforma no sustituye la validación visual ni la ejecución en hardware real.

## Condiciones para revisar la decisión

Revisar la elección de egui/epaint para las etiquetas si profiling demuestra un cuello de botella de texto, si se requieren capacidades avanzadas de shaping/fallback no cubiertas por egui o si la densidad de etiquetas del chart exige un control de caché y atlas más especializado.

Revisar Vello únicamente ante una decisión explícita de evolucionar el renderer vectorial de la escena completa, con una comparación de coste total, integración, portabilidad y rendimiento.


## Estado de implementación (2026-11-28)

La selección inicial descrita en la decisión se encuentra implementada en `main`:

- `Cargo.toml` activa `eframe/default_fonts` manteniendo `default-features = false`.
- `FontDefinitions::default()` registra Ubuntu Light para proporcional, Hack para monospace y los iconos de egui 0.37.
- `src/presentation/desktop/fonts.rs::configure` aplica las definiciones centralmente durante la inicialización de `ApexTraderApp`.
- `licenses/THIRD_PARTY_FONTS.md` contiene los notices y textos de licencia pertinentes; `cargo-deny.toml` permite explícitamente las licencias declaradas por los recursos embebidos.
- CI tiene matrices Windows, macOS y Ubuntu. Esta cobertura de compilación/pruebas no equivale a una validación visual sobre instalaciones limpias.

La prueba automatizada de cobertura de glifos de la Issue #27 se está incorporando separadamente: debe verificar las familias efectivamente instaladas en el contexto de egui y no limitarse a comprobar los nombres registrados en `FontDefinitions`. La aceptación final de la Issue sigue condicionada a evidencia visual en los tres sistemas y a confirmar que los textos de controles y los emitidos por `egui::Painter` usan el contexto configurado.

La selección de fuentes por defecto es, por tanto, la implementación inicial adoptada, no una garantía universal de cobertura Unicode: si los glifos requeridos no están presentes, la solución deberá incorporar una fuente/fallback redistribuible adicional y sus avisos correspondientes.
