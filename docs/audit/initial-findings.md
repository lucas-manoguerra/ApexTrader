# Auditoría técnica de ApexTrader

## Estado

Esta auditoría es incremental. Este documento registra hallazgos verificados en la implementación inspeccionada y define el método para completar el inventario. No representa una certificación de la arquitectura ni una revisión exhaustiva de todos los módulos.

## Criterios

- Separación explícita de responsabilidades y límites de contexto.
- Dependencias dirigidas hacia contratos estables.
- Dominio independiente de frameworks y detalles de infraestructura.
- Casos de uso pequeños, testeables y sin responsabilidades de presentación.
- Errores tipados y tratamiento explícito de fallos.
- Rendimiento medido con benchmarks reproducibles.
- Seguridad y validación en los límites de entrada.
- Ausencia total de comentarios en código, verificada automáticamente.
- Cambios incrementales con pruebas y CI.

## Hallazgos confirmados

### A-001 — La capa de aplicación concentra responsabilidades de presentación y orquestación

**Severidad:** crítica  
**Evidencia:** `src/application/services/mod.rs`

El módulo declara `ApexTraderApp` e implementa directamente `eframe::App`. En el mismo módulo se coordinan widgets de egui, interacción, selección de símbolo, carga de velas, cálculo de escalas, invalidación, renderizado y actualizaciones realtime. También instancia proveedores mock concretos.

**Impacto:** los casos de uso quedan acoplados a la UI y a implementaciones concretas de mercado. Esto dificulta probar la orquestación sin interfaz, sustituir proveedores y evolucionar la presentación independientemente.

**Dirección recomendada:** separar el adaptador de presentación, el estado de pantalla, los casos de uso de carga/actualización y los adaptadores de proveedores. Definir puertos de aplicación antes de mover responsabilidades. Mantener la lógica de dominio fuera de la UI.

### A-002 — La comprobación de arquitectura cubre solo una regla parcial

**Severidad:** alta  
**Evidencia:** `scripts/verify_architecture.ps1`

El verificador actual busca dependencias de crates de infraestructura dentro de `src/domain`. No detecta ciclos entre módulos propios, dependencias entre capas no permitidas, acceso desde dominio a otros módulos internos externos al dominio ni dependencias de presentación filtradas a aplicación.

**Impacto:** el control puede pasar mientras se incumplen otras reglas descritas en la documentación.

**Dirección recomendada:** definir una matriz explícita de dependencias permitidas y prohibidas; construir un grafo de módulos a partir de imports; detectar ciclos y validar todas las capas. Añadir pruebas del propio verificador con fixtures válidos e inválidos.

### A-003 — La crate permite silenciosamente código muerto

**Severidad:** alta  
**Evidencia:** `src/lib.rs`

La raíz de la biblioteca aplica `#![allow(dead_code)]` globalmente.

**Impacto:** se suprimen advertencias útiles de código sin uso, ocultando APIs abandonadas, implementaciones incompletas y deuda de mantenimiento.

**Dirección recomendada:** retirar la supresión global y resolver advertencias por caso. Las excepciones locales deben ser acotadas, justificadas fuera del código y temporales; no deben convertirse en una política general.

### A-004 — El contrato de renderizado depende de egui

**Severidad:** alta  
**Evidencia:** `src/renderer/mod.rs`

El trait `ChartRenderer` recibe `egui::Painter` y `egui::Rect`. Por tanto, el contrato que debería permitir intercambiar backends depende directamente del framework de UI.

**Impacto:** los backends alternativos y los tests de renderizado quedan condicionados por egui. La abstracción no representa una frontera independiente del framework.

**Dirección recomendada:** evaluar un contrato de destino de renderizado neutral basado en geometría y comandos propios. El adaptador egui debe traducir ese contrato a primitivas de egui; un backend GPU debe implementar el mismo contrato sin introducir tipos de UI.

### A-005 — El manifiesto mezcla dependencias de presentación, GPU y sistema de ventanas

**Severidad:** media  
**Evidencia:** `Cargo.toml`

La crate principal declara eframe/egui junto con wgpu, winit, raw-window-handle, pollster y dependencias de geometría/GPU. Parte de esas dependencias puede ser necesaria para el backend experimental, pero la configuración actual no demuestra una separación de compilación o de características.

