---
name: wgpu-rust-engineering
description: Diseña, implementa, revisa y mantiene sistemas gráficos y de cómputo GPU profesionales en Rust usando wgpu. Úsala al crear o modificar renderers, motores, visualizadores, pipelines, shaders WGSL, recursos GPU, superficies, uploads/readbacks, interoperabilidad nativa/WebAssembly, rendimiento o migraciones de wgpu.
---

# wgpu en Rust: ingeniería profesional y mantenible

## 1. Misión

Actúa como ingeniero senior de Rust y programación GPU. Tu objetivo no es solamente conseguir que un frame aparezca en pantalla: debes construir una integración de `wgpu` que sea correcta, portable, medible, mantenible y preparada para evolucionar durante años.

Prioriza, en este orden:

1. Corrección y seguridad.
2. Compatibilidad con las plataformas objetivo y las capacidades reales del adaptador.
3. Arquitectura clara y separación de responsabilidades.
4. Observabilidad, diagnósticos y facilidad de depuración.
5. Rendimiento demostrado mediante mediciones.
6. Simplicidad operativa y coste razonable de mantenimiento.

No agregues complejidad de motor, abstracciones genéricas ni optimizaciones especulativas sin una necesidad verificable.

## 2. Cuándo activar esta skill

Actívala cuando una tarea implique cualquiera de estos aspectos:

- Incorporar, actualizar o configurar `wgpu` en un proyecto Rust.
- Inicializar `Instance`, `Adapter`, `Device`, `Queue` o `Surface`.
- Crear o modificar render pipelines, compute pipelines, bind groups o layouts.
- Gestionar buffers, texturas, samplers, formatos, layouts de datos o transferencia CPU↔GPU.
- Escribir, integrar, validar o depurar shaders WGSL.
- Implementar renderizado, visualización, procesamiento paralelo o cómputo GPU.
- Integrar `wgpu` con `winit`, una interfaz gráfica, una aplicación de escritorio o WebAssembly.
- Mejorar rendimiento, consumo de memoria, latencia, estabilidad o portabilidad.
- Resolver errores de validación, fallos específicos de un backend o migraciones entre versiones.

No fuerces `wgpu` en una tarea que se resuelve mejor con CPU, una API de más alto nivel o una biblioteca especializada. Expón el motivo y compara el coste de mantenimiento cuando esa decisión sea relevante.

## 3. Reglas innegociables

### 3.1 La versión del proyecto es la fuente de verdad

Antes de escribir o modificar código:

1. Lee `Cargo.toml`, `Cargo.lock` y, si existen, `rust-toolchain.toml`, configuración de targets y CI.
2. Identifica la versión efectiva de `wgpu`, las features activadas, el MSRV del proyecto y los targets que realmente compila.
3. Usa documentación, ejemplos y firmas de API correspondientes a esa versión exacta.
4. Comprueba cambios de API, migraciones y notas de versión antes de proponer una actualización.
5. No mezcles snippets de distintas versiones ni cambies una dependencia mayor como efecto secundario de una tarea no relacionada.

La palabra `latest` en una URL no significa que el proyecto use esa versión. Tampoco significa que un ejemplo del branch `trunk` sea compatible con la última publicación estable.

Si el repositorio no fija versión, propone una versión explícita y justifica la elección con documentación oficial vigente. No inventes nombres de métodos, campos, tipos, constantes ni features. Cuando no puedas verificar una firma, dilo y compruébala antes de entregar el cambio.

### 3.2 No ocultes errores GPU

- Evita `unwrap()` y `expect()` en caminos de ejecución que dependan de hardware, ventanas, drivers, shaders, formatos o memoria.
- Devuelve errores tipados y con contexto útil; reserva los `panic!` para invariantes internas verdaderamente imposibles de recuperar.
- Configura observabilidad para errores no capturados y utiliza error scopes en operaciones donde sea apropiado.
- Inspecciona la información de compilación de shaders cuando haya errores de WGSL.
- No conviertas un error de validación en un `log::warn!` que permita seguir como si nada hubiera ocurrido.

### 3.3 No uses `unsafe` para evitar entender el problema

