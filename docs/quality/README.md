# Calidad y validación

## Gates de Rust

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

## Gates del repositorio

```bash
python scripts/verify_no_code_comments.py
pwsh -File scripts/verify_architecture.ps1
pwsh -File scripts/audit_dependencies.ps1
```

## Estrategia de pruebas

- Unit tests para reglas y transformaciones locales.
- Integration tests para contratos entre capas.
- Property tests para transformaciones matemáticas y estados con invariantes.
- Regression tests para bugs corregidos.
- Benchmarks para rutas sensibles al rendimiento.

## Criterios

Una prueba debe ser determinista, aislada de servicios externos cuando sea posible y expresar comportamiento observable. La cobertura es una señal, no un sustituto de calidad de casos.
