# Auditoría de cierre del PR #10

## Estado de la revisión

**Resultado:** fundaciones sustancialmente alineadas con el alcance acordado; revisión de cierre todavía no exhaustiva. El PR debe permanecer abierto y en borrador hasta resolver las observaciones de preparación para revisión y obtener revisión humana.

- PR: [#10](https://github.com/lucas-manoguerra/ApexTrader/pull/10)
- Base: `main` (`b14d400d23fd45d090d3a58055bc0d077ce227cb`)
- Head auditado: `7ee8a37877bcef8defa9e780469e39ffe142c754`
- Diferencia reportada por GitHub: 292 commits, 108 archivos, 3664 adiciones y 2083 eliminaciones.
- CI: ejecución [#45920252688](https://github.com/lucas-manoguerra/ApexTrader/actions/runs/45920252688), completada con **19/19 jobs exitosos**.

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

**Estado:** remediación aplicada; validación en CI del head actual.

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
**Evidencia:** `Cargo.toml` conserva `pollster`, `wgpu-types`, `raw-window-handle` y `winit`, aunque se retiró el renderer paralelo que justificaba parte de ellas. La limpieza intentada previamente se revirtió porque el lockfile no era reproducible con `--locked`.

**Acción recomendada:** confirmar consumidores reales mediante compilación y análisis de dependencias; eliminar solo las dependencias demostrablemente innecesarias y regenerar `Cargo.lock` con Cargo. No editar el lockfile manualmente ni conservar una limpieza que no pase CI con `--locked`.

### C-004 — Descripción histórica del PR

**Severidad:** media.  
**Evidencia:** el cuerpo anterior del PR conserva referencias a heads antiguos, ejecuciones en cola y próximos pasos ya completados.

**Acción recomendada:** sustituir el historial acumulativo por un resumen estable del alcance, decisiones, validación actual y limitaciones pendientes.

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

- [x] CI del head auditado verde en Windows, macOS y Linux.
- [x] WGPU seleccionado explícitamente como backend eframe; callback WGPU del chart integrado.
- [x] No se mantiene el renderer paralelo con lifecycle independiente.
- [x] Fallback histórico sintético implícito eliminado.
- [x] Decisiones y limitaciones principales registradas en ADR/auditorías.
- [x] Retirar la supresión global `dead_code`.
- [x] Corregir los hallazgos concretos expuestos por Clippy; CI del head corregido pendiente.
- [ ] Confirmar el destino de las dependencias directas candidatas a obsoletas.
- [ ] Sincronizar el cuerpo del PR con el head y CI vigentes.
- [ ] Completar revisión humana del diff y obtener aprobación requerida.
- [ ] Mantener explícito que runtime GPU y distribución no están certificados.

## 6. Recomendación

El PR #10 está **funcionalmente validado por CI y alineado en lo esencial con su objetivo de fundaciones de ingeniería**. No debe ampliarse para resolver toda la deuda arquitectónica del producto.

Antes de quitar el estado draft, conviene resolver C-001 o aceptar formalmente su aplazamiento, completar C-002/C-003 como alcance explícito de seguimiento, sincronizar la descripción y obtener una revisión completa del diff. No se recomienda mergear únicamente por tener CI verde.
