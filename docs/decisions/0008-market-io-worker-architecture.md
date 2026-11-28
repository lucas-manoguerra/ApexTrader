# ADR-0008: Ejecución aislada para I/O de mercado

## Estado

Aceptada

## Contexto

Las interfaces `MarketDataProvider` y `RealtimeMarketDataProvider` permiten operaciones que pueden bloquear por I/O. Ejecutarlas en el hilo de interfaz puede congelar el renderizado durante una carga histórica o una actualización realtime.

La UI debe conservar la propiedad exclusiva de `ChartState`. Los workers reciben solicitudes y producen resultados/mensajes; nunca mutan el gráfico ni comparten referencias mutables a su estado.

La implementación actual ya ejecuta las cargas históricas y el polling realtime en workers dedicados. Esta ADR registra la arquitectura implementada y sus límites conocidos; no implica que exista todavía un adaptador de exchange real.

## Alternativas consideradas

1. **Runtime async (por ejemplo, Tokio).** Ofrece I/O concurrente y un ecosistema de red amplio, pero agrega una dependencia de runtime y decisiones de integración/ciclo de vida que no son necesarias para los providers mock actuales.
2. **Un único worker para histórico y realtime.** Es simple, pero una carga histórica lenta puede retrasar el stream realtime y viceversa.
3. **Workers dedicados con primitivas de la biblioteca estándar.** Separa los ciclos de vida histórico y realtime, evita un runtime global y permite comunicación explícita y acotada.

## Decisión

Adoptar workers dedicados basados en `std::thread`, comunicación acotada mediante primitivas de `std::sync::mpsc` y mailboxes, y señales de cancelación cooperativa. No introducir Tokio ni otro runtime async en esta frontera por ahora.

### Propiedad y responsabilidades

- La UI conserva `ChartState`, el renderer y la aplicación de cada resultado.
- El worker histórico procesa solicitudes latest-wins. Una nueva solicitud reemplaza la pendiente y solicita cancelar la operación activa anterior.
- Las respuestas históricas incluyen generación e identidad de símbolo/serie. La UI descarta resultados que ya no correspondan a la selección vigente.
- El worker realtime posee su provider y ejecuta su ciclo de polling fuera del hilo UI. La creación del provider mediante factory también ocurre en el worker.
- Los workers no reciben `ChartState`, `egui::Context` ni referencias a widgets.
- La UI drena resultados sin espera bloqueante y aplica los cambios al gráfico desde el hilo UI.

### Cambio de símbolo y respuestas obsoletas

Cada cambio de símbolo incrementa la generación activa, solicita la cancelación de la carga anterior y descarta el estado de gráfico anterior. La UI valida generación e identidad antes de aplicar una respuesta histórica, de modo que una respuesta tardía no pueda sobrescribir el símbolo seleccionado más recientemente.

El worker histórico mantiene como máximo la solicitud pendiente más reciente. La cancelación no equivale a interrumpir arbitrariamente una llamada bloqueante: los métodos cancellable por defecto comprueban el token antes y después del método legacy, pero un provider debe sobrescribirlos para interrumpir una operación en curso.

### Backpressure y semántica realtime

- Las colas y mailboxes tienen capacidad finita.
- Las actualizaciones de una vela abierta pueden coalescerse por símbolo e instante de apertura, conservando la versión más reciente.
- Los cierres de vela no se expulsan para dar lugar a actualizaciones reemplazables. Si el mailbox está lleno de cierres, se aplica backpressure mediante espera cancelable.
- La mailbox conserva el último error realtime para que pueda presentarse en la UI.
- La UI valida la identidad del símbolo antes de aplicar actualizaciones. La validación temporal y de integridad del dominio sigue siendo responsabilidad de las capas correspondientes.

### Cancelación, shutdown y Drop

`CancellationToken` permite solicitar cancelación cooperativa. Los providers que realicen I/O remoto deberán usar timeouts finitos de conexión, lectura y operación, y revisar la cancelación en sus puntos de espera.

Hay dos semánticas de cierre deliberadamente distintas:

- `shutdown()` es explícito y hace join del worker. Debe invocarse únicamente desde un contexto que pueda esperar.
- `Drop` solicita cancelación, cierra la mailbox y transfiere el `JoinHandle` a un thread reaper dedicado. Así, destruir el worker desde la UI no espera sincrónicamente a que termine una operación del provider.

El reaper evita bloquear al llamador; no garantiza que un provider no cooperativo termine en un plazo acotado. Un provider que ignore la cancelación y carezca de timeouts finitos puede dejar su thread ejecutándose en segundo plano. Por eso los timeouts y la cancelación cooperativa son requisitos para cualquier integración de red.

### Estado funcional y límites actuales

La implementación cubre workers históricos y realtime, cancelación, descarte de respuestas históricas obsoletas, coalescing de velas abiertas, preservación de cierres bajo backpressure y destrucción no bloqueante desde `Drop`.

Todavía no implementa una política general de reconexión con backoff, recuperación/backfill de gaps ni estados completos de conexión para un provider remoto. Los errores realtime se exponen, pero la reconexión y la reconciliación de datos deberán diseñarse y probarse antes de depender de un feed real.

La primera implementación usa la biblioteca estándar y providers mock. No define endpoints, credenciales ni protocolos de exchange. La arquitectura debe funcionar en Windows, macOS y Linux.

## Consecuencias

### Positivas

- La UI no ejecuta I/O de providers ni espera sincrónicamente por su terminación al destruir workers.
- Histórico y realtime tienen ciclos de vida aislados.
- Las respuestas históricas tardías se descartan mediante generación e identidad.
- La memoria queda acotada por colas/mailboxes y políticas explícitas de coalescing y backpressure.
- No se añade una dependencia de runtime antes de necesitarla.

### Negativas y riesgos

- La gestión de threads, cancelación, errores y cierre añade complejidad explícita.
- `shutdown()` puede bloquear y debe reservarse para contextos aptos para esperar.
- El reaper evita bloquear `Drop`, pero no puede forzar la terminación de I/O no cooperativo.
- La cancelación de operaciones remotas depende de que el provider implemente timeouts finitos y soporte cooperativo.
- La reconexión y recuperación de gaps siguen siendo responsabilidades pendientes de los futuros adaptadores.

## Condiciones de revisión

Revisar esta decisión si se requiere multiplexar muchas conexiones, compartir un runtime con otras partes de la aplicación, o si la implementación estándar demuestra complejidad o coste operativo mayor que un runtime async.

Antes de incorporar un provider remoto de producción, deberán implementarse y probarse sus timeouts, cancelación durante I/O, reconexión/backoff, recuperación de gaps y cierre. La integración no debe trasladar esas responsabilidades al hilo UI.