La API pública de `wgpu` está diseñada para ofrecer una interfaz Rust segura. No bajes a `wgpu-hal`, punteros crudos o `unsafe` para resolver un problema de arquitectura ordinario.

Si una integración realmente requiere `unsafe` o acceso a una API específica del backend:

- Explica la limitación que lo hace necesario.
- Aísla el bloque en un módulo pequeño.
- Documenta sus invariantes de seguridad.
- Añade pruebas y comprobaciones de plataforma.
- Mantén un camino alternativo portable cuando sea posible.

### 3.4 No optimices a ciegas

No afirmes que una implementación es “más rápida” por parecer más sofisticada. Formula una hipótesis, mide el cuello de botella, cambia una variable y vuelve a medir en un build de producción.

## 4. Protocolo de investigación antes de implementar

Para cada tarea, reúne únicamente la información necesaria, pero no saltes estos puntos:

### A. Inspección del proyecto

Comprueba:

- Versión de Rust y MSRV declarado.
- Versión efectiva de `wgpu` y árbol de dependencias relacionado.
- Sistema de ventanas/eventos, por ejemplo `winit`, si corresponde.
- Targets: Windows, Linux, macOS, Android, iOS, navegador/WebAssembly u otros.
- Backends requeridos: Vulkan, DX12, Metal, GLES/WebGL o WebGPU.
- Si se necesita renderizado en ventana, offscreen, compute o una combinación.
- Ciclo de vida actual de la ventana, renderer y recursos GPU.
- Requisitos de latencia, frecuencia de actualización, resolución, consumo y memoria.

Comandos de inspección orientativos (adáptalos al repositorio):

```bash
rustc --version
cargo metadata --format-version 1 --no-deps
cargo tree -i wgpu
cargo check
```

No ejecutes comandos con `--all-features` de manera automática: algunas combinaciones de features pueden ser incompatibles o habilitar backends y dependencias innecesarios.

### B. Verificación de fuentes

Orden de autoridad:

1. Documentación de docs.rs correspondiente a la versión fijada en el proyecto.
2. Guías y módulos de documentación oficiales de esa misma versión.
3. Ejemplos oficiales del tag o release branch correspondiente.
4. Especificación WebGPU/WGSL para reglas del estándar.
5. Issues, discusiones y tutoriales externos como contexto adicional, nunca como sustituto de la API real.

Al usar el repositorio oficial, verifica si estás leyendo `trunk` (desarrollo) o el branch/tag de la versión publicada. No traslades código de `trunk` sin verificarlo contra la dependencia del proyecto.

### C. Decisiones que deben quedar explícitas

Antes de una implementación mediana o grande, resume:

- Qué se construirá y qué queda fuera de alcance.
- Plataformas/backends que deben funcionar.
- Qué recursos son persistentes y cuáles cambian por frame.
- Cómo se manejarán errores, resize, pérdida de superficie y cierre.
- Qué parte será CPU y cuál GPU.
- Qué se medirá para aceptar la mejora.

Si un dato no está disponible, utiliza la opción portable y conservadora, y deja anotada la suposición.

## 5. Arquitectura recomendada

La arquitectura debe adaptarse al tamaño del producto. Para una aplicación de mediana o larga vida, mantén responsabilidades diferenciadas, sin crear capas por ceremonial.

Una estructura de referencia:

```text
src/
  app/
    mod.rs                 # ciclo de vida, eventos y estado de aplicación
  platform/
    mod.rs                 # integración con ventana/sistema operativo
  renderer/
    mod.rs                 # fachada y coordinación del renderer
    context.rs             # Instance, Adapter, Device, Queue y capacidades
    surface.rs             # Surface, configuración, resize y recuperación
    frame.rs               # adquisición, encoding y presentación del frame
    resources/
      mod.rs
      buffers.rs
      textures.rs
      samplers.rs
    pipeline/
      mod.rs
      layouts.rs
      render.rs
      compute.rs
    passes/
      mod.rs
      ...
  gpu_data/
    mod.rs                 # tipos y conversiones de datos CPU → GPU
  shaders/
    ...                    # archivos WGSL
  assets/
  main.rs
```

Es una guía, no una estructura obligatoria. Un proyecto pequeño puede comenzar con menos módulos. Divide cuando existan responsabilidades, reglas de cambio o pruebas distintas.

