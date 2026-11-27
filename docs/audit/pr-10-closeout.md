# Auditoría de cierre del PR #10

## Estado de la revisión

**Resultado:** fundaciones sustancialmente alineadas con el alcance acordado; revisión de cierre todavía no exhaustiva. La CI #45921206092 del head anterior está verde (19/19). La corrección de encapsulación temporal tuvo un fallo inicial por un acceso de campo residual, ya corregido en `35d6a3a3cb996df7cedac36ea60564d3ba81f7ec`. La CI #45922397027 corresponde al head documental `20af5c78bea2ff1c04d6754b80d2396c817e1784` y está pendiente. El PR debe permanecer abierto y en borrador hasta validar ese head.

- PR: [#10](https://github.com/lucas-manoguerra/ApexTrader/pull/10)
- Base: `main` (`b14d400d23fd45d090d3a58055bc0d077ce227cb`)
- Head auditado: `7ee8a37877bcef8defa9e780469e39ffe142c754`
- Diferencia reportada por GitHub: 292 commits, 108 archivos, 3664 adiciones y 2083 eliminaciones.
- CI anterior: [#45921206092](https://github.com/lucas-manoguerra/ApexTrader/actions/runs/45921206092), **19/19 jobs exitosos**. CI del head actual: [#45922397027](https://github.com/lucas-manoguerra/ApexTrader/actions/runs/45922397027), pendiente al actualizar esta auditoría.

Esta revisión combina inspección de los archivos de gobernanza, workflow CI, ADR principales, auditorías, manifiesto, bootstrap, módulos de presentación/rendering y comparación completa de nombres/estadísticas de archivos. No equivale a una revisión línea por línea de las 108 diferencias ni a una ejecución local de Cargo.

## 1. Evidencia CI

La ejecución #45920252688 sobre el head auditado finalizó correctamente:

- Formato: Windows, macOS y Linux.
- Compilación debug y release: Windows, macOS y Linux.
- Clippy: Windows, macOS y Linux.
- Tests: Windows, macOS y Linux.
- Documentación: Windows, macOS y Linux.
- Seguridad de dependencias.
- Verificación arquitectónica actual.
- Referencias de documentación de la biblioteca.
- Política de cero comentarios.

La CI acredita compilación y pruebas automatizadas en runners nativos. **No acredita** ejecución gráfica con GPU/driver real, clipping visual, resize, recuperación del dispositivo, DPI real ni distribución instalable.

## 2. Evaluación por alcance

| Área | Evaluación | Estado |
|---|---|---|
| Gobernanza de agentes y contribuciones | Contratos centrales y guías coherentes | Aceptable |
| ADR y documentación técnica | Decisiones principales registradas; algunos documentos son snapshots históricos | Aceptable con mantenimiento |
| Separación bootstrap/application/presentation | Se retiró el arranque desktop de application y se movió el adaptador egui | Parcial, adecuado para esta fase |
| WGPU como backend principal | eframe selecciona WGPU y el chart usa callback/pipeline propio | Implementado; runtime GPU pendiente |
| Recursos WGPU | Recursos por callback, buffer persistente, poda por owner, invalidación y límites documentados | CI verificada; hardware pendiente |
| Integridad de mercado | Fallback histórico sintético implícito eliminado | Resolución parcial; simulación aún no aislada |
| Multiplataforma | CI nativa Windows/macOS/Linux | Build/test verificados, runtime/distribución pendientes |
| Modularidad interna | Auditorías y ownership definidos; aplicación desktop aún concentra responsabilidades | Deferir refactor amplio |
| Modelo numérico y temporal | Riesgos y alternativas documentados sin imponer semánticas no acordadas | Deferir implementación |
| Seguridad y dependencias | cargo-audit/cargo-deny pasan en CI | Aceptable; limpieza de dependencias directas pendiente de confirmar |

## 3. Observaciones que requieren decisión antes de declarar el PR listo

### C-001 — Supresión global de código muerto

**Estado:** remediado; validado por CI #45921206092 (19/19) en el head anterior y pendiente de revalidación del head actual.

**Severidad:** alta para calidad de ingeniería.  
**Evidencia:** `src/lib.rs` ya no contiene `#![allow(dead_code)]`. La CI posterior detectó dos variantes de `LoadState` sin uso y dos funciones de prueba sin atributo `#[test]`. Se eliminaron las variantes `Idle` y `Loading` y se añadieron los atributos faltantes en `controller.rs` y `viewport.rs`.

La supresión global ocultaba código sin uso y debilitaba la señal de calidad. Se mantiene retirada. La corrección no introduce excepciones locales para silenciar Clippy.

**Criterio:** mantener la supresión retirada y exigir Clippy verde en los tres sistemas operativos.

### C-002 — Verificador de arquitectura con cobertura limitada

**Severidad:** media-alta.  
**Evidencia:** `scripts/verify_architecture.ps1` solo busca determinados crates de infraestructura bajo `src/domain`.

El control no verifica la matriz completa de capas, ciclos internos ni dependencias desde presentation/application/market/renderer. La documentación reconoce esta limitación.

**Acción recomendada:** mantener el alcance de la afirmación CI limitado a la regla que realmente se comprueba. Diseñar una mejora incremental con fixtures positivos/negativos y detección de ciclos en un PR específico; no presentar el verificador actual como garantía arquitectónica integral.

### C-003 — Dependencias directas candidatas a obsoletas

**Severidad:** media.  
**Estado:** auditoría estática de consumidores completada; limpieza del manifiesto diferida por falta de un entorno Cargo disponible para regenerar y validar el lockfile.

**Evidencia:** se inspeccionaron los 61 archivos Rust del head actual. No se encontraron consumidores directos de `egui`, `pollster`, `wgpu`, `wgpu-types`, `raw-window-handle` ni `winit`. Los usos de egui y wgpu en el código activo se realizan mediante los reexports `eframe::egui` y `eframe::wgpu`. `bytemuck_derive` sí tiene uso directo (`Pod`/`Zeroable`); `bytemuck` también se usa en el callback WGPU. `Cargo.lock` conserva todas las dependencias como dependencias directas del paquete raíz.

La inspección de fuentes identifica candidatos claros, pero no sustituye `cargo tree`, `cargo check` ni la regeneración real del lockfile. La CI ejecuta comandos con `--locked`, y no se dispone de Cargo local en este entorno para producir un lockfile reproducible.

**Acción recomendada:** en un entorno con la versión fijada por `rust-toolchain.toml`, retirar las dependencias confirmadas como no usadas (`egui`, `pollster`, `wgpu`, `wgpu-types`, `raw-window-handle`, `winit`), regenerar `Cargo.lock` con Cargo y verificar el diff de resolución. Mantener `eframe` con feature `wgpu`; no retirar `bytemuck` ni `bytemuck_derive`. No editar el lockfile manualmente ni conservar una limpieza que no pase CI con `--locked`.

### C-005 — Encapsulación de invariantes temporales

**Severidad:** alta para integridad del dominio.  
**Estado:** cambios implementados; CI del head actual pendiente.

**Evidencia:** `TimePoint` mantiene privados apertura, cierre e intervalo nominal y expone getters. Se actualizaron los consumidores del dominio, mercado, aplicación, presentación y renderer. La primera CI encontró un acceso residual en `presentation/desktop/app.rs`; se corrigió mediante `open_time_ms()` en `35d6a3a3cb996df7cedac36ea60564d3ba81f7ec`. El primer intento de CI no validó el head completo porque fue cancelado al entrar commits posteriores.

**Semántica preservada:** el intervalo nominal no se fuerza a coincidir con el span temporal sin un contrato de proveedor/timeframe. La revisión detallada de unidad, DST, calendarios, zonas horarias y fronteras de proveedor está en `docs/audit/temporal-model-review.md`, enlazada desde ADR-0004.

**Criterio:** CI verde sobre el head más reciente y mantener la encapsulación sin accesos directos externos.

### C-004 — Descripción histórica del PR

**Severidad:** media.  
**Estado:** remediado en la descripción actual del PR.

**Evidencia:** el cuerpo del PR fue reemplazado por un resumen estable del alcance, decisiones, validación y limitaciones. Debe mantenerse sincronizado si cambia el head o la CI.

## 4. Trabajo que debe permanecer fuera del PR #10

Los siguientes temas son deuda reconocida, no criterios para expandir indefinidamente esta entrega:

1. Descomposición interna de `presentation::desktop::app` por ownership y casos de uso.
2. Puertos de application para proveedores y contratos canónicos de datos de mercado.
3. Aislamiento explícito de simulación frente a datos de mercado reales.
4. Identidad canónica de instrumento/serie, timeframe, venue, proveedor y tipo de precio.
5. Modelo numérico financiero autoritativo, escalas, ticks/lots y redondeo.
6. Reconciliación histórico/realtime, velas provisionales/cerradas/corregidas y calidad de ingestión.
7. Calendarios y sesiones de mercado, zonas horarias y DST.
8. Validación visual WGPU con hardware/driver real en Windows, macOS y Linux, incluyendo clipping, DPI, resize y recuperación.
9. Persistencia, configuración, instaladores, firma, actualización y distribución.
10. Benchmarks y optimización del renderer con perfiles representativos.

Cada tema debe entrar en un PR propio con contrato, pruebas y criterios de aceptación verificables.

## 5. Criterios para cerrar esta etapa

- [x] CI de una revisión anterior verde en Windows, macOS y Linux.
- [ ] CI del head actual verde en Windows, macOS y Linux.
- [x] WGPU seleccionado explícitamente como backend eframe; callback WGPU del chart integrado.
- [x] No se mantiene el renderer paralelo con lifecycle independiente.
- [x] Fallback histórico sintético implícito eliminado.
- [x] Decisiones y limitaciones principales registradas en ADR/auditorías.
- [x] Retirar la supresión global `dead_code`.
- [x] Corregir los hallazgos concretos expuestos por Clippy; CI #45921206092 verde (19/19) en el head anterior.
- [x] Auditar estáticamente los consumidores de dependencias directas candidatas; limpieza del manifiesto/lockfile diferida hasta disponer de Cargo.
- [x] Sincronizar el cuerpo del PR con el estado conocido de head y CI.
- [ ] Validar la encapsulación de `TimePoint` y auditoría temporal con CI del head actual.
- [ ] Completar revisión humana del diff y obtener aprobación requerida.
- [ ] Mantener explícito que runtime GPU y distribución no están certificados.

## 6. Recomendación

El PR #10 está **alineado en lo esencial con su objetivo de fundaciones de ingeniería**, pero el head actual todavía no está validado por CI tras encapsular `TimePoint`. La auditoría temporal documenta los contratos pendientes sin añadir dependencias ni cambiar el runtime. No debe ampliarse para resolver toda la deuda arquitectónica del producto.

Antes de quitar el estado draft, conviene decidir si la limpieza reproducible de dependencias C-003 se realizará en este PR o en uno específico, y completar C-002 como seguimiento explícito. Se requiere revisión humana completa del diff. No se recomienda mergear únicamente por tener CI verde.
