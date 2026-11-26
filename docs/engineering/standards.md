# Estándares de ingeniería

## Lenguaje y toolchain

- Rust estable, Edition 2024.
- Formato obligatorio mediante rustfmt.
- Clippy sin warnings en los targets cubiertos por CI.
- APIs públicas pequeñas, explícitas y con contratos verificables.
- No usar características nightly sin una decisión arquitectónica registrada.

## Diseño

- Aplicar cohesión alta y acoplamiento bajo.
- Mantener una sola fuente de verdad para cada estado o regla.
- Preferir composición y tipos explícitos.
- Evitar abstracciones especulativas y generalización prematura.
- Mantener límites de módulo alineados con responsabilidades de dominio.

## Errores

- Usar tipos de error específicos.
- Propagar errores recuperables con `Result`.
- No ocultar fallos mediante valores por defecto silenciosos.
- Reservar panic para invariantes internas que no puedan fallar por entradas externas.

## Memoria y rendimiento

- Preferir borrowing sobre clonación cuando el contrato lo permita.
- Evitar asignaciones por frame en rutas críticas.
- Evaluar complejidad algorítmica y costo de sincronización.
- Usar benchmarks reproducibles antes de afirmar mejoras.
- No sacrificar corrección ni legibilidad por microoptimizaciones no medidas.

## Concurrencia y seguridad

- Preferir primitivas seguras de Rust.
- Mantener el alcance de locks reducido y evitar bloqueos durante operaciones de I/O.
- Cualquier uso de `unsafe` requiere revisión especializada, límites mínimos y un caso de seguridad documentado fuera del código.
- Revisar dependencias por licencia, advisories, mantenimiento y superficie de ataque.

## Prohibición de comentarios

No se permiten comentarios dentro del código mantenido por ApexTrader: comentarios de línea, bloque, documentación, Rustdoc, TODO, FIXME ni código comentado. Esto incluye archivos fuente, scripts y configuración ejecutable. No se deben usar atributos Rustdoc como sustituto de comentarios.

La intención debe expresarse mediante nombres, tipos, módulos, interfaces, tests y documentación externa. La documentación externa puede explicar motivaciones, decisiones y procedimientos.

Los comentarios dentro de código generado, dependencias externas o contenido vendorizado se gestionan como material de terceros y no deben alterarse automáticamente.

## Documentación

- Documentar decisiones, contratos, procedimientos y arquitectura en Markdown.
- No duplicar la misma regla en múltiples fuentes canónicas.
- Mantener el estado real de funcionalidades y backends.
- Registrar cambios arquitectónicos relevantes mediante ADR.