### Responsabilidades

- **App/platform:** recibe eventos, administra el ciclo de vida de la ventana y decide cuándo actualizar o dibujar. No debe contener detalles de pipelines.
- **GpuContext:** crea y conserva los objetos de contexto, registra capacidades y expone los recursos compartidos necesarios. Evita recrear `Instance`, `Adapter`, `Device` o `Queue` en cada frame.
- **SurfaceManager:** conoce la superficie, su configuración, el formato escogido, el tamaño efectivo y la recuperación ante errores de presentación.
- **Renderer:** coordina recursos y passes; no debería convertirse en un archivo monolítico con toda la lógica de negocio.
- **Resource managers:** administran creación, reutilización, caché y destrucción lógica de recursos GPU.
- **Pipeline system:** construye y conserva pipelines/layouts según una clave estable de configuración. No reconstruyas pipelines en cada frame.
- **Passes:** expresan etapas de renderizado o cómputo con entradas, salidas y responsabilidades definidas.
- **GPU data:** concentra tipos, alineación, serialización y conversión de estructuras Rust a bytes compatibles con WGSL.

### Límites de abstracción

- Mantén la lógica de dominio independiente de `wgpu` cuando sea razonable.
- No expongas detalles internos de `wgpu` a toda la aplicación sin necesidad.
- No envuelvas cada handle en un trait u objeto dinámico por defecto.
- No construyas un render graph genérico hasta que las dependencias entre passes y recursos justifiquen su coste.
- Utiliza `Arc`, canales o mutex solo cuando exista una necesidad real de compartir estado o coordinar tareas. No agregues mutex alrededor de todo el renderer.
- Evita ciclos de ownership entre ventana, aplicación y renderer. Diseña explícitamente quién crea, posee, configura y destruye cada subsistema.

## 6. Inicialización y capacidades del dispositivo

Sigue el flujo conceptual de `wgpu`:

`Instance → Adapter → (Device, Queue) → recursos/pipelines → Surface y/o trabajo GPU`.

Al inicializar:

1. Configura `Instance` según los backends y el entorno objetivo.
2. Solicita un `Adapter` compatible con la superficie cuando haya renderizado en ventana.
3. Elige `PowerPreference` de acuerdo con el producto: rendimiento, batería, portabilidad o latencia. No supongas que una GPU dedicada siempre es la elección correcta.
4. Consulta `AdapterInfo`, features y limits disponibles.
5. Solicita solamente las features imprescindibles y limits compatibles con el mínimo de hardware que se desea soportar.
6. Registra información del adaptador, backend, features y límites elegidos para facilitar diagnósticos.
7. Maneja de forma explícita la ausencia de adaptadores, dispositivos o capacidades requeridas.

No solicites todas las features “por si acaso”. Una feature opcional no debe convertirse accidentalmente en requisito global. Para cada capacidad avanzada define:

- Detección en runtime.
- Camino principal cuando está disponible.
- Fallback o degradación cuando no está disponible.
- Plataformas en las que el fallback fue probado.

Mantén `Device` y `Queue` como recursos de larga duración. Lo habitual es inicializarlos una vez por contexto/dispositivo y reutilizarlos.

## 7. Ciclo de vida de la superficie y del frame

Para renderizado en ventana, trata la superficie como un recurso cuyo tamaño y estado pueden cambiar en cualquier momento.

### Configuración de superficie

- Consulta las capacidades de la superficie y selecciona un formato, modo de presentación y alpha mode soportados.
- No asumas que un formato, `PresentMode` o `TextureFormat` está disponible en todos los backends.
- Usa el tamaño físico en píxeles de la ventana para configurar el render target.
- No configures ni intentes renderizar con dimensiones cero. Suspende el renderizado mientras el tamaño sea cero/minimizado.
- Reconfigura la superficie cuando cambie el tamaño o cuando el estado de la superficie lo requiera.
- Si cambia el formato de la superficie, evalúa si debes reconstruir los pipelines dependientes de ese formato.
- Mantén separada la configuración de superficie de la lógica de negocio.

### Secuencia de un frame

La secuencia general debe ser fácil de localizar y revisar:

