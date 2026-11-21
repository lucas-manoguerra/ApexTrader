---
name: egui-immediate-mode-ui
description: Diseño de interfaces immediate-mode, estado, layout, interacción, accesibilidad y rendimiento.
---

# egui-immediate-mode-ui

## Misión

Actúa como ingeniero senior de UI Rust especializado en `egui`. Construye interfaces inmediatas rápidas, coherentes, accesibles y extensibles, separando el estado de presentación del dominio y entendiendo que la UI se reconstruye repetidamente.

## Puerta de compatibilidad

- Comprueba la versión de `egui` y crates del ecosistema (`epaint`, `emath`, `egui-winit`, `egui-wgpu`, `eframe`) en `Cargo.lock`.
- La versión observada de docs.rs es 0.37.0 y la documentación advierte que el proyecto evoluciona con cambios que pueden romper compatibilidad; declara Rust 1.95.0 como requisito en la documentación actual. Confirma la versión real para tu toolchain.
- Revisa las feature flags, que pueden no ser iguales a las del framework que integra `egui`.
- Verifica nombres y firmas de paneles, ventanas, viewports, widgets y métodos en la versión bloqueada.

## Modelo immediate-mode

Cada frame se vuelve a ejecutar la descripción de la interfaz usando un `Context` (`ctx`) y construyendo `Ui` mediante `Panel`, `Window`, menús y contenedores. Los widgets generan `Response` con información de interacción. No almacenes objetos widget esperando que sobrevivan de un frame al siguiente; almacena los datos de la aplicación y deja que egui reconstruya la representación.

## Arquitectura del estado

- Divide estado en: dominio (verdad del producto), presentación (selecciones/expansiones temporales), preferencias persistentes y estado efímero de interacción.
- Las funciones de UI deben recibir estado explícito y modificar solo la parte autorizada.
- No hagas que un ID visual sea una identidad de negocio.
- Usa IDs estables y únicos para widgets dinámicos, listas, ventanas y elementos repetidos; resuelve colisiones cuando se reordenen o inserten elementos.
- Guarda estados complejos fuera del árbol UI si deben sobrevivir a cambios de pantalla o a reinicios.
- Usa `Context` para repaint/temporización y estado temporal solo cuando sea apropiado; evita usarlo como base de datos global.

## Composición y layout

- Organiza ventanas y paneles según jerarquía de interacción; evita anidar contenedores sin razón.
- Usa layouts y tamaños relativos al espacio disponible antes que coordenadas absolutas.
- Comprueba la unidad de medidas: egui trabaja con puntos lógicos y aplica escalado; no mezcles puntos con píxeles físicos.
- Diseña para resize, DPI, fuentes grandes, traducciones y textos largos.
- Utiliza `Response` para detectar `clicked`, `changed`, hover, focus y drag según el API actual.
- Mantén controles con labels claros, feedback visible, keyboard navigation y estados disabled/loading.

## Coste por frame

- La construcción de UI debe ser ligera. No hagas parsing, consultas de red, lecturas de disco, generación pesada ni bloqueos dentro del closure de UI.
- Calcula snapshots o resultados antes del frame y pásalos a los componentes.
- Evita formatear grandes volúmenes de datos sin cambios; cachea datos derivados con invalidación explícita.
- En tablas/listas grandes, usa virtualización/recorte cuando el API lo soporte y el tamaño lo justifique.
- No clones buffers grandes o estructuras profundas en cada frame.
- Solicita repaint cuando haya cambios o animaciones; no fuerces repaint continuo sin motivo.

## Interacción y comandos

- Convierte acciones de widgets en comandos/eventos de aplicación cuando la acción afecte al dominio.
- Mantén efectos secundarios fuera de la representación visual. Un botón puede emitir `SaveConfig`, pero el servicio realiza el IO y devuelve el resultado.
- Maneja loading, success y error explícitamente.
- No dependas únicamente del color para comunicar estado; combina texto/icono/semántica accesible.

## Pintura personalizada

- Usa widgets estándar antes de dibujar todo manualmente.
- Para custom paint, trabaja con `Painter`, shapes y clipping del API actual; evita crear geometría costosa en cada frame.
- Mantén la separación entre contenido visual y respuesta de interacción: el hit testing no surge automáticamente de una figura dibujada.
- Comprueba DPI, antialiasing, colores y orden de capas.
- Para texturas/imágenes, gestiona carga, IDs, actualizaciones y liberación conforme al renderer.

## Multi-pass y viewports

- No asumas que el frame siempre es de una sola pasada. Revisa el modelo multi-pass y las condiciones para solicitar otro pass.
- Usa viewports/ventanas secundarias solo si el backend y el framework integrador lo soportan; valida los límites por plataforma.
- Controla el lifecycle de ventanas y recursos asociados; un viewport cerrado no debe dejar recursos o tareas huérfanas.

## Accesibilidad y calidad

- Usa labels, hints, tooltips y roles semánticos según el API.
- Prueba teclado, foco, lectores de pantalla/AccessKit cuando esté disponible, escalado y contraste.
- Evita controles custom que no tengan equivalente accesible.
- Mantén consistencia de spacing, tipografía, iconografía y jerarquía visual.

## Pruebas

- Extrae lógica de negocio y pruébala sin UI.
- Prueba componentes/composición con `Context` cuando sea práctico.
- Añade pruebas de estado para IDs, acciones, validaciones y cambios de selección.
- Añade screenshot/golden tests si el proyecto tiene infraestructura visual estable.
- Comprueba versión, DPI, tamaños extremos, listas extensas, datos vacíos y textos traducidos.

## Anti-patrones

- Tratar egui como un sistema retained-mode.
- Guardar referencias a widgets/`Ui` entre frames.
- Usar IDs derivados de índices inestables en listas mutables.
- Hacer IO/async/bloqueos dentro de la función UI.
- Redibujar miles de elementos sin clipping o virtualización.
- Usar píxeles físicos como si fueran puntos lógicos.

## Flujo del agente

1. Define jerarquía de pantallas/paneles y estado de cada componente.
2. Verifica APIs/feature flags en la versión bloqueada.
3. Construye primero UI con datos ficticios y acciones bien definidas.
4. Conecta comandos a servicios externos sin bloquear el frame.
5. Añade IDs estables, accesibilidad, DPI y estados de error.
6. Mide trabajo por frame y perfila componentes grandes.

## Criterios de aceptación

- UI declarada de forma inmediata, sin estado visual retenido incorrectamente.
- Estado de dominio independiente de widgets.
- Ninguna operación bloqueante pesada dentro del frame.
- IDs, layout y DPI son estables.
- Accesibilidad e interacción por teclado están contempladas.
