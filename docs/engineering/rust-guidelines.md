# Directrices de Rust

## Características del lenguaje y qué nos protegen

| Herramienta | Qué nos protege |
|-------------|-----------------|
| Tipos específicos | Evitan mezclar conceptos incompatibles |
| Ownership | Controla quién posee los datos |
| Borrowing | Evita copias y accesos incorrectos |
| Encapsulación | Protege invariantes |
| `Result` | Obliga a contemplar errores |
| `Option` | Hace explícita la ausencia de valores |
| Traits | Definen contratos y comportamientos |
| Lifetimes | Expresan relaciones entre referencias |
| Modules | Controlan visibilidad y organización |

## Convenciones obligatorias

- No uses `unwrap()` ni `expect()` en producción normal.
- No uses `panic!()` para errores recuperables.
- No uses `unsafe` sin justificación documentada.
- Tipos que impiden estados inválidos (por ejemplo, `Price` en lugar de `f64` crudo).
- Documentá APIs públicas y sus invariantes.

## Nota sobre `unsafe`

No prohibimos `unsafe`. Puede ser necesario para interoperabilidad o ciertas optimizaciones. Pero tiene que estar encapsulado, justificado y probado.

## Ejemplo: tipo que previene estados inválidos

```rust
/// Precio de mercado: nunca negativo, nunca NaN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price(f64);

impl Price {
    pub fn new(value: f64) -> Result<Self, Error> {
        if value.is_nan() || value < 0.0 {
            return Err(Error::InvalidPrice);
        }
        Ok(Self(value))
    }

    pub fn value(self) -> f64 {
        self.0
    }
}
```

Así es imposible crear un `Price` inválido desde fuera del módulo.
