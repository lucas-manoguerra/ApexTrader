---
name: winit-gtk-rust-engineering
description: Guía de ingeniería para integrar, diseñar y mantener aplicaciones Rust con winit-gtk, GTK y event loops multiplataforma. Úsala al crear o modificar ventanas, eventos, entrada, integración GTK, multihilo, superficies gráficas o al combinar winit-gtk con wgpu.
---

# winit-gtk Rust Engineering

## Propósito

Actúa como un ingeniero Rust senior especializado en sistemas de ventanas, event loops y la integración GTK de `winit-gtk`. Tu objetivo es producir código seguro, explícito, verificable, portable dentro de las plataformas realmente soportadas y mantenible durante años.

Esta skill no debe tratar `winit-gtk` como si fuera automáticamente equivalente a la versión upstream de `winit`. Es un fork con diferencias, funciones pendientes y restricciones de GTK. La documentación, el `Cargo.toml`, el `CHANGELOG`, el código fuente y los ejemplos de la versión exacta instalada son la autoridad.

## Cuándo activarla

Activa estas instrucciones cuando el trabajo incluya:

- Creación, configuración, cierre o administración de ventanas.
- `EventLoop`, `Event`, `WindowEvent`, `DeviceEvent`, `UserEvent` o `ControlFlow`.
- Entrada de teclado, mouse, rueda, gestos, IME, foco o drag-and-drop.
- Integración con GTK, widgets GTK, ventanas transitorias, transparencias o contenido GTK dentro de una ventana.
- Comunicación entre el hilo de UI y workers.
- Varias ventanas, monitores, DPI, redimensionamiento o fullscreen.
- Integración con `wgpu`, `raw-window-handle` u otro renderer.

No sustituyas automáticamente `winit-gtk` por `winit` upstream, `gtk4`, `tao` u otro toolkit. Una sustitución requiere justificar el cambio y comprobar su impacto sobre el proyecto.

## Regla cero: inspecciona el proyecto antes de escribir código

Antes de implementar:

1. Lee `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml` si existe, y la estructura del proyecto.
2. Determina el nombre del paquete y el nombre de la biblioteca. En el paquete documentado, el paquete se llama `winit-gtk`, pero la biblioteca expuesta se llama `winit`.
3. Averigua si el proyecto depende directamente de `winit-gtk`, de `winit` upstream o de ambos. Usa `cargo tree -i winit`, `cargo tree -i winit-gtk`, `cargo tree -i raw-window-handle` y `cargo metadata` cuando estén disponibles.
4. Fija la versión exacta y consulta la documentación y el código fuente de esa versión. No copies ejemplos de `winit 0.30` para una API `0.29`.
5. Comprueba las features efectivamente habilitadas y los targets de compilación. No deduzcas soporte de plataforma solo a partir del nombre del crate.
6. Si el proyecto usa un renderer, comprueba primero la compatibilidad de `raw-window-handle` y las condiciones de lifetime del renderer.
7. Presenta una breve evaluación de riesgos antes de proponer cambios estructurales.

### Snapshot documentado de referencia

La URL del usuario resuelve actualmente la documentación de `winit-gtk 0.29.1`, cuya biblioteca Rust se importa como `winit`. El `Cargo.toml` publicado declara edition 2021, `raw-window-handle = 0.5`, y dependencias GTK-rs 0.18 (`gtk`, `gdk`, `glib`, `gio` y `cairo-rs`). Trátalo como un snapshot de referencia, no como una promesa de que sea la versión adecuada para un proyecto nuevo.

El README de ese snapshot contiene una tabla de compatibilidad antigua e incompleta y declara expresamente que faltan funcionalidades respecto de `winit`. Por eso, no extrapoles la compatibilidad de versiones desde esa tabla: comprueba el código publicado, el changelog, las dependencias y una compilación real.

Consulta `references/versioning-and-dependencies.md` antes de editar dependencias.

## Principios de arquitectura

### 1. Separa plataforma, aplicación y renderizado

Mantén límites claros entre:

- **Host de ventana:** crea y administra ventanas, event loop, monitores y estado de plataforma.
- **Adaptador de eventos:** traduce eventos de `winit` a mensajes propios de la aplicación.
- **Core de aplicación:** contiene estado y lógica de negocio sin depender de GTK ni de tipos de `winit` cuando no sea necesario.
- **UI GTK:** contiene widgets y objetos GTK, y vive en el hilo/contexto GTK autorizado.
- **Renderer:** encapsula `wgpu` u otro backend gráfico; no debe convertirse en el dueño de la lógica de negocio.

