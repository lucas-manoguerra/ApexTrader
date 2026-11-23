# Directrices de seguridad

## Validaciones

- Validá toda entrada externa (datos de mercado, órdenes, configuración).
- Usá tipos que impidan estados inválidos (por ejemplo, `Price`, `Quantity`, `OrderId`).
- Nunca confíes en datos del broker sin validar.
- Sanitizá cualquier dato antes de persistirlo o renderizarlo en la UI.

## Errores

- Tratá errores recuperables explícitamente con `Result`.
- No expongas información interna en mensajes de error al usuario.
- Registrá errores con `tracing` para depuración interna.

## Integridad de datos

- Verificá checksums o firmas cuando descargués datos de brokers.
- Persistí solo datos validados.
- Usá transacciones o escrituras atómicas para evitar corrupción.

## Recuperación

- El sistema tiene que poder reiniciarse desde un estado consistente.
- No pierdas órdenes o posiciones ante un cierre inesperado.
- Probá escenarios de fallo (disco lleno, red caída, broker inaccesible).

## Secretos

- Nunca comitees keys, tokens o credenciales en el repositorio.
- Usá variables de entorno o un gestor de secretos.
- `.env` está en `.gitignore`.
- Revisá antes de cada commit que no estés agregando ningún secreto accidental.

## Auditoría de dependencias (supply chain)

- Ejecutá `cargo audit` para detectar vulnerabilidades conocidas en dependencias.
- Ejecutá `cargo deny check` para auditar licencias, versiones yanked y crates no mantenidos.
- Las herramientas se invocan mediante `scripts/audit_dependencies.ps1`.
- Configuración en `cargo-deny.toml`: se deniegan licencias AGPL y se advierte sobre crates no actualizados en más de 2 años.
- Una vulnerabilidad o licencia prohibida bloquea el merge a `main` (ver CI).

## Alcance

Estas reglas aplican a todo código que entre al repositorio, incluyendo pruebas, scripts y herramientas de construcción.
