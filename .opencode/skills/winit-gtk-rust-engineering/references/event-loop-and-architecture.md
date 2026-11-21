# Event loop, ciclo de vida y arquitectura

## Modelo mental

`winit-gtk` es la capa de ventanas y eventos. No es un toolkit UI completo ni un renderer. El código de aplicación debe gestionar explícitamente el ciclo de vida de ventanas y el scheduling de trabajo.

## Flujo habitual

1. Crea el `EventLoop` en el hilo que será dueño del contexto GTK.
2. Crea la ventana o ventanas asociadas a ese event loop.
3. Inicializa el estado de aplicación, recursos GTK y renderer respetando los lifetimes reales.
4. Entra en `EventLoop::run`.
5. Convierte eventos de plataforma en cambios del estado interno.
6. Solicita redraw cuando el estado visual esté sucio.
7. Renderiza cuando se procesa `RedrawRequested` y la ventana/superficie está lista.
8. Atiende el cierre y libera recursos en el orden correcto.

## API versionada

La documentación de `winit-gtk 0.29.1` muestra el patrón de API de `winit 0.29`: `EventLoop::new()`, `WindowBuilder`, `EventLoop::run` con argumentos `(event, target, control_flow)`, `Event::MainEventsCleared` y `Event::RedrawRequested`. Las API recientes de `winit` cambiaron, por lo que el agente debe copiar la forma de la versión instalada y no mezclar ejemplos.

## ControlFlow

- `Wait`: adecuado cuando la aplicación puede dormir hasta nuevo evento.
- `WaitUntil`: adecuado para un deadline conocido, temporizadores o actualizaciones con frecuencia limitada.
- `Poll`: mantiene la iteración activa. Úsalo solo si la aplicación realmente necesita actualizar continuamente y se ha medido el coste energético/CPU.

No alternes `Poll` y `Wait` sin una política central clara. Define cuándo la aplicación necesita frames continuos, por ejemplo durante una animación, interacción arrastrada o una vista que se actualiza periódicamente.

## Redraw y dirty state

Mantén flags o generaciones de estado visual sucio por ventana. Una actualización solicita redraw, pero no debe dibujar inmediatamente desde cualquier callback. Agrupa cambios compatibles y evita solicitar múltiples frames redundantes.

El sistema operativo también puede pedir un redraw (por exposición, restauración u otras razones). No supongas que solo la aplicación controla cuándo hay que volver a dibujar.

## Múltiples ventanas

Usa `WindowId` para mapear cada evento a su estado:

```rust
// Esquema conceptual: adapta tipos y API a la versión instalada.
HashMap<WindowId, WindowState>
```

Cada estado de ventana puede almacenar su configuración visual, tamaño físico, factor DPI, flags de foco/minimización y referencia a su renderer. No uses una variable global `window` como si todas las ventanas fueran la misma.

## Tamaños y suspensión

- Conserva por separado el tamaño lógico de UI y el tamaño físico de render.
- Ante cambios de escala, recalcula el tamaño físico y actualiza layouts y superficies.
- Si la ventana queda con dimensiones físicas cero, suspende el render en lugar de configurar una superficie inválida.
- Ante cierre, destruye los recursos específicos de esa ventana y elimina su entrada del mapa.

## Responsabilidades de módulos sugeridas

```text
src/
├── app/             # Estado y orquestación de la aplicación
├── windowing/       # EventLoop, ventanas y adaptadores de plataforma
├── input/           # Normalización de teclado/mouse/gestos
├── gtk_ui/          # Widgets y llamadas GTK (hilo autorizado)
├── renderer/        # Integración wgpu u otro backend
└── domain/          # Estado y lógica no visual
```

Adapta la estructura al tamaño real del proyecto; no crees módulos vacíos solo para cumplir un esquema.
