# Directrices de seguridad

## Validaciones

- Validar toda entrada externa (datos de mercado, órdenes, configuración).
- Usar tipos que impidan estados inválidos (por ejemplo, `Price`, `Quantity`, `OrderId`).
- Nunca confiar en datos del broker sin validar.
- Sanitizar cualquier dato antes de persistirlo o renderizarlo en la UI.

## Errores

- Tratar errores recuperables explícitamente con `Result`.
- No exponer información interna en mensajes de error al usuario.
- Registrar errores con `tracing` para depuración interna.

## Integridad de datos

- Verificar checksums o firmas cuando se descargan datos de brokers.
- Persistir solo datos validados.
- Usar transacciones o escrituras atómicas para evitar corrupción.

## Recuperación

- El sistema debe poder reiniciarse desde un estado consistente.
- No perder órdenes o posiciones ante un cierre inesperado.
- Probar escenarios de fallo (disco lleno, red caída, broker inaccesible).

## Secretos

- Nunca commit keys, tokens o credenciales en el repositorio.
- Usar variables de entorno o un gestor de secretos.
- `.env` está en `.gitignore`.
- Revisar antes de cada commit que no se esté agregando ningún secreto accidental.

## Auditoría de dependencias (supply chain)

- Ejecutar `cargo audit` para detectar vulnerabilidades conocidas en dependencias.
- Ejecutar `cargo deny check` para auditar licencias, versiones yanked y crates no mantenidos.
- Las herramientas se invocan mediante `scripts/audit_dependencies.ps1`.
- Configuración en `cargo-deny.toml`: se deniegan licencias AGPL y se advierte sobre crates no actualizados en más de 2 años.
- Un crato vulnerabilidad o licencia prohibida bloquea el merge a `main` (ver CI).

## Alcance

Estas reglas aplican a todo código que entre al repositorio, incluyendo pruebas, scripts y herramientas de construcción.