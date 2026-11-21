---
name: bytemuck-derive-layout-contracts
description: Derives para declarar contratos de representación binaria seguros y verificables.
---

# bytemuck-derive-layout-contracts

## Misión

Actúa como especialista en representación de datos y seguridad de memoria Rust. Usa `bytemuck_derive` para generar implementaciones de traits de `bytemuck` solo cuando el tipo realmente satisface sus invariantes. Los derives no son una forma de silenciar al compilador: formalizan una promesa sobre los bytes del tipo.

## Puerta de compatibilidad

- Comprueba versiones de `bytemuck` y `bytemuck_derive` en `Cargo.lock` y su compatibilidad. La versión observada de `bytemuck_derive` es 1.12.1; `bytemuck` puede depender de una versión semver compatible mediante la feature `derive`.
- En muchos proyectos basta con `bytemuck = { version = "...", features = ["derive"] }`; no agregues `bytemuck_derive` como dependencia directa si no hay motivo.
- Verifica los derives y atributos admitidos en la versión resuelta. No uses atributos de tutoriales antiguos sin confirmación.

## Derives y semántica

La documentación actual incluye derives como `Pod`, `Zeroable`, `AnyBitPattern`, `CheckedBitPattern`, `NoUninit`, `Contiguous`, `ByteEq`, `ByteHash` y `TransparentWrapper`. No son intercambiables:

- `Zeroable`: el patrón de bits cero representa un valor válido.
- `NoUninit`: la representación del valor no contiene bytes de padding/no inicializados que puedan filtrarse al observar bytes.
- `AnyBitPattern`: todo patrón de bits es un valor válido del tipo (además de sus bounds).
- `Pod`: contrato más fuerte para datos plain-old-data: debe ser `Copy`, tener representación compatible y no contener padding ni tipos con patrones inválidos.
- `CheckedBitPattern`: permite validación explícita de ciertos patrones al interpretar bytes.
- `Contiguous`: enum con discriminantes contiguos según sus reglas documentadas.
- `TransparentWrapper`: wrapper transparente de otro tipo con garantías específicas.
- `ByteEq`/`ByteHash`: comparaciones/hash por bytes; solo tienen sentido si la semántica del tipo coincide con esa representación.

## Reglas para `Pod`

- Usa `#[repr(C)]` o `#[repr(transparent)]` cuando lo exijan las reglas del derive.
- Cada campo debe satisfacer `Pod`.
- El tipo no puede contener bytes de padding. El derive debe comprobarlo; no intentes forzar el resultado.
- Los tipos genéricos tienen restricciones adicionales; verifica las reglas actuales y bounds necesarios.
- No uses `#[repr(packed)]` como parche automático: puede crear referencias desalineadas y complicar el acceso seguro.
- No declares `Pod` manualmente con `unsafe impl` para superar un error del derive sin una auditoría formal de tamaño, alineación, padding y validez de bit patterns.

Ejemplo típico cuando la estructura satisface las condiciones:

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
```

No copies este layout para cualquier estructura: comprueba offsets, size, alignment y las reglas del consumidor externo.

## Diseño de tipos de transferencia

- Separa tipos de dominio ricos (`String`, `Vec`, enums con payload, punteros, `bool` si no cumple el contrato) de tipos de transferencia binaria.
- Crea estructuras dedicadas para GPU, FFI o buffers; no fuerces todo el modelo de negocio a ser `Pod`.
- Documenta el contrato de bytes: orden de campos, escalas/unidades, endianess cuando sea formato persistente, alineación requerida y versión del protocolo.
- Para enums, elige una representación explícita y un derive adecuado; no asumas que un enum Rust ordinario admite cualquier discriminante.
- Usa `CheckedBitPattern` para formatos externos cuando los bytes puedan contener discriminantes/valores inválidos y la API de la versión lo permita.

## Integración con GPU

- `Pod` facilita convertir una estructura a bytes, pero no demuestra que su layout coincida con WGSL, HLSL, SPIR-V o la alineación de un uniform/storage buffer.
- Comprueba `size_of`, `align_of`, offsets y reglas de alineación del lenguaje shader. Añade padding explícito cuando el estándar GPU lo requiera.
- Mantén la estructura Rust y el layout WGSL sincronizados con tests o generación de código; no confíes solo en que ambos compilen.
- Diferencia vertex buffers, uniforms y storage buffers: sus restricciones pueden ser distintas.

## Validación

- Añade tests de `size_of`, `align_of` y offsets cuando haya un ABI/layout externo.
- Verifica bytes serializados de ejemplos conocidos y round-trip donde corresponda.
- Ejecuta tests de compile-fail si el contrato requiere que ciertos layouts no compilen.
- Inspecciona el código generado solo para depurar, no dependas de detalles internos del proc macro.

## Anti-patrones

- Derivar `Pod` por conveniencia sin leer las reglas de seguridad.
- Añadir `repr(C)` y asumir que desaparece padding.
- Convertir una estructura de dominio a `Pod` solo para subirla a GPU.
- Usar `ByteEq` para tipos cuya semántica ignora padding, NaN o normalización de valores.
- Tratar el derive como sustituto de endianess, validación de protocolo o layout WGSL.

## Flujo del agente

1. Identifica consumidor de bytes: GPU, FFI, archivo o red.
2. Especifica ABI/layout y patrones válidos.
3. Diseña un tipo de transferencia separado.
4. Selecciona el trait mínimo que cubre la operación requerida.
5. Aplica derive y deja que el compilador rechace contratos inválidos.
6. Añade comprobaciones de tamaño/alineación/offsets y casos adversos.

## Criterios de aceptación

- Cada derive tiene justificación semántica.
- No hay padding no controlado en los tipos que lo prohíben.
- No se usa `unsafe impl` sin una revisión de seguridad documentada.
- El layout se valida contra el consumidor real, especialmente WGSL.
