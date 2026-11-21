# Directrices de seguridad

## Validaciones

- Validar toda entrada externa (datos de mercado, órdenes, configuración).
- Usar tipos que impidan estados inválidos (por ejemplo, `Price`, `Quantity`, `OrderId`).
- Nunca confiar en datos del broker sin validar.

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