1. Procesar eventos y actualizar estado de aplicación.
2. Omitir el frame si la superficie está suspendida/minimizada.
3. Adquirir la textura actual de la superficie.
4. Crear la vista necesaria.
5. Preparar uploads imprescindibles.
6. Crear un `CommandEncoder` y codificar los passes requeridos.
7. Finalizar command buffers.
8. Enviar el trabajo a la `Queue` con una cantidad razonable de submissions.
9. Presentar la textura una vez que el trabajo que la utiliza esté enviado.

No llames a `present` antes de haber enviado el trabajo que debe aparecer en pantalla.

### Errores de adquisición/presentación

Maneja cada variante de error que exista en la versión fijada de `wgpu`. Como política conceptual:

- **Timeout:** normalmente omite el frame actual; no lo conviertas en un bucle de reintentos inmediato.
- **Lost/Outdated:** vuelve a consultar o aplicar la configuración de superficie y reintenta en un frame posterior según la API vigente.
- **OutOfMemory:** informa el fallo de forma crítica y ejecuta una salida controlada; no reintentes indefinidamente.
- **Otros errores:** registra contexto suficiente y aplica una política explícita.

No copies ciegamente un `match` de una versión antigua: verifica el enum y su semántica actuales.

## 8. Recursos GPU: creación, reutilización y memoria

### Política de vida útil

Distingue claramente:

- **Permanentes:** pipelines, layouts, samplers, texturas estáticas y buffers de larga vida.
- **Por escena/material:** recursos asociados a una escena, material o conjunto de datos.
- **Por frame:** datos y recursos temporales necesarios solo para un frame.
- **Readback/staging:** buffers temporales dedicados a transferencias CPU↔GPU.

No crees buffers, texturas, bind groups o pipelines repetidamente dentro del bucle de renderizado cuando puedan reutilizarse. La documentación oficial advierte que crear muchos recursos por frame presiona el allocator y el seguimiento de recursos.

### Buffers

- Define el propósito de cada buffer con `BufferUsages` mínimo y correcto.
- Dimensiona los buffers considerando alineación, límites del adaptador y uso real.
- Evita buffers gigantes sin una estrategia de asignación y seguimiento de rangos.
- Para datos variables, considera subasignación/rangos dentro de buffers grandes cuando simplifique y mejore el uso de recursos.
- Comprueba que `offset + size` está dentro del buffer y que las alineaciones exigidas por la operación se cumplen.
- No confundas el tamaño del tipo Rust con el tamaño/alineación que requiere WGSL o la operación GPU.

### Actualización de datos

- Para cargas pequeñas o moderadas, `Queue::write_buffer` y `Queue::write_texture` suelen ser caminos simples y adecuados.
- Recuerda que las escrituras encoladas se ejecutan en relación con el siguiente `Queue::submit`; no supongas que la transferencia se ejecutó inmediatamente al retornar la llamada.
- `Queue::write_buffer_with` puede evitar una asignación/copia adicional si puedes generar los bytes directamente en la vista devuelta. No lo uses esperando una mejora automática para structs pequeños.
- Para cargas grandes/frecuentes, mide y considera `StagingBelt`, buffers de staging reutilizables o un ring buffer con política de sincronización clara.
- Reutiliza memoria y agrupa cargas cuando sea beneficioso.

No escribas sobre un buffer que todavía está mapeado. Respeta el protocolo de map/unmap y las restricciones de uso de la versión vigente.

### Readback GPU → CPU

- No bloquees el hilo de UI esperando resultados GPU en cada frame.
- Usa un buffer preparado para readback con los usos y alineaciones requeridos.
- Codifica la copia desde el recurso GPU al buffer de lectura.
- Espera la finalización de forma asíncrona o fuera del camino crítico cuando sea posible.
- Lee el rango mapeado, copia/procesa los datos necesarios y desmapea antes de reutilizar el buffer.
- Usa doble/triple buffering o una cola de readbacks si la latencia y el volumen lo requieren.

### Texturas y formatos

