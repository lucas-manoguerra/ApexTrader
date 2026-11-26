# ADR-0005: Soporte desktop multiplataforma

## Estado

Aceptada como requisito de producto y arquitectura

## Contexto

ApexTrader debe ejecutarse como aplicación desktop en Windows, macOS y Linux. La compatibilidad no puede depender de portar el producto al final ni de asumir que una compilación exitosa en un único sistema demuestra soporte general.

La arquitectura gráfica adoptada en ADR-0003 utiliza eframe como owner del ciclo nativo y wgpu como backend principal. Esta decisión reduce la cantidad de código específico de plataforma, pero no elimina diferencias de drivers, superficies, escalado DPI, entrada, rutas, empaquetado, permisos ni distribución.

La CI existente ejecutaba sus controles Rust en Windows solamente. Por ello no proporcionaba evidencia de compilación ni pruebas en macOS y Linux.

## Decisión

Windows, macOS y Linux son plataformas objetivo de primera clase. La compatibilidad se valida progresivamente mediante niveles de evidencia explícitos:

1. **Portable by design:** dominio, casos de uso y modelos compartidos no dependen de APIs específicas del sistema operativo.
2. **Build verified:** CI compila los targets nativos y ejecuta controles Rust en Windows, macOS y Linux.
3. **Test verified:** las pruebas automatizadas pasan en cada sistema operativo; pruebas que requieren GPU/ventana se identifican y ejecutan en runners adecuados.
4. **Runtime verified:** la aplicación inicia, crea ventana/surface, presenta UI y chart WGPU, gestiona resize/DPI y termina correctamente en cada sistema.
5. **Distribution verified:** artefactos instalables, rutas de datos, firma/notarización cuando corresponda, actualización y desinstalación se validan por plataforma.

No se declara soporte completo de una plataforma hasta cumplir los niveles relevantes y documentar sus limitaciones.

## Responsabilidades y límites

- El dominio y los casos de uso no conocen el sistema operativo, window handles, rutas de usuario, APIs gráficas ni detalles de empaquetado.
- `bootstrap` compone los adaptadores concretos y la configuración de plataforma.
- eframe/winit poseen el ciclo de ventana y eventos mientras siga vigente ADR-0003.
- eframe/wgpu posee la surface y el ciclo de vida del device/queue compartido; el subsistema rendering posee únicamente los recursos y pipelines propios del chart.
- El acceso a directorios de configuración, datos, caché y logs debe usar APIs multiplataforma y políticas explícitas; no concatenar rutas manualmente ni asumir separadores.
- Entrada, DPI, escalado, suspensión/reanudación y cierre deben probarse como comportamiento observable, no inferirse de la compilación.
- Los formatos persistidos y datos financieros no dependen de locale ni de la zona horaria local del equipo.

## Estrategia CI

La matriz de CI debe incluir runners nativos `windows-latest`, `macos-latest` y `ubuntu-latest` para formato, compilación, Clippy, tests y documentación. Linux instala explícitamente las bibliotecas de desarrollo requeridas por la integración desktop. Los controles de gobernanza basados en PowerShell pueden mantenerse en Windows hasta que exista una necesidad concreta de hacerlos portables.

Una compilación cruzada no reemplaza una compilación nativa. Los tests unitarios no demuestran que un driver o surface funcione. La CI debe distinguir los controles sin GPU de las pruebas de integración gráfica que requieren display, software rendering o hardware real.

## Consecuencias

### Positivas

- Las regresiones específicas de plataforma se detectan antes.
- La arquitectura mantiene las particularidades del sistema fuera del dominio.
- La declaración de compatibilidad queda respaldada por evidencia diferenciada.
- La decisión de wgpu se valida en el contexto real de los tres sistemas operativos.

### Costes y riesgos

- Los runners macOS y Windows pueden ser más lentos y costosos que Linux.
- La disponibilidad de GPU y display en CI hospedada es limitada y no representa todos los drivers.
- Empaquetado, firma, notarización y distribución requieren pipelines y secretos específicos.
- Diferencias entre versiones de sistema operativo y drivers requieren una matriz de runtime más amplia que la matriz mínima de compilación.

## Condiciones de aceptación

- Los controles de formato, compilación, Clippy, tests y documentación corren en los tres sistemas.
- Los pasos de dependencias nativas Linux están declarados y reproducibles.
- Los límites de plataforma se documentan y no filtran al dominio.
- Existen pruebas de runtime para ventana, surface y presentación WGPU en cada sistema antes de declarar soporte gráfico completo.
- Los pipelines de distribución y sus artefactos se verifican antes de declarar soporte de instalación.

## Condiciones de revisión

Revisar esta decisión si cambia el framework desktop, el backend gráfico, los sistemas operativos objetivo o los requisitos de distribución. La matriz CI puede ajustarse por coste siempre que no se reduzca silenciosamente el nivel de evidencia declarado.
