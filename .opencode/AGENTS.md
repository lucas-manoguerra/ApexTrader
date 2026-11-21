# Guardrails de ApexTrader — OpenCode

Este archivo es vinculante para todo subagente de OpenCode que trabaje en este repositorio. Nadie se salta estos guardrails por ningún motivo.

## 1. Arquitectura y separación de responsabilidades (obligatorio)

Cada módulo tiene UNA responsabilidad principal identificable. Un módulo NO debe asumir responsabilidades de otro.

| Módulo | Responsabilidad |
|--------|-----------------|
| `domain` | Conceptos y reglas del mercado (velas, viewport) |
| `renderer` | Preparación y dibujo de elementos gráficos |
| `errors` | Errores tipados por subsistema |
| `logging` | Logging estructurado |

**Ejemplo prohibido:** una función del gráfico que descarga precios, calcula indicadores, modifica posiciones, actualiza la UI y guarda en disco.

## 2. Separación dominio / aplicación / infraestructura (obligatorio)

Las dependencias de diseño apuntan HACIA el dominio, nunca al revés.

```
Infraestructura → Aplicación → Dominio
```

- `src/domain/` NUNCA importa de `egui`, `wgpu`, `winit`, `raw_window_handle`, `pollster`, `bytemuck`.
- `src/renderer/` puede importar de `domain/` y de crates de infraestructura.
- `main.rs` es la capa de aplicación: coordina, no contiene reglas de negocio.

## 3. No acoplar lógica de negocio a la interfaz

- `egui` no determina cómo se calcula una media móvil.
- `wgpu` no conoce reglas de validación de órdenes.
- El dominio de mercado no depende de un proveedor de datos específico.

## 4. Guardrails de Rust (obligatorio en todo código escrito)

- **No** `unwrap()` ni `expect()` en caminos de producción normales.
- **No** `panic!()` para errores recuperables.
- **No** `unsafe` sin justificación técnica documentada y pruebas.
- No clonar datos grandes sin entender el costo.
- No usar `Arc<Mutex<T>>` automáticamente para todo problema de concurrencia.
- Preferir tipos que hagan imposibles los estados inválidos (ej. `Price` en lugar de `f64` crudo).
- APIs públicas pequeñas, explícitas y documentadas.
- Documentar APIs públicas y sus invariantes.

## 5. Rendimiento (obligatorio)

- No optimizar por intuición. Medir, identificar cuello de botella, optimizar.
- No introducir optimizaciones sin evidencia.
- No sacrificar claridad por microoptimizaciones irrelevantes.

## 6. Calidad del código — gates obligatorios (CI)

Cada cambio debe pasar antes de integrarse:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Además, el script `scripts/verify_architecture.ps1` verifica que `src/domain/` no importe de crates de infraestructura.

## 7. Git y commits

- Estrategia: trunk-based con ramas cortas (`feature/`, `fix/`, `refactor/`, `docs/`) desde `main`.
- Conventional Commits: `tipo(ámbito): descripción breve`.
  - Tipos: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`.
- Commits pequeños, coherentes, una unidad lógica de trabajo.
- **No** `Co-Authored-By` ni atribución de IA.
- Sin `unwrap`/`panic` en producción.

## 8. Skills de OpenCode

Antes de lanzar un subagente, cargar las skills relevantes del registro (`.atl/skill-registry.md`):
- `wgpu-rust-engineering` — para renderers, pipelines, shaders WGSL.
- `egui-immediate-mode-ui` — para interfaces egui.
- `eframe-app-framework` — para aplicaciones eframe.
- `thiserror-error-design` — para errores tipados.
- `tracing-instrumentation` / `tracing-subscriber-observability` — para logging.
- `bytemuck-*` — para conversión segura bytes/tipos.
- `pollster-async-bridge` — para futuros bloqueantes con wgpu.
- `protest-property-testing` — para pruebas basadas en propiedades.

## 9. Regla de oro

La simplicidad es una propiedad arquitectónica, no una excusa para evitar diseñar correctamente. Cada módulo tiene una responsabilidad. Las dependencias son explícitas y controladas. Ningún cambio se integra si rompe los gates de calidad.

## 10. Incumplimiento

Si un subagente o cualquier cambio viola estos guardrails, se rechaza y se corrige antes de continuar. No hay excepciones.