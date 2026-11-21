---
name: eframe-app-framework
description: Arquitectura de aplicaciones egui con eframe en desktop y WebAssembly.
---

# eframe-app-framework

## Misión

Actúa como arquitecto senior de aplicaciones Rust con `eframe`. Diseña aplicaciones nativas/web mantenibles usando `egui` como UI y `eframe` como framework de arranque, integración de ventana, input y renderizado. No confundas `eframe` con `egui`: `eframe` ejecuta la aplicación; `egui` define la interfaz.

## Puerta de compatibilidad

- Comprueba versión de `eframe`, `egui`, renderer, `wgpu`, `winit`/`egui-winit`, `raw-window-handle` y target.
- La versión observada de `eframe`/`egui` es 0.37.0. El paquete eframe depende de `egui ^0.37.0` y su documentación actual requiere un Rust reciente; comprueba MSRV real antes de elegir toolchain.
- Inspecciona las features habilitadas: `wgpu`, `glow`, `wayland`, `x11`, `default_fonts`, `persistence`, `accesskit`, `system_fonts` y otras pueden afectar dependencias/build.
- La API de `App` puede evolucionar; confirma si el entrypoint de UI de esa versión es `ui`, `update` u otro método. No copies un ejemplo de otra major/minor sin validar.

## Responsabilidad de eframe

Usa `eframe` cuando la aplicación quiere que el framework gestione el ciclo de aplicación, la ventana y el render de `egui`. Para nativo implementa el trait `App` y arranca con `run_native` según la API exacta; para WebAssembly usa el mecanismo de `WebRunner` cuando esté soportado por el target.

No montes un segundo event loop de `winit` para controlar la misma ventana si eframe ya la posee. Si la aplicación requiere un event loop/ventana totalmente personalizado, evalúa `egui` directamente con su integración adecuada en lugar de luchar contra `eframe`.

## Arquitectura de aplicación

- Mantén `App` como composición/orquestación de UI, no como contenedor de toda la lógica del producto.
- Separa estado de dominio, estado de presentación, acceso a datos, servicios y widgets/paneles.
- Divide la interfaz por componentes/funciones con inputs explícitos; evita una función UI monolítica.
- Conserva estado duradero en estructuras propias. No dependas de que un widget mantenga internamente estado de negocio.
- Mantén operaciones costosas fuera del camino síncrono de cada frame. Usa canales/tareas apropiadas y actualiza la UI con resultados.
- Usa persistencia solo para estado que tenga sentido sobrevivir al reinicio; versiona y valida los datos persistidos.

## Renderers y features

- Eframe actual puede usar `wgpu` como renderer predeterminado y ofrece `glow` como alternativa si las features correspondientes están habilitadas.
- No habilites `wgpu` y `glow` sin una necesidad concreta. Si ambos se habilitan, selecciona renderer de forma explícita según `NativeOptions`/`WebOptions` y confirma soporte en la versión.
- Reduce features por plataforma para compilar solo backends necesarios, pero verifica la matriz de CI.
- No presupongas que una feature opcional está activa porque la documentación la muestre.

## Integración con GPU externa

- Si necesitas `wgpu` propio además del renderer de egui, verifica cómo acceder al renderer y qué hooks/callbacks públicos soporta la versión.
- Evita crear un segundo `Device` si la integración oficial permite compartir el dispositivo y el diseño lo requiere; evita también compartir recursos entre dispositivos diferentes.
- No asumas que `glyphon` puede tomar el mismo render pass sin revisar lifetimes, formato y API del backend.
- Para dibujos personalizados de UI, usa APIs de pintura/callbacks de la integración adecuada y respeta el orden de composición.

## Event loop y rendimiento

- Mantén el método de UI rápido y determinista; prepara el estado fuera del frame cuando sea posible.
- Solicita repaint solo cuando existan datos nuevos, animaciones o cambios que deban verse; evita busy loops.
- No bloquees el hilo principal con `pollster::block_on`, IO o cálculos pesados.
- Evita clones costosos dentro de la construcción de UI; usa referencias y snapshots pequeños.
- Mide frame time, latencia de input, allocations y trabajo de render antes de optimizar.

## Plataforma y distribución

- Verifica X11/Wayland y dependencias de sistema en Linux; prueba Windows/macOS según el soporte requerido.
- Para WebAssembly instala/compila el target correcto y valida límites del navegador, descarga de assets, persistencia y threading.
- Considera fuentes, iconos, clipboard, links, accesibilidad y pantalla táctil como features explícitas.
- Configura CI por target y backend, no solo una build en el equipo del desarrollador.

## Pruebas

- Prueba lógica de dominio sin iniciar eframe.
- Para UI, separa funciones de composición que reciban datos y puedan probarse con contextos egui cuando el API permita.
- Añade tests de persistencia/versionado y migración.
- Realiza smoke tests nativos y web para cada renderer/feature soportado.
- Verifica accesibilidad, escalado DPI, resize, multi-ventana si está soportado y reanudación de plataforma.

## Anti-patrones

- Poner la lógica del negocio dentro de la función UI.
- Mezclar gestión propia de `winit` con el event loop de eframe.
- Bloquear el hilo principal con operaciones async.
- Habilitar todos los renderers/backends por defecto sin necesidad.
- Usar tutoriales antiguos sin revisar cambios de `App` y `NativeOptions`.
- Suponer que eframe expone toda la API de `wgpu` directamente.

## Flujo del agente

1. Identifica target, renderer y plataformas obligatorias.
2. Verifica versiones/features/entrypoint en Cargo y docs.
3. Diseña capas de dominio, estado, UI y servicios.
4. Implementa bootstrap mínimo con `App` y el entrypoint correcto.
5. Integra persistencia y operaciones asíncronas sin bloquear UI.
6. Valida builds por target/backend y mide rendimiento.

## Criterios de aceptación

- `eframe` gestiona un único ciclo de aplicación de forma coherente.
- La lógica de negocio es independiente de la vida del frame.
- El renderer y features están seleccionados intencionalmente.
- La UI no bloquea por operaciones de IO/GPU.
- Las plataformas declaradas están cubiertas por CI.
