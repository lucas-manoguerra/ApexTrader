---
name: bytemuck-memory-casting
description: Conversión segura entre valores Rust y bytes con garantías explícitas de validez y alineación.
---

# bytemuck-memory-casting

## Misión

Actúa como ingeniero senior de Rust enfocado en memoria, layout y conversión de bytes. Utiliza `bytemuck` para casts de memoria cuando las garantías de tipo, alineación, tamaño y validez estén demostradas. No lo trates como un serializador universal.

## Puerta de compatibilidad

- Verifica la versión efectiva, features y tipos disponibles en el `Cargo.lock`. La versión observada en docs.rs es 1.25.2.
- Revisa las features del crate, especialmente `derive`, y habilita solo lo necesario.
- Consulta documentación de cada función/trait; los requisitos de `cast`, `try_cast`, `bytes_of`, `cast_slice` y APIs equivalentes no son idénticos.

## Modelo de garantías

La biblioteca usa traits de seguridad como `NoUninit`, `AnyBitPattern`, `Zeroable` y `Pod` para expresar diferentes garantías. Conceptualmente:

- `NoUninit`: los bytes observables están inicializados; evita exponer padding no inicializado.
- `AnyBitPattern`: cualquier patrón de bits es válido para el tipo destino, sujeto a sus bounds.
- `Zeroable`: el patrón de bits cero representa un valor válido.
- `Pod`: contrato más fuerte que combina garantías útiles para tratar el tipo como datos plain-old-data.

No asumas una jerarquía intuitiva perfecta entre traits: respeta los bounds exactos publicados por la versión.

## Selección de API

- Usa `cast`/`cast_ref`/`cast_mut`/`cast_slice` cuando el tamaño, la alineación y la validez estén garantizados por construcción y la firma sea apropiada.
- Prefiere variantes `try_*` o funciones que devuelven `Result` para entradas de tamaño dinámico, límites externos o datos que pueden no cumplir las precondiciones.
- Para exponer bytes de un valor, usa la API documentada adecuada (por ejemplo, `bytes_of`) y exige el trait correcto.
- Nunca uses casts de punteros manuales como atajo cuando una función segura de `bytemuck` expresa la operación.
- Comprueba alineación del destino; una secuencia de bytes arbitraria no necesariamente está alineada para cualquier tipo.

## Seguridad y límites

1. Los bytes externos no son automáticamente un valor Rust válido. Valida discriminantes, rangos, NaN/infinito, longitudes y versiones antes de usarlos.
2. Un cast no hace endian conversion. Si el formato define little-endian/big-endian, transforma explícitamente cada campo.
3. Un cast no proporciona estabilidad de formato a través de versiones/compiladores/arquitecturas. No guardes structs nativos como archivos de larga duración sin un protocolo explícito.
4. Un cast no resuelve padding ni alineación de GPU/FFI. Comprueba el layout del consumidor.
5. No expongas memoria sin inicializar ni bytes de padding a logs, red, archivos o APIs externas.
6. Evita reinterpretar tipos con invariantes internos (punteros, referencias, `bool`, enums sin validación, `NonZero*`) desde bytes arbitrarios.

## GPU y FFI

- Para buffers GPU, combina `bytemuck` con tipos de transferencia dedicados y validación de layout. WGSL tiene reglas de alineación y tamaño que no son idénticas a `repr(C)`.
- Para vertex data, verifica stride, offsets, formato de atributo y alignment del buffer.
- Para uniform/storage buffers, comprueba reglas de layout de WGSL (incluidos padding y alineación de vectores/matrices) y contrástalas con Rust.
- Para FFI, declara ABI (`repr(C)`), tamaño, alineación y propiedad/lifetime por separado; `bytemuck` no resuelve ownership ni compatibilidad completa del ABI.

## Ejemplo conceptual

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Color {
    rgba: [u8; 4],
}

let color = Color { rgba: [255, 128, 64, 255] };
let bytes: &[u8] = bytemuck::bytes_of(&color);
```

El tipo debe cumplir realmente las condiciones de `Pod`; este ejemplo es un caso simple, no una plantilla para estructuras arbitrarias.

## Manejo de errores

- Distingue errores de tamaño, alineación y bit pattern; devuelve errores tipados o los propaga en lugar de asumir éxito.
- En datos corruptos o no confiables, valida antes de elevar bytes a tipos con invariantes.
- No uses `unwrap` en cargas de archivos, paquetes o buffers proporcionados por usuarios.

## Anti-patrones

- Usar `transmute` manual para evitar traits.
- Considerar `Pod` equivalente a “se puede serializar para siempre”.
- Casts directos sobre bytes de red sin validar.
- Olvidar endianess o layout GPU.
- Usar `repr(packed)` para forzar la ausencia de padding sin revisar accesos no alineados.
- Convertir grandes buffers repetidamente cuando puede mantenerse una representación byte-oriented.

## Flujo del agente

1. Identifica el origen y destino de los bytes.
2. Especifica tamaño, alineación, endianess y patrones válidos.
3. Selecciona el trait mínimo que soporta la operación.
4. Elige cast infalible solo si las precondiciones son estáticas; en otro caso valida.
5. Añade pruebas de layout y bytes conocidos.
6. Prueba entradas truncadas, desalineadas, fuera de rango y malformadas.

## Criterios de aceptación

- No se confunde reinterpretación de memoria con serialización.
- Todas las garantías están expresadas mediante traits/API apropiados.
- No se aceptan patrones de bits inválidos.
- Tamaño, alineación y endianess están verificados.
