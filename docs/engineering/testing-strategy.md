# Estrategia de pruebas

## Niveles de prueba

| Nivel | Qué cubre | Herramienta |
|-------|-----------|-------------|
| Unitario | Un módulo o función aislada | `cargo test` |
| Integración | Varios módulos trabajando juntos | `cargo test` (tests/) |
| Propiedades | Invariantes que deben cumplirse para muchos casos | `protest` |
| Regresión | Que un cambio no rompa comportamiento previo | `cargo test` |

## Reglas

- Cada funcionalidad importante debe tener pruebas adecuadas.
- Cada test afirma todos los efectos observables de la regla (salida, código de salida, datos persistidos).
- Los tests cubren los casos que la regla misma nombra.
- Los tests pasan por la interfaz pública, no por detalles internos.
- Antes de implementar una corrección, observar RED (test fallando), luego implementar hasta GREEN.

## Cuándo NO escribir tests

- Documentación pasiva sin comportamiento verificable.
- Runners no disponibles.
- No hay un resultado esperado claro y determinista.

En esos casos, se justifica por escrito y se usan chequeos funcionales o estructurales proporcionados.