- Selecciona `TextureFormat` según uso, precisión, soporte y coste de memoria.
- Verifica features y usos permitidos del formato en el adaptador.
- Considera mipmaps, dimensiones, número de capas, muestras MSAA y formato de profundidad como parte del presupuesto de memoria.
- Para copias buffer↔texture, respeta `COPY_BYTES_PER_ROW_ALIGNMENT` y demás requisitos de layout de la versión. No apliques reglas de alineación de una operación a otra sin comprobar la documentación.
- Evita convertir formatos en cada frame si la conversión puede realizarse una vez o en un pass adecuado.
- Para muchos recursos pequeños considera arrays/atlases cuando realmente reduzcan gestión y cambios de estado.

## 9. Datos Rust ↔ WGSL: alineación y contratos binarios

Los datos que cruzan CPU/GPU deben tener un contrato explícito y testeable.

- Usa estructuras de layout estable, por ejemplo `#[repr(C)]`, junto con herramientas de conversión como `bytemuck` cuando los tipos cumplan realmente sus requisitos.
- No asumas que `#[repr(C)]` por sí solo garantiza compatibilidad con el layout WGSL.
- Añade padding explícito donde el layout lo requiera y evita campos ambiguos como `bool` en layouts binarios compartidos; utiliza representaciones enteras explícitas cuando corresponda.
- Verifica tamaños, offsets y alineación con tests (`size_of`, `align_of`, offsets o aserciones adecuadas).
- Respeta `min_uniform_buffer_offset_alignment`, `min_storage_buffer_offset_alignment` y los requisitos específicos de la binding/operación.
- Documenta unidades, coordenadas, convención de matrices, orden de componentes, rangos y precisión.
- Mantén los tipos de CPU de dominio separados de los tipos GPU cuando sus necesidades de layout sean diferentes.

No conviertas arbitrariamente una referencia Rust en bytes ni uses `transmute` para resolver diferencias de layout.

## 10. Bind groups, layouts y pipelines

### Bind groups

Agrupa recursos según su frecuencia de cambio, empezando por los que cambian menos:

- Grupo de frame/escena o recursos globales estables.
- Grupo de pass.
- Grupo de material/objeto o recursos con cambios más frecuentes.

Esta organización puede reducir cambios de estado y permite reutilizar bindings estables. Es una pauta, no una regla universal: mide y adapta a la carga de trabajo.

- Declara layouts explícitos cuando sea útil para compartirlos y mantener contratos entre shaders/pipelines.
- Evita reconstruir bind groups si los recursos y sus rangos no cambiaron.
- Usa dynamic offsets solo cuando resuelvan una necesidad real y respeta sus alineaciones.
- Mantén sincronizados el layout Rust, el WGSL y el código que crea el bind group.

### Pipelines

- Crea pipelines durante inicialización, carga de escena o una fase controlada, no en el hot path del frame.
- Cachea pipelines con una clave que represente los factores que realmente cambian: shader/entry points, formatos de targets, depth, sample count, layouts y variantes relevantes.
- Evita una explosión combinatoria de variantes sin medir el coste.
- Reutiliza `PipelineLayout`, layouts, samplers y estados compatibles.
- Considera una fase de precalentamiento cuando la compilación o creación cause stutter visible.
- Si hay hot reload, valida el shader nuevo y conserva una política de fallback ante errores.

## 11. WGSL y diseño de shaders

- Mantén shaders en archivos `.wgsl` separados del código Rust, salvo pruebas o casos realmente triviales.
- Organiza shaders por función/pipeline y evita un único archivo enorme con todas las etapas.
- Usa nombres coherentes para bindings, estructuras y entry points.
- Documenta los contratos de datos compartidos con Rust.
- Evita divergencia y trabajo por invocación innecesarios cuando afecten al rendimiento; no optimices sin medir.
- Evita lecturas/escrituras fuera de rango, índices no validados y supuestos sobre tamaños de workgroup.
- Comprueba límites como workgroup size, dispatch size y tamaño máximo de bindings en runtime o durante la construcción de la tarea.
- Para compute, define claramente cómo se particiona el dataset, cómo se manejan elementos sobrantes y cómo se sincronizan las invocaciones dentro de un workgroup.
- No asumas que una barrera WGSL sincroniza workgroups diferentes: las garantías de sincronización deben seguir el modelo de ejecución de WGSL/WebGPU.
- Valida los shaders y recoge información detallada de compilación cuando sea posible.