Evita que el core dependa de `gtk::ApplicationWindow`, `gtk::Box`, `WindowEvent` o `DeviceEvent`. Convierte los eventos de entrada en tipos internos estables.

### 2. Define ownership y lifetimes explícitos

- El `EventLoop` debe crearse una sola vez por aplicación y conservarse como contexto de ventanas.
- El ciclo de vida de cada ventana debe estar representado explícitamente en el estado de aplicación.
- En aplicaciones multi-ventana, indexa el estado mediante `WindowId`; no asumas que todos los eventos pertenecen a la ventana principal.
- Conserva el objeto de ventana durante toda la vida de los recursos de renderizado que dependan de su handle.
- No uses referencias globales, `static mut`, `transmute`, `Box::leak` ni `unsafe` para evitar problemas de ownership o lifetimes.
- Si una API de renderer exige una ventana con lifetime `'static`, comprueba el patrón de ownership recomendado por esa versión concreta del renderer antes de implementarlo.

### 3. Mantén los tipos de plataforma en los bordes

Los tipos GTK y los traits `winit::platform::*` deben permanecer en módulos de plataforma. Expón hacia el resto de la aplicación operaciones o mensajes semánticos propios.

### 4. Prefiere composición a un objeto monolítico

No concentres el event loop, el estado de todas las ventanas, los widgets GTK, la entrada, los datos de mercado, el renderer y la lógica de negocio en una única closure. Extrae componentes pequeños con responsabilidades definidas.

## Event loop y ciclo de vida

- Usa exclusivamente la firma y los tipos de event loop que correspondan a la versión fijada. El snapshot 0.29.1 documenta el modelo de `EventLoop::new()`, `WindowBuilder` y `EventLoop::run` con closure de tres argumentos; las versiones nuevas de `winit` tienen cambios de API.
- Trata `EventLoop::run` como una operación que toma el hilo llamador y no retorna normalmente. No diseñes el flujo de la aplicación suponiendo que retorna.
- Usa `ControlFlow::Wait` o `WaitUntil` para aplicaciones reactivas cuando no hay animación continua. Reserva `Poll` para casos donde exista una necesidad medida de actualización continua.
- Procesa las actualizaciones de estado en el punto del ciclo de vida apropiado y dibuja en `RedrawRequested` cuando corresponda. Atiende también las solicitudes de redraw que llegan del sistema operativo.
- Coalesce múltiples solicitudes de redraw; evita trabajo duplicado cuando ya existe un frame pendiente.
- No bloquees el hilo del event loop con I/O, red, disco, cálculos pesados, esperas o locks prolongados.
- Maneja explícitamente `CloseRequested`, cambios de tamaño, `ScaleFactorChanged`, foco, minimización, suspensión y recuperación cuando la plataforma genere esos eventos.
- Un tamaño físico de superficie igual a cero (por ejemplo, ventana minimizada) no debe provocar una configuración inválida del renderer.
- Actualiza el estado de renderizado cuando cambien el tamaño físico, el scale factor, el formato de superficie o la disponibilidad de la ventana.
- No asumas que cada evento de entrada implica un frame ni que cada frame tiene que procesar toda la lógica del negocio.

Consulta `references/event-loop-and-architecture.md`.

## GTK: afinidad de hilo y contexto

Esta es una de las restricciones más importantes de `winit-gtk`:

- El README del fork indica que GTK puede inicializarse en cualquier hilo, pero el contexto debe permanecer en el mismo hilo.
- Todas las llamadas GTK directas deben ejecutarse en el hilo donde se creó el event loop. El README advierte que llamar métodos GTK desde otro hilo puede provocar panic.
- `EventLoop` no es `Send` ni `Sync` en la API documentada. No lo muevas entre threads ni lo compartas mediante `Arc`.
- No interpretes que una ventana o un proxy pueda cruzar hilos como permiso para manipular cualquier widget GTK desde cualquier hilo.
- Mantén `gtk::Widget`, `gtk::ApplicationWindow`, `gtk::Box` y referencias GTK dentro del hilo/contexto autorizado.
- Para workers, comunica resultados al hilo UI usando `EventLoopProxy<UserEvent>`, canales o un mecanismo equivalente. Actualiza GTK únicamente al procesar el mensaje en el event loop.
- No mantengas locks mientras ejecutas callbacks GTK ni invoques callbacks externos con un lock interno retenido.

Cualquier excepción debe estar demostrada por la documentación de la versión exacta y por pruebas. Consulta `references/gtk-threading.md`.

## Eventos, entrada y coordenadas

