# GTK, hilos y contexto

## Restricción principal

El README del fork `winit-gtk` establece que GTK puede inicializarse en cualquier hilo, pero el contexto debe usarse en el mismo hilo. También advierte que, aunque el fork hace que `Window` y ciertos tipos proxy puedan utilizarse desde múltiples hilos, las llamadas a métodos GTK deben ejecutarse en el hilo donde se creó el event loop; de lo contrario GTK puede hacer panic.

La API documentada de `EventLoop` tampoco implementa `Send` ni `Sync`.

## Política segura

- Crea el event loop y todos los objetos GTK asociados en el mismo hilo.
- Mantén GTK widgets y referencias nativas confinados a ese hilo.
- Nunca captures un `gtk::Widget`, `gtk::ApplicationWindow` o `gtk::Box` para usarlos en un worker.
- Los workers realizan I/O, cálculo, decodificación o procesamiento de datos sin tocar GTK.
- Envían mensajes de resultados al hilo UI mediante `EventLoopProxy<UserEvent>` o un canal con mecanismo de despertar.
- El hilo UI recibe el mensaje, actualiza el estado de la aplicación y modifica los widgets GTK.
- Evita locks que cubran llamadas GTK, callbacks o renderizado.

## Mensajes de usuario

Define un enum de eventos de aplicación en vez de enviar closures opacas que capturen referencias GTK:

```rust
// Ejemplo conceptual; confirma la firma de EventLoopProxy en la versión instalada.
enum UserEvent {
    DataUpdated,
    BackgroundTaskFinished(Result<(), String>),
    RequestClose(WindowId),
}
```

Haz los mensajes pequeños, tipados y con propiedad clara. Para datos grandes, almacena el payload en una cola/canal y envía un mensaje de notificación para despertar el event loop.

## Uso de Window desde otro hilo

El README y los ejemplos del fork incluyen patrones como solicitar redraw desde un worker. No extrapoles esa capacidad a todo el API de ventana y, especialmente, no la extrapoles a los métodos GTK expuestos por traits de plataforma. Verifica cada método, su documentación y sus garantías de thread-safety.

## Diagnóstico de bloqueos

Si la interfaz se congela:

1. Comprueba si hay I/O o cálculo dentro del event loop.
2. Busca locks mantenidos durante callbacks.
3. Verifica que el worker no espere sincrónicamente una operación que necesita el hilo GTK.
4. Verifica que no se haya cruzado el contexto GTK entre hilos.
5. Añade trazas temporales de inicio/fin de handlers y mide latencias sin inundar logs de producción.