**Impacto:** aumenta el acoplamiento y potencialmente el tiempo de compilación, el árbol de dependencias y la superficie de mantenimiento.

**Dirección recomendada:** medir primero el grafo y los tiempos de compilación; después evaluar features y crates separados solo si existe una frontera de producto y despliegue real. No extraer crates únicamente por estética.

## Riesgos que requieren investigación

- Ciclos entre módulos y dependencias internas no permitidas.
- Límites entre dominio, aplicación, chart, mercado y renderizador.
- Complejidad y tamaño de funciones, especialmente en la aplicación.
- Uso de tipos flotantes para precios, cantidades y precisión financiera.
- Orden, duplicados, huecos temporales y consistencia de datos de mercado.
- Semántica de concurrencia y actualización realtime.
- Gestión de errores, recuperación y observabilidad.
- Coste de asignaciones, copias, cálculos repetidos y renderizado por frame.
- Cobertura real de pruebas y propiedades invariantes.
- Seguridad de dependencias, licencias y cadena de suministro.
- Calidad de la implementación experimental de wgpu frente al backend funcional.

## Secuencia de auditoría

1. Inventariar todos los archivos, módulos, binarios, benchmarks, tests y dependencias.
2. Construir el grafo real de imports y contrastarlo con las reglas arquitectónicas.
3. Revisar cada módulo por responsabilidad, API pública, acoplamiento y complejidad.
4. Auditar el flujo de datos desde proveedor hasta dominio, aplicación y renderizador.
5. Auditar precisión numérica, tiempo, validación y consistencia de datos financieros.
6. Establecer una línea base de tests, lint, seguridad y rendimiento.
7. Priorizar los cambios por impacto, riesgo y coste; ejecutar refactors pequeños y verificables.

## Política de cambios

No se reorganizarán directorios ni se extraerán crates hasta tener el mapa de dependencias y las responsabilidades actuales documentadas. Cada refactor deberá incluir pruebas adecuadas y mantener la política global de cero comentarios en código.


6. **A-006 alto**: El límite público de `application` está acoplado a `eframe`. `src/application/mod.rs` expone `run() -> eframe::Result`, y `src/application/events/mod.rs` importa egui para convertir input en eventos. Impacto: el núcleo de aplicación no puede ejecutarse o probarse sin el framework de presentación y los adaptadores de UI no están claramente delimitados. Dirección: mover el arranque nativo a un adaptador de presentación/binario y mantener los casos de uso con contratos neutrales; conservar el traductor egui en la capa de presentación.

7. **A-007 medio**: La validación temporal de `MarketCandle` no garantiza que la conversión a punto de gráfico tenga éxito. `MarketCandle::is_valid` permite timestamps iguales (`close_time_ms == open_time_ms`), mientras que `to_chart_data_point` deriva un intervalo que `TimePoint::new` rechaza si no es positivo. Además, la resta directa `close_time_ms - open_time_ms` puede desbordar para valores extremos de `i64`. Impacto: un registro declarado válido puede fallar en la conversión y los extremos temporales pueden provocar overflow en vez de un error de dominio. Dirección: definir el contrato temporal de `MarketCandle`, alinear `is_valid` y conversión, y usar `checked_sub` con errores explícitos. No se debe imponer que `TimePoint.interval_ms` sea igual a la diferencia entre timestamps sin resolver primero su semántica: las pruebas actuales construyen intencionalmente un intervalo nominal distinto del span temporal.

8. **A-008 medio**: Las invariantes de `TimePoint` pueden eludirse porque sus campos son públicos. Aunque `TimePoint::new` rechaza rangos invertidos e intervalos no positivos, `open_time_ms`, `close_time_ms` e `interval_ms` son campos públicos y permiten construir el struct directamente o mutarlo después de validarlo. `ChartState::add_data_point` y `add_data_points` aceptan esos valores sin validación temporal adicional. Impacto: el estado del gráfico puede almacenar puntos que no cumplen las reglas del constructor, debilitando las garantías del dominio y complicando el razonamiento sobre orden y escalas temporales. Dirección: encapsular los campos, exponer getters y asegurar que todos los caminos de ingreso validen los datos antes de modificar el estado.
