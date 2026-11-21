---
name: pollster-async-bridge
description: Uso seguro de block_on como puente puntual entre código síncrono y futures.
---

# pollster-async-bridge

## Misión

Actúa como ingeniero senior de Rust especializado en integrar APIs `async` en aplicaciones predominantemente síncronas usando `pollster`. Úsalo como puente acotado, no como runtime general.

## Puerta de compatibilidad

- Comprueba la versión bloqueada y la API en la documentación de esa versión. La versión `latest` actualmente es 1.0.2, pero no fijes el proyecto a esa versión sin revisar MSRV y compatibilidad.
- Verifica la disponibilidad de `pollster::block_on`, `FutureExt`, `#[pollster::main]` y `#[pollster::test]` y las features/macro opcionales reales.
- Revisa el executor ya usado por el proyecto (Tokio, async-std, futures, runtime propio). No añadas `pollster` si un executor presente resuelve mejor el caso.

## Qué hace y qué no hace

`pollster` bloquea el hilo actual hasta que un `Future` termina. Su valor está en evaluar futures puntuales sin añadir un runtime grande. No proporciona un sistema general de tareas, timers, sockets async, cancelación estructurada ni coordinación de miles de operaciones concurrentes.

## Usos apropiados

- Inicialización asíncrona puntual desde un `main` síncrono antes de entrar al event loop, cuando la API y la plataforma permiten bloquear ahí.
- Herramientas CLI, scripts de desarrollo o utilidades pequeñas que necesitan esperar una sola operación async.
- Tests síncronos que deben esperar un future aislado, usando la macro o función de la versión verificada.
- Código de ejemplo y bootstrap de recursos como un `wgpu::Adapter`/`Device`, siempre que el hilo y el backend no requieran que el event loop avance durante la espera.

## Usos no apropiados

- Bloquear el hilo UI durante un frame, callback de entrada, render loop o tarea de interacción.
- Ejecutar `block_on` en cada frame o dentro de un bucle de alta frecuencia.
- Usarlo como sustituto de Tokio/async-std para IO continuo o concurrencia.
- Invocar `block_on` recursivamente o sobre futures que dependen de progreso de un runtime/event loop que el mismo hilo acaba de bloquear.
- Asumir que un future diseñado para un runtime concreto funcionará sin ese runtime. Inspecciona sus requisitos.
- En WASM/browser, no asumir que el bloqueo síncrono es viable; sigue el modelo async del target y la documentación de plataforma.

## Reglas de diseño

1. Identifica qué future se espera, cuánto puede tardar y qué hilo se bloquea.
2. Confirma que todas las operaciones que hacen avanzar el future pueden progresar mientras ese hilo está bloqueado.
3. Si la espera es larga o repetida, cambia el diseño: propaga `async`, usa un runtime apropiado o comunica resultado mediante canales/estado compartido cuidadosamente sincronizado.
4. Mantén el límite sync/async en una frontera arquitectónica visible, no disperses `block_on` por módulos de dominio.
5. No llames a `block_on` con locks, mutexes o recursos bloqueados que el future pueda necesitar.
6. No conviertas errores en `unwrap`; devuelve/propaga `Result`.
7. Documenta claramente los puntos de bloqueo y sus presupuestos de latencia.

## Ejemplos de forma

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = pollster::block_on(async {
        // operación async autocontenida y acotada
        Ok::<_, std::io::Error>(())
    });
    result?;
    Ok(())
}
```

El ejemplo solo ilustra el límite síncrono; no lo copies dentro de un event loop ni presupongas que cualquier future es compatible.

## Integración con GUI y GPU

- Para `wgpu`, evalúa si la inicialización puede completarse antes del loop de eventos. Comprueba si el backend necesita interacción con una ventana/event loop ya activo.
- No bloquees el hilo GTK/winit/eframe durante eventos o renderizado. Una app puede inicializar de forma sync si su backend lo permite, pero las tareas largas deben continuar de forma no bloqueante.
- Diseña estados explícitos `Loading`, `Ready`, `Failed` cuando la inicialización tenga que ocurrir mientras la interfaz ya está viva.
- Evita ejecutar una inicialización completa cada vez que se recrea un frame o cambia el tamaño de la ventana.

## Anti-patrones

- Tratar `pollster` como reemplazo de Tokio.
- Bloquear la UI para ocultar una API async mal integrada.
- Mantener un mutex durante `block_on`.
- Suponer que `block_on` hace async el código llamador: el hilo llamador queda bloqueado.
- Ignorar comportamiento específico de WASM, runtimes o event loops.

## Flujo del agente

1. Localiza el borde síncrono/async y los futures involucrados.
2. Identifica runtime, hilo y necesidades de progreso.
3. Determina si el bloqueo es corto, aislado y seguro.
4. Implementa una única frontera con `block_on` o una alternativa no bloqueante.
5. Mide latencia y comprueba que UI/event loop siguen respondiendo.
6. Añade tests de éxito, error y rutas de espera relevantes.

## Criterios de aceptación

- El hilo bloqueado y el presupuesto temporal son explícitos.
- El future puede progresar sin un runtime/event loop detenido.
- No se bloquea el hilo de UI/render en operación normal.
- No se usa `pollster` como runtime de tareas.
- La API/macro usada coincide con la versión resuelta.
