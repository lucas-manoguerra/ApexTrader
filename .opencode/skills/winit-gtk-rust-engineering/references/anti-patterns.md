# Anti-patrones y correcciones

| Anti-patrón | Por qué es peligroso | Enfoque correcto |
|---|---|---|
| Tratar `winit-gtk` como upstream `winit` | El fork declara funciones faltantes y cambios específicos | Verificar source y docs de la versión fijada |
| Importar un supuesto `winit::platform::gtk` | El módulo puede no existir en la API publicada | Inspeccionar el módulo real; en 0.29.1 hay extensiones GTK en `platform::unix` |
| Mezclar ejemplos de winit 0.29 y 0.30 | Las firmas del event loop y los eventos cambiaron | Mantener ejemplos versionados |
| Invocar métodos GTK desde un worker | Puede provocar panic o violar la afinidad del contexto | Enviar `UserEvent` al hilo UI |
| Compartir `EventLoop` por `Arc` | `EventLoop` no es `Send` ni `Sync` | Mantenerlo en el hilo propietario y usar proxy |
| Ejecutar I/O en callbacks | Congela la interfaz y retrasa eventos | Workers + mensajes asíncronos |
| Usar `Poll` sin necesidad | Consume CPU y energía constantemente | `Wait`/`WaitUntil` y redraw bajo demanda |
| Renderizar en cada evento | Trabajo duplicado y latencia variable | Dirty state y `RedrawRequested` |
| Mezclar coordenadas lógicas y físicas | Errores de escala y posicionamiento | Tipos/nombres explícitos y conversión única |
| Suponer que `wgpu` y `winit-gtk` usan el mismo `raw-window-handle` | Fallos de compilación o integración | Revisar `cargo tree` y manifiestos antes de escribir Surface |
| Convertir handles con `transmute` | Riesgo de UB y errores por plataforma | Usar combinación soportada o fork migrado con revisión |
| Usar `unwrap()` en inicialización de plataforma | Fallo abrupto sin contexto útil | Propagar errores con contexto y política de recuperación |
| Hacer depender el dominio de GTK | Acoplamiento y migraciones costosas | Adaptadores y tipos propios |
| Suponer que todos los monitores reportan primary | Algunos backends devuelven `None` | Tratar `Option` y consultar capacidades |
| Probar únicamente con `cargo check` | No valida comportamiento real de display, GTK o GPU | Smoke tests en targets reales |
