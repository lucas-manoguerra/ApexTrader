# Calidad y validación

## Gates de Rust

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo doc --workspace --no-deps --locked
```

## Gates del repositorio

```bash
bash scripts/verify_architecture.sh
bash scripts/verify_lib_docs.sh
bash scripts/verify_no_code_comments.sh
bash scripts/audit_dependencies.sh
```

La auditoría de dependencias requiere `cargo-audit` y `cargo-deny`. CI instala estas herramientas explícitamente. Los scripts deben invocarse desde la raíz del repositorio con Bash.

## Plataformas

La matriz objetivo del proyecto es Windows, macOS y Linux. Mientras exista una exclusión temporal documentada de una plataforma en CI, los resultados de las plataformas activas no deben presentarse como cobertura completa multiplataforma. La exclusión y su reversión deben estar descritas en la PR correspondiente.

## Estrategia de pruebas

- Unit tests para reglas y transformaciones locales.
- Integration tests para contratos entre capas.
- Property tests para transformaciones matemáticas y estados con invariantes.
- Regression tests para bugs corregidos.
- Benchmarks para rutas sensibles al rendimiento.

## Criterios

Una prueba debe ser determinista, aislada de servicios externos cuando sea posible y expresar comportamiento observable. La cobertura es una señal, no un sustituto de calidad de casos. No se deben debilitar pruebas para hacer pasar CI.
