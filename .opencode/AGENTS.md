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

## 4. Regla inquebrantable: sin comentarios en el código (obligatorio)

No se escriben comentarios en el código fuente. El código debe ser autodocumentado mediante:

- Nombres de tipos, funciones, variables, módulos y campos que expresen intención.
- Estructura clara: un archivo, una responsabilidad; archivos pequeños y enfocados.
- Tests que pinchen el comportamiento observable (documentan el contrato por ejemplo).
- Tipos que hagan imposibles los estados inválidos (ej. `Price` en lugar de `f64` crudo).

Excepciones permitidas (y solo estas):

- Documentación de APIs públicas (`///` en items públicos) que describa el contrato, no la implementación.
- Justificación de `unsafe` requerida por el guardrail de `unsafe`.
- Atribución de librerías externas requerida por el guardrail de "Documentación obligatoria de librerías", pero solo en el `## Log` de la tarea ODD, nunca dentro del código fuente.

## 5. Documentación pública obligatoria: `///` y `//!` (obligatorio)

Todo item público (struct, enum, trait, función, método, constante, módulo) **debe** tener documentación `///` o `//!` profesional, actualizada y precisa.

Requisitos:

- **`///` en items públicos:** describir el contrato, el propósito y el comportamiento observable, no la implementación interna. Incluir parámetros, retorno, errores, panics e invariantes cuando aplique.
- **`//!` en módulos:** describir el propósito del módulo, su responsabilidad, cómo encaja en la arquitectura y las reglas de uso.
- **Mantener actualizado:** si cambia el comportamiento, la firma o el propósito de un item público, actualizar su documentación en el mismo commit. Nunca dejar documentación desactualizada.
- **Profesional:** escribir en español argentino (voseo), claro y conciso. Sin jargon innecesario, sin comentarios de implementación, sin "TODO", "FIXME" o notas internas.
- **No documentar lo obvio:** si el nombre ya expresa la intención, la documentación puede ser breve, pero nunca ausente en items públicos.

## 6. Guardrails de Rust (obligatorio en todo código escrito)

- **No** `unwrap()` ni `expect()` en caminos de producción normales.
- **No** `panic!()` para errores recuperables.
- **No** `unsafe` sin justificación técnica documentada y pruebas.
- No clonar datos grandes sin entender el costo.
- No usar `Arc<Mutex<T>>` automáticamente para todo problema de concurrencia.
- Preferir tipos que hagan imposibles los estados inválidos (ej. `Price` en lugar de `f64` crudo).
- APIs públicas pequeñas, explícitas y documentadas.
- Documentar APIs públicas y sus invariantes.

## 7. Rendimiento (obligatorio)

- No optimizar por intuición. Medir, identificar cuello de botella, optimizar.
- No introducir optimizaciones sin evidencia.
- No sacrificar claridad por microoptimizaciones irrelevantes.

## 8. Calidad del código — gates obligatorios (CI)

Cada cambio debe pasar antes de integrarse:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Además, el script `scripts/verify_architecture.ps1` verifica que `src/domain/` no importe de crates de infraestructura.

## 9. Git y commits

- Estrategia: trunk-based con ramas cortas (`feature/`, `fix/`, `refactor/`, `docs/`) desde `main`.
- Conventional Commits: `tipo(ámbito): descripción breve`.
  - Tipos: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`.
- Commits pequeños, coherentes, una unidad lógica de trabajo.
- **No** `Co-Authored-By` ni atribución de IA.
- Sin `unwrap`/`panic` en producción.

## 10. Documentación obligatoria de librerías antes de cualquier código (CRÍTICO)

Antes de escribir una sola línea de código o planificar una implementación que use una librería externa, el agente **debe** leer la documentación oficial de esa librería en su versión exacta declarada en `Cargo.toml`.

**Prohibido usar implementaciones anteriores, viejas o asumidas de cualquier librería.** La versión fuente de verdad es la resuelta en `Cargo.lock`.

### Procedimiento obligatorio

1. Identificar qué crates externos se van a usar (ej. `wgpu`, `egui`, `eframe`, `winit`, `bytemuck`, `pollster`, `thiserror`, `tracing`).
2. Verificar la versión exacta en `Cargo.toml` y `Cargo.lock`.
3. Leer la documentación oficial en `docs.rs/crate/version` para esa versión exacta.
4. Confirmar que las APIs, traits, structs y métodos que se van a usar existen en esa versión.
5. Registrar en el comentario del código o en el `## Log` de la tarea ODD qué versión se consultó y qué APIs se usaron.

### Ejemplo de entrada requerida en el código o log

```
// Librerías consultadas: wgpu 30.0.2 (docs.rs/wgpu/30.0.2)
// APIs usadas: Instance::new, request_adapter, request_device
// Justificación: wgpu 30 cambió la API de Surface y DeviceDescriptor vs 0.x
```

## 11. Skills de OpenCode

Antes de lanzar un subagente, cargar las skills relevantes del registro (`.atl/skill-registry.md`):
- `wgpu-rust-engineering` — para renderers, pipelines, shaders WGSL.
- `egui-immediate-mode-ui` — para interfaces egui.
- `eframe-app-framework` — para aplicaciones eframe.
- `thiserror-error-design` — para errores tipados.
- `tracing-instrumentation` / `tracing-subscriber-observability` — para logging.
- `bytemuck-*` — para conversión segura bytes/tipos.
- `pollster-async-bridge` — para futuros bloqueantes con wgpu.
- `protest-property-testing` — para pruebas basadas en propiedades.

## 12. Regla de oro

La simplicidad es una propiedad arquitectónica, no una excusa para evitar diseñar correctamente. Cada módulo tiene una responsabilidad. Las dependencias son explícitas y controladas. Ningún cambio se integra si rompe los gates de calidad.

## 13. Incumplimiento

Si un subagente o cualquier cambio viola estos guardrails, se rechaza y se corrige antes de continuar. No hay excepciones.