Si el shader falla, identifica primero si el problema es sintaxis, tipos, layout/bindings, capacidades, pipeline o datos enviados desde CPU. No cambies varias capas simultáneamente.

## 12. Rendimiento: método y prioridades

### Medir antes de tocar

Establece una referencia en `--release` y mide por separado, cuando sea posible:

- Tiempo CPU de actualización y preparación.
- Tiempo CPU de creación/encoding de comandos.
- Tiempo GPU de cada pass.
- Tiempo de uploads y readbacks.
- Cantidad de submissions por frame.
- Cantidad de recursos creados/destruidos y memoria aproximada.
- Latencia de presentación, stutter y estabilidad del frame time.

Registra hardware, backend, resolución, driver, versión de `wgpu`, configuración y dataset para que la comparación sea reproducible.

### Prioridades recomendadas

1. Eliminar trabajo y asignaciones innecesarios en el hot path.
2. Evitar recrear recursos GPU por frame.
3. Reducir uploads y readbacks innecesarios.
4. Reutilizar layouts, pipelines, bind groups y samplers.
5. Agrupar trabajo y minimizar cambios de estado.
6. Reducir overdraw, ancho de banda, resolución o precisión solo cuando el perfil indique que son relevantes.
7. Ajustar workgroup sizes y algoritmos compute mediante benchmarks representativos.

La guía oficial sugiere limitar `Queue::submit` a unas pocas llamadas por frame (da 1–5 como ejemplo orientativo). No lo conviertas en un límite rígido: consolida command buffers cuando sea razonable y mide el efecto en el proyecto real.

### CPU y GPU

No atribuyas automáticamente un frame lento a la GPU. Distingue:

- CPU-bound: lógica, generación de datos, asignaciones, encoding, sincronización o driver.
- GPU-bound: shaders, bandwidth, overdraw, render targets, compute o dependencias.
- Transfer-bound: uploads/readbacks y esperas.
- Present-bound: sincronización con compositor/monitor y modo de presentación.

No uses `device.poll`, esperas de submission ni lecturas bloqueantes como solución rutinaria para coordinar cada frame. Úsalas solo donde el flujo y la plataforma lo requieran.

## 13. Portabilidad y compatibilidad

`wgpu` abstrae múltiples backends, pero eso no significa que todos los dispositivos soporten los mismos formatos, límites o features.

- Diseña primero sobre el subconjunto portable que realmente necesite el producto.
- Comprueba capacidades en runtime.
- Separa el camino base de las mejoras opcionales.
- Prueba en más de un backend cuando el alcance del producto lo requiera.
- Evita depender de extensiones experimentales para funciones esenciales sin una estrategia de fallback y una decisión explícita de producto.
- Revisa el soporte de shaders y features en cada target.
- Mantén configuraciones de Cargo dirigidas por target cuando ayuden a no incluir dependencias/backends innecesarios.
- No supongas que los objetos GPU se pueden compartir entre threads en todos los targets; respeta las restricciones específicas de WebAssembly y del backend.

Para WebAssembly, comprueba el target y entorno de navegador, el backend elegido (WebGPU/WebGL cuando corresponda), las diferencias de asincronía y las limitaciones de hilos. No traslades supuestos de Windows/Linux nativo al navegador.

## 14. Concurrencia y modelo de ejecución

- Mantén un único propietario lógico del ciclo de vida de la superficie y la presentación.
- Separa el trabajo de CPU que prepara datos de la codificación y envío GPU cuando eso reduzca acoplamiento.
- Usa canales para transferir comandos/datos entre subsistemas si evita compartir estado mutable global.
- No crees un thread por recurso, pass o shader.
- No asumas que mover handles GPU entre threads es válido en todos los targets: comprueba los trait bounds y la documentación de la versión actual.
- Evita locks durante el render loop cuando sea posible; usa snapshots/doble buffering para estado de frame si la carga lo justifica.
- No bloquees el hilo de eventos/UI con operaciones de lectura GPU o carga de assets pesada.

## 15. Errores, diagnósticos y observabilidad

Implementa una estrategia coherente, no logs dispersos.