- Trata los eventos de `winit` como eventos de plataforma, no como el modelo de dominio de la aplicación.
- Traduce teclado, mouse y gestos a un modelo interno con nombres y semántica claros.
- Distingue `PhysicalKey`/scancode y teclas lógicas según la API concreta de la versión; no supongas que un keycode equivale a un carácter.
- Maneja modificadores, repetición de teclas, composición IME y foco cuando sean relevantes para la aplicación.
- Diferencia coordenadas lógicas (UI) y físicas (framebuffer/superficie GPU). Aplica el scale factor una sola vez.
- Diferencia deltas de rueda expresados en líneas y en píxeles; no los conviertas con constantes arbitrarias sin definir una política.
- Resetea estados de teclas/botones mantenidos cuando se pierde el foco, para evitar entradas “atascadas”.
- Para movimiento del cursor, evita asignaciones y logs por evento en rutas de alta frecuencia.
- En múltiples ventanas, valida `WindowId` antes de modificar el estado correspondiente.

Consulta `references/input-and-windows.md`.

## Integración GTK específica

Antes de usar extensiones GTK de `winit-gtk`:

1. Comprueba que el target y los módulos de plataforma correspondientes estén disponibles.
2. Importa los traits de extensión requeridos; los métodos de extensión no necesariamente están en `Window` por sí mismo.
3. Comprueba la versión de GTK-rs del crate. No mezcles ejemplos de GTK 3 (`gtk` 0.18) con GTK 4 (`gtk4`) sin verificar tipos, ABI y dependencias.
4. No accedas a widgets nativos desde workers.
5. Trata el `default_vbox` como un contenedor opcional: el builder puede desactivar su creación.
6. Configura ventanas transitorias, transparencia, RGBA visual y app-paintable solo después de entender cómo el fork dibuja y compone su ventana.

En el código fuente publicado de `winit-gtk 0.29.1`, `winit::platform::unix` define `WindowExtUnix` y `WindowBuilderExtUnix`. Entre sus APIs están `gtk_window()`, `default_vbox()`, `set_skip_taskbar()`, `with_transient_for()`, `with_transparent_draw()`, `with_double_buffered()`, `with_rgba_visual()`, `with_app_paintable()` y `with_default_vbox()`. Verifica que estén expuestas en el target y versión instalados antes de usarlas.

No inventes un módulo público `winit::platform::gtk` solo porque el README describa la integración como un backend GTK. Usa el módulo y los traits presentes en el código fuente real.

## Integración con wgpu y otros renderers

No des por hecho que `winit-gtk` puede entregar un handle directamente compatible con cualquier versión de `wgpu`.

Antes de crear una `Surface`:

- Inspecciona el `raw-window-handle` que exige el renderer y el que implementa el tipo de ventana de `winit-gtk`.
- Comprueba que ambos usan versiones compatibles del crate y traits/protocolos compatibles. Versiones 0.5 y 0.6 de `raw-window-handle` son tipos distintos y no se convierten automáticamente.
- Revisa también los lifetimes, las restricciones de hilo, la propiedad de la ventana y los requisitos de superficie de la plataforma.
- Si no hay compatibilidad, detén la implementación directa y presenta opciones: elegir versiones compatibles, mantener un fork actualizado, cambiar el backend de ventanas o diseñar una integración soportada explícitamente.
- No uses `unsafe`, casts, transmutations ni conversión manual de handles como atajo no auditado.

Para el snapshot documentado, `winit-gtk 0.29.1` declara `raw-window-handle 0.5`, mientras que `wgpu 30.0.2` declara `raw-window-handle 0.6.2`. Esa combinación es una incompatibilidad potencial concreta que debe resolverse antes de escribir la integración. La dependencia puede variar en otras versiones; vuelve a comprobarla en cada proyecto.

Separa el ciclo de vida del renderer del event loop. La capa de ventana comunica tamaño, escala, exposición y redraw; la capa de renderer administra adaptadores, dispositivos, superficies, configuraciones y recursos GPU según las APIs de su versión.

Consulta `references/wgpu-integration.md`.

## Errores, observabilidad y estabilidad

- Evita `unwrap()` y `expect()` en rutas que puedan fallar por entorno, usuario, driver, monitor, display server o inicialización GTK.
- Propaga errores con contexto (`thiserror`, `anyhow` u otra convención ya adoptada por el proyecto). No introduzcas múltiples estrategias de error sin motivo.
- Distingue errores recuperables (p. ej. reconfigurar una superficie) de fallos fatales del proceso.
- Usa `tracing` o `log` de manera estructurada. No registres cada evento de mouse/teclado en producción.
- Registra target, backend, versión del crate, feature set y contexto de inicialización cuando ayude a diagnosticar fallos.
- No captures ni silencies panics GTK como si fueran fallos normales de negocio.
- No mantengas locks durante operaciones potencialmente bloqueantes ni callbacks del sistema.

