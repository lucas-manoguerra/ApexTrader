---
name: tracing-subscriber-observability
description: Composición de subscribers/layers, formatos, filtros, salida JSON y configuración de observabilidad.
---

# tracing-subscriber-observability

## Misión

Actúa como ingeniero senior de observabilidad Rust especializado en `tracing-subscriber`. Diseña un pipeline de diagnóstico configurable, de bajo costo y seguro: subscriber, layers, filtros, formatos, destinos y comportamiento por entorno.

## Puerta de compatibilidad

- Comprueba versiones `tracing` y `tracing-subscriber` en Cargo y su compatibilidad.
- La versión observada de `tracing-subscriber` es 0.3.23. Revisa features en el paquete real: `std`, `alloc`, `env-filter`, `fmt`, `ansi`, `registry`, `json`, `tracing-log` y otras pueden variar y algunas son opcionales.
- Asegúrate de que los features que importas están habilitados; no supongas que `EnvFilter`, JSON o `fmt` existen con cualquier combinación.
- Confirma firmas y comportamiento de `registry`, `Layer`, `EnvFilter`, `fmt`, `reload` y `try_init` en la versión resuelta.

## Separación de responsabilidades

`tracing` emite spans y events desde librerías y aplicaciones. `tracing-subscriber` decide cómo recogerlos, filtrarlos, formatearlos y enviarlos. La instalación del subscriber global pertenece normalmente al binario/entrypoint, no a una librería reusable.

## Pipeline recomendado

- Comienza con un `Registry` y agrega layers por responsabilidad: formatter de consola, formatter JSON, métricas o exportador si el producto los necesita.
- Construye filtros explícitos por entorno y por módulo/target. Usa niveles sensatos: `ERROR`/`WARN` para problemas, `INFO` para operaciones de alto nivel, `DEBUG` para diagnóstico y `TRACE` para detalle intensivo.
- Usa `EnvFilter` si la feature está habilitada. Define un fallback conocido cuando la variable de entorno no existe o tiene directivas inválidas.
- Configura `fmt` con timestamps, target, thread/span context y ANSI solo cuando el destino soporte terminal.
- Usa salida JSON en producción cuando vaya a ser consumida por sistemas de logs; evita mezclar JSON y texto humano en el mismo stream.
- Si la configuración necesita cambiar en runtime, evalúa `reload` y documenta quién puede cambiarla y con qué límites.

## Inicialización

- Instala el subscriber global una vez, al inicio del proceso, antes de iniciar tareas que emitan eventos.
- Usa `try_init`/mecanismo equivalente para detectar que ya existe un subscriber. No ignores el error de inicialización sin justificarlo.
- En librerías, nunca fuerces `set_global_default`; deja que el consumidor elija el pipeline.
- Para tests paralelos, evita reinicializar globalmente en cada test. Usa subscriber por scope, utilidades de captura o una estrategia de testing compatible con concurrencia.

## Configuración y seguridad

- Trata `RUST_LOG`/directivas equivalentes como configuración no confiable: parsing con fallback y límites.
- No registres secretos, credenciales, tokens, claves API, contenido privado de órdenes ni datos personales.
- Evita filtros demasiado amplios en producción que produzcan volumen/costo excesivo.
- Evita targets con cardinalidad ilimitada (IDs únicos en target o nombres de span dinámicos); pon los valores en campos cuando sea apropiado y evalúa agregación.
- Diferencia nivel global y filtro por layer. No asumas que un layer ve exactamente los mismos eventos que otro.
- Protege archivos/streams de log: permisos, rotación, retención y volumen son parte de la operación, aunque el backend final requiera otra crate.

## Capas y composición

- Usa `Layer` para añadir comportamiento composable, no dupliques subscribers completos para cada destino.
- Separa filtros de formato y destino; deja claro qué eventos llega a cada backend.
- Controla orden y compatibilidad de layers con el `Registry` y los bounds reales.
- Para un layer propio, evita bloquear el hilo que emite eventos y evita locks largos.
- No realices red/IO síncrono costoso dentro de callbacks de un layer; usa colas acotadas y define qué pasa ante backpressure.

## Formatos y entorno

- Desarrollo: consola legible, color solo en TTY y filtros configurables.
- Producción: JSON estructurado o formato requerido por la plataforma, timestamp consistente y campos estables.
- Tests: captura controlada o subscriber scoped; asserts sobre campos semánticos, evitando atarse a espacios/formato incidental.
- CLI: permite sobrescribir filtros y destinos, documentando defaults.

## Integración con tracing

- Alinea convenciones de nombres de target, spans y campos con la skill `tracing`.
- No dupliques información en cada evento si ya vive en un span padre.
- Usa campos tipados y nombres estables para consulta y agregación.
- Recuerda que los eventos de `tracing` no se muestran si ningún subscriber está instalado.

## Anti-patrones

- Instalar un subscriber global desde una librería.
- Llamar a init en cada módulo/test.
- Ignorar errores `already set` sin documentar.
- Emitir logs de producción sin filtros ni límites.
- Usar formato JSON en terminal interactiva sin una necesidad.
- Hacer IO bloqueante/red dentro de un `Layer`.
- Registrar secretos o campos de alta cardinalidad sin control.

## Flujo del agente

1. Identifica aplicación, entorno, destinos y nivel de sensibilidad.
2. Comprueba features Cargo y versión exacta.
3. Diseña layers y filtros independientes.
4. Instala pipeline global una vez en el entrypoint.
5. Define fallbacks de configuración y política de secretos.
6. Añade tests de parsing, filtros, formato y captura.
7. Mide throughput y backpressure con carga representativa.

## Criterios de aceptación

- Subscriber configurado una sola vez desde el binario.
- Capas y filtros tienen responsabilidad explícita.
- Features necesarias están habilitadas de forma deliberada.
- No se filtran secretos ni se generan volúmenes sin control.
- Tests y ejecución de desarrollo/producción tienen comportamiento definido.