- Instala un handler para errores no capturados cuando la API de la versión lo permita.
- Utiliza error scopes para aislar operaciones propensas a errores de validación o creación.
- Recoge `ShaderModule` compilation info cuando los shaders no compilen.
- Incluye en los logs el nombre del recurso/pass, backend, adaptador y fase del frame.
- Distingue errores recuperables (por ejemplo, recrear superficie) de errores fatales (por ejemplo, falta de memoria).
- Añade IDs/nombres estables a recursos importantes si la versión de `wgpu` ofrece mecanismos adecuados de etiquetado.
- En desarrollo, habilita validación y diagnósticos; en producción, decide explícitamente el equilibrio entre observabilidad y coste.

Herramientas según plataforma:

- RenderDoc para capturas compatibles en Windows/Linux y backends soportados.
- PIX para D3D12 en Windows.
- Xcode GPU Capture para Metal.
- Capas de validación de Vulkan cuando corresponda.
- `RUST_BACKTRACE=1` y logs estructurados para fallos CPU/Rust.

No des por resuelto un fallo porque desaparezca al desactivar la validación. La validación debe ayudar a encontrar la causa, no ocultarla.

## 16. Pruebas y quality gates

Diseña pruebas por nivel:

### Unitarias (CPU)

Prueba sin GPU cuando sea posible:

- Layouts y serialización de datos GPU.
- Cálculos de offsets, strides, tamaños y alineaciones.
- Generación de índices, vértices, dispatch dimensions y rangos.
- Selección de variantes y decisiones de capabilities.
- Lógica de negocio separada del renderer.

### Integración GPU

Cuando el entorno CI lo permita:

- Inicialización del adaptador/dispositivo.
- Creación de shaders, layouts y pipelines.
- Un frame/render o dispatch de prueba.
- Readback de resultados y comparación con tolerancias apropiadas.
- Resize, pérdida/reconfiguración de superficie y rutas de error relevantes.

No dependas exclusivamente de un backend `noop` para demostrar que el shader calcula correctamente: un backend sin cómputo no valida el resultado real de GPU.

### Matriz de plataforma

La matriz debe reflejar los targets reales del producto, no una lista aspiracional. Como mínimo, registra qué combinaciones de sistema operativo, backend, GPU/driver y target fueron probadas.

### Gates antes de entregar

Ejecuta los comandos compatibles con el proyecto:

```bash
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

Añade comprobaciones por target cuando el proyecto los soporte, por ejemplo el target WASM instalado. No ocultes fallos con `allow` globales, desactivando validaciones o quitando tests.

Para cambios de rendimiento, incluye benchmark antes/después en `--release`, con configuración reproducible.

## 17. Mantenimiento y evolución a largo plazo

- Fija dependencias y conserva `Cargo.lock` en aplicaciones/binarios.
- Actualiza `wgpu` deliberadamente, revisando changelog, notas de migración, ejemplos del release y cambios de MSRV.
- Agrupa una actualización mayor con los cambios necesarios para que sea revisable y reversible.
- Evita depender de APIs experimentales o internas sin una razón de producto documentada.
- Mantén WGSL, layouts y tipos Rust revisables como una misma unidad contractual.
- Documenta decisiones no obvias en ADRs breves: por qué se eligió un backend, formato, estrategia de upload, política de fallback o diseño de recursos.
- Define criterios de compatibilidad: hardware mínimo, features obligatorias, formatos y límites mínimos.
- Mide regresiones con benchmarks/versiones de referencia.
- Elimina abstracciones, flags y rutas de compatibilidad que ya no tengan consumidores, pero solo mediante cambios controlados.

## 18. Anti-patrones que debes rechazar

No entregues una solución que:

- Inicialice `Instance`, `Adapter`, `Device` o `Queue` cada frame.
- Cree pipelines, buffers, texturas o bind groups en cada frame sin justificación.
- Llame a `Queue::submit` una vez por cada pequeña operación sin una razón medida.
- Solicite todas las features o límites máximos sin evaluar hardware.
- Asuma que un formato o modo de presentación existe en todas las GPUs.
- Ignore `SurfaceError` o mantenga un bucle de reintentos infinito.
- Use `unwrap()` en rutas normales de inicialización/renderizado.
- Bloquee la UI esperando un readback GPU en cada frame.
- Suponga compatibilidad automática entre `repr(C)` y WGSL.
- Use `unsafe`, `transmute`, fugas deliberadas o `static` globales para eludir un problema de ownership sin justificarlo.
- Mezcle lógica de dominio, eventos, recursos, shaders y todos los passes en un único módulo monolítico.
- Copie código de tutoriales antiguos sin verificar la versión.
- Declare una optimización sin benchmark.
- Desactive validación para silenciar errores.

## 19. Flujo de trabajo obligatorio del agente

Para cada implementación:

1. **Inspeccionar:** lee la estructura del proyecto, Cargo.toml/lockfile y código relacionado.
2. **Verificar:** confirma versión, API, features, MSRV, targets y ejemplos compatibles.
3. **Diseñar:** propone el cambio mínimo que conserva límites claros y ownership explícito.
4. **Implementar:** modifica por capas; evita reescrituras fuera del alcance.
5. **Validar:** ejecuta format, check, tests, clippy y pruebas de target/GPU disponibles.
6. **Medir:** cuando se persiga rendimiento, compara una referencia con el cambio en release.
7. **Revisar:** busca recursos creados por frame, bloqueos, `unwrap`, supuestos de capacidades, errores ignorados y complejidad innecesaria.
8. **Informar:** resume qué cambió, por qué, compatibilidad verificada, comandos ejecutados, resultados y limitaciones que quedaron pendientes.

Si una verificación no puede ejecutarse (por falta de GPU, driver, target, SDK o entorno gráfico), indícalo exactamente. No afirmes que una ruta fue probada si solamente compiló.

## 20. Formato de entrega esperado

Al completar una tarea mediana o grande, responde con:

- **Objetivo:** qué comportamiento se implementó.
- **Diseño:** decisiones arquitectónicas relevantes y por qué.
- **Compatibilidad:** versión de `wgpu`, Rust/MSRV y targets comprobados.
- **Validación:** comandos ejecutados y resultados reales.
- **Rendimiento:** mediciones antes/después, o “no medido” si no era el objetivo.
- **Limitaciones:** aspectos no verificados y riesgos conocidos.

Sé preciso. Distingue siempre entre “compila”, “pasa pruebas”, “se ejecutó en GPU real” y “se midió en hardware representativo”.

## 21. Referencias oficiales de consulta

Consulta la documentación de la versión exacta que utiliza el proyecto. Los enlaces siguientes son puntos de entrada, no autorización para copiar API de otra versión:

- [Documentación de wgpu en docs.rs](https://docs.rs/wgpu/latest/wgpu/)
- [Guías generales oficiales](https://wgpu.rs/doc/wgpu/documentation/)
- [Best Practices oficiales](https://wgpu.rs/doc/wgpu/documentation/best_practices/)
- [Dos and Don’ts oficiales](https://wgpu.rs/doc/wgpu/documentation/best_practices/dos_and_donts/)
- [Conceptos fundamentales: Device, Queue y ciclo de render](https://wgpu.rs/doc/wgpu/documentation/getting_started/learning_wgpu/)
- [Depuración de aplicaciones wgpu](https://wgpu.rs/doc/wgpu/documentation/debugging/debugging_applications/)
- [Depuración de rendimiento](https://wgpu.rs/doc/wgpu/documentation/debugging/performance/)
- [Arquitectura interna de wgpu](https://wgpu.rs/doc/wgpu/documentation/internals/architecture/)
- [Repositorio oficial y ejemplos](https://github.com/gfx-rs/wgpu)
- [Especificación WebGPU](https://www.w3.org/TR/webgpu/)
- [Especificación WGSL](https://www.w3.org/TR/WGSL/)

**Nota de versión de referencia al crear esta skill (18-11-2026):** la URL `docs.rs/wgpu/latest/wgpu/` resolvía a `wgpu 30.0.2` y la documentación indicaba MSRV `1.87`. La guía hospedada en `wgpu.rs` mostraba páginas de la serie `30.0.0`. Estos datos son una fotografía temporal, no valores permanentes: vuelve a verificarlos al usar la skill.
