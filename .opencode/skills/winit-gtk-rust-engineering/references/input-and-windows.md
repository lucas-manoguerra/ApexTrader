# Ventanas, entrada, DPI y monitores

## Ventana

Configura ventanas mediante el builder de la versión fijada. Centraliza políticas comunes: título, dimensiones iniciales, redimensionamiento, visibilidad, decoraciones, nivel, fullscreen e iconos.

No hagas que cada módulo de negocio cree ventanas directamente. Usa un `WindowManager`/host que controle identificadores, estados y cierre.

## Teclado

- Diferencia tecla física de tecla lógica y de texto producido.
- Procesa modificadores y repetición de forma consistente.
- Respeta el foco: al perderlo, limpia las teclas/botones mantenidos.
- Usa IME/composición para texto internacional; no conviertas eventos de tecla en texto de manera ingenua.
- Mantén las conversiones específicas de versión dentro del adaptador de entrada.

## Mouse y gestos

- Guarda la posición de cursor con su espacio de coordenadas declarado.
- Distingue movimiento relativo de posición absoluta.
- Normaliza rueda en líneas/píxeles usando una política explícita.
- Considera mouse capture/grab y cursor confinado como capacidades específicas de plataforma; maneja fallos.
- No supongas que gestos de touchpad están disponibles igual en todas las plataformas.

## DPI y coordenadas

Mantén explícitos los siguientes valores:

- Coordenadas lógicas para layout e interacción de UI.
- Dimensiones físicas para framebuffer/superficie.
- Factor de escala reportado por el sistema.

Convierte en un único límite del sistema. Doble aplicación del factor de escala produce errores de tamaño y coordenadas.

## Monitores y fullscreen

- Trata monitor primario como opcional; algunas plataformas pueden no informar uno.
- Revalida disponibilidad de monitor/video mode antes de operaciones de fullscreen.
- Maneja cambios de monitor, resolución y escala.
- No asumas que `primary_monitor()` devuelve un valor en Wayland; la documentación de `winit` ya advierte que algunos backends pueden devolver `None`.

## GTK platform extensions en 0.29.1

El source de `winit-gtk 0.29.1` documenta traits en `winit::platform::unix`:

- `WindowExtUnix::gtk_window()` devuelve el `gtk::ApplicationWindow` utilizado por la ventana.
- `WindowExtUnix::default_vbox()` devuelve el `gtk::Box` creado como único hijo por defecto, o `None` si se deshabilitó.
- `WindowBuilderExtUnix::with_default_vbox(bool)` controla esa creación.
- `with_transient_for(...)` establece una ventana transitoria.
- `with_transparent_draw(bool)`, `with_double_buffered(bool)`, `with_rgba_visual(bool)` y `with_app_paintable(bool)` afectan la composición/dibujo.

Estos métodos son específicos de esa implementación. Comprueba `cfg`, target y la API publicada antes de importarlos; no los conviertas en dependencias del core de la aplicación.