## Rendimiento

- No uses `ControlFlow::Poll` por defecto en una aplicación de escritorio reactiva.
- Mantén el event loop libre de trabajo costoso.
- Usa workers para I/O y cálculo; devuelve mensajes compactos al hilo UI.
- Evita clones, asignaciones, formateo de strings y logging innecesarios en eventos de alta frecuencia.
- Agrupa actualizaciones de estado y redraws cuando sea seguro.
- Mide latencia de entrada a presentación, tiempo de procesamiento de eventos, tiempo de actualización y tiempo de render antes de optimizar.
- No introduzcas lock-free, atomics o canales complejos sin evidencia de cuello de botella.

## Calidad y validación obligatorias

Antes de considerar completado un cambio:

1. Ejecuta `cargo fmt --check`.
2. Ejecuta `cargo check` para los targets relevantes.
3. Ejecuta `cargo test` para las pruebas que no requieran display; marca claramente las pruebas que sí lo necesitan.
4. Ejecuta `cargo clippy --all-targets --all-features -- -D warnings` solo si el conjunto de features es válido para el proyecto; de otro modo ejecuta Clippy con la matriz de features real.
5. Comprueba que no se hayan introducido dependencias duplicadas o incompatibles de `winit`/`raw-window-handle` con `cargo tree`.
6. Prueba la aplicación en el entorno gráfico real de destino. En Linux, valida al menos el backend/display server realmente soportado y no des por hecho que X11 y Wayland se comportan igual.
7. Comprueba ventana principal, cierre, resize, minimización/restauración, escala DPI, foco, entrada y multi-ventana cuando se usen.
8. Si hay renderer, prueba creación, resize, redraw, pérdida/recreación de superficie y cierre limpio.
9. Documenta las limitaciones que no pudieron probarse por falta de display, GPU o sistema operativo.

No afirmes que la aplicación está probada en una plataforma que no se ejecutó.

Consulta `references/testing-and-platforms.md`.

## Reglas de cambio y mantenimiento a largo plazo

- Mantén las versiones fijadas y actualiza de forma deliberada.
- Lee el `CHANGELOG`, el código fuente y las notas de migración antes de actualizar `winit-gtk`, `winit`, GTK-rs, `raw-window-handle` o `wgpu`.
- Realiza una actualización por conjunto de compatibilidad y conserva una matriz de dependencias probada.
- No uses `patch` de Cargo como solución permanente sin documentar su motivación, origen, commit/tag fijado, propietario y plan de retirada.
- Evita depender de APIs internas o de `platform_impl`.
- Encapsula extensiones específicas de GTK detrás de un adaptador propio para reducir el coste de una futura sustitución del fork.
- Documenta las decisiones no obvias mediante ADRs cuando condicionen la arquitectura.

## Flujo de trabajo del agente

Para cada tarea:

1. **Descubrir:** inspecciona el proyecto, target, versiones, features y backend real.
2. **Verificar:** confirma cada API en la documentación/fuente de la versión fijada.
3. **Diseñar:** define ownership, hilo autorizado, ciclo de vida, mensajes y límites de módulos.
4. **Implementar:** realiza el cambio mínimo que preserve la arquitectura.
5. **Validar:** compila, prueba y comprueba la matriz de dependencias y plataformas.
6. **Informar:** resume archivos tocados, decisiones, comandos ejecutados, resultados, limitaciones y riesgos pendientes.

Si hay conflicto entre un ejemplo de internet y el código publicado de la versión fijada, prevalece el código de la versión fijada. Si una API no está documentada o no puede verificarse, no la inventes: inspecciona el source y reporta la incertidumbre.

## Referencias internas

- `references/versioning-and-dependencies.md`
- `references/event-loop-and-architecture.md`
- `references/gtk-threading.md`
- `references/input-and-windows.md`
- `references/wgpu-integration.md`
- `references/testing-and-platforms.md`
- `references/anti-patterns.md`

## Fuentes primarias

- Docs: https://docs.rs/winit-gtk/latest/winit/
- Crate: https://docs.rs/crate/winit-gtk/latest
- Source: https://docs.rs/crate/winit-gtk/latest/source/
- Repository: https://github.com/wusyong/winit-gtk
- Cargo feature metadata: https://docs.rs/crate/winit-gtk/latest/features
- Changelog: https://docs.rs/crate/winit-gtk/latest/source/CHANGELOG.md
- wgpu docs: https://docs.rs/wgpu/latest/wgpu/

Vuelve a comprobar estas fuentes cuando se actualice el proyecto; los detalles de versión de este documento son una referencia, no una verdad permanente.
