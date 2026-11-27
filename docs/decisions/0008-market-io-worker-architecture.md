# ADR-0008: Ejecución aislada para I/O de mercado

## Estado

Propuesta

## Contexto

La implementación actual ejecuta `MarketDataProvider::fetch_candles()` durante la construcción de `ApexTraderApp` y desde el callback de recarga de símbolo. También ejecuta `RealtimeMarketDataProvider::next_update()` dentro de `ApexTraderApp::ui`, con un sondeo cada 250 ms.

Los mocks actuales responden localmente, pero esos contratos permiten que una implementación remota bloquee el hilo de interfaz durante DNS, conexión, lectura, timeout, rate limit o reconexión. El problema afecta tanto la carga histórica como el flujo realtime.

La UI debe conservar la propiedad exclusiva de `ChartState`. Los workers solo reciben solicitudes y devuelven mensajes inmutables; nunca mutan el gráfico ni comparten referencias mutables a su estado.

## Alternativas consideradas

1. **Runtime async (por ejemplo, Tokio).** Ofrece I/O concurrente y ecosistema de red amplio, pero agrega una dependencia de runtime y decisiones de integración/ciclo de vida que todavía no son necesarias para los mocks ni para la primera integración.
2. **Un único worker para histórico y realtime.** Es simple, pero una carga histórica lenta puede retrasar el stream realtime y viceversa.
3. **Workers dedicados con primitivas de la biblioteca estándar.** Separa los ciclos de vida histórico y realtime, evita un runtime global y permite comunicación explícita y acotada.

## Decisión

Adoptar inicialmente workers dedicados basados en `std::thread`, canales acotados de `std::sync::mpsc` y señales de cancelación atómicas. No introducir Tokio ni otro runtime async como parte de esta frontera.

### Propiedad y responsabilidades

- La UI conserva `ChartState`, el renderer y la aplicación de cada resultado.
- Un worker de histórico ejecuta una solicitud de carga y devuelve un resultado completo o un error. Las solicitudes se identifican con un `generation_id` monotónico y la identidad del instrumento/serie.
- Un worker realtime posee su provider y su conexión durante una generación activa. La creación, polling, reconexión y destrucción del provider ocurren fuera del hilo UI.
- Los workers no reciben `ChartState`, `egui::Context` ni referencias a widgets.
- La UI drena mensajes con operaciones no bloqueantes y solicita repaint cuando recibe resultados o cuando hay trabajo pendiente.

### Cambio de símbolo y respuestas obsoletas

Cada cambio válido de símbolo incrementa la generación activa, cancela las tareas anteriores y solicita detener el worker realtime anterior. Todo mensaje incluye generación e identidad de serie. La UI descarta cualquier mensaje cuya generación o identidad no coincida con la selección vigente, incluso si la cancelación llegó después de que el provider terminara una operación.

La carga histórica nueva no debe esperar a que una petición anterior termine para poder invalidarla. Las llamadas remotas deben tener timeout finito; la cancelación cooperativa se comprueba antes y después de cada operación bloqueante. No se prometerá interrupción instantánea de una llamada de red que no soporte cancelación.

### Backpressure y semántica realtime

- Todos los canales entre workers y UI tienen capacidad finita y documentada; no se permiten colas ilimitadas.
- Las actualizaciones de una vela todavía abierta pueden coalescerse por identidad de serie y `open_time_ms`: solo hace falta conservar la versión más reciente pendiente.
- Un evento de cierre de vela no se descarta silenciosamente. Si el canal está lleno, el worker aplica backpressure cancelable o reporta un fallo recuperable que obliga a reconciliar/backfill antes de continuar.
- Los errores y cambios de estado (conectando, conectado, reconectando, detenido) son mensajes explícitos y no compiten con ticks reemplazables.
- La UI valida de nuevo símbolo, generación, tiempos y datos antes de mutar el gráfico. Los workers no interpolan huecos ni cambian las reglas de integridad existentes.

### Timeouts, reconexión y cierre

Los providers remotos deberán exponer operaciones con espera acotada o timeouts configurables, para que el worker pueda revisar cancelación y cierre. La política concreta de retry/backoff y recuperación de gaps pertenece al adaptador/proveedor, no a la UI.

Al cambiar de símbolo o cerrar la aplicación, se solicita cancelación, se cierran canales de entrada y se detienen los workers. El cierre debe ser determinista para providers que respeten sus timeouts; nunca se bloqueará el hilo UI esperando una operación remota sin límite. La estrategia de join y la propiedad de handles se implementarán en un coordinador de ciclo de vida, con tests específicos.

### Compatibilidad y alcance

La primera implementación usará solo la biblioteca estándar de Rust y conservará los providers mock. No define endpoints, credenciales ni protocolos de exchange. La arquitectura debe funcionar en Windows, macOS y Linux. La incorporación de un runtime async podrá reconsiderarse si el número de streams, la multiplexación o las necesidades de I/O lo justifican con evidencia.

## Consecuencias

### Positivas

- La UI no ejecuta I/O ni espera resultados de providers.
- Histórico y realtime no se bloquean mutuamente.
- Las respuestas tardías se aíslan mediante generación e identidad.
- La memoria queda acotada por canales y política de coalescing.
- No se añade una dependencia de runtime antes de necesitarla.

### Negativas y riesgos

- La gestión de threads, cancelación, errores y cierre añade complejidad explícita.
- La cancelación de una operación bloqueante depende de que el provider implemente timeouts finitos.
- La coalescencia debe distinguir cuidadosamente updates abiertos de cierres de vela.
- Los canales acotados requieren una política definida para saturación y estados críticos.

## Condiciones de revisión

Revisar esta decisión si se requiere multiplexar muchas conexiones, compartir un runtime con otras partes de la aplicación, o si la implementación estándar demuestra complejidad o coste operativo mayor que un runtime async. Antes de incorporar Binance, deberán estar implementados y probados los límites de timeout, cancelación, backpressure y cierre descritos aquí.
