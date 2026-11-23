# Reglas de dependencias

## Dirección de las dependencias

Las dependencias de diseño apuntan hacia el dominio:

```
Infraestructura → Aplicación → Dominio
```

Nunca al revés. El dominio no importa de infraestructura ni de aplicación.

## Ejemplos permitidos y no permitidos

| Desde | Hacia | ¿Permitido? | Razón |
|-------|-------|-------------|-------|
| `chart` | `domain` | ✅ Sí | Chart depende del dominio de charts |
| `domain` | `chart` | ❌ No | El dominio no conoce el motor de chart |
| `renderer` | `domain` | ✅ Sí | Renderer lee datos de dominio para dibujar |
| `domain` | `renderer` | ❌ No | El dominio no conoce el renderer |
| `application` | `domain` | ✅ Sí | Aplicación usa el dominio |
| `domain` | `application` | ❌ No | El dominio no conoce la aplicación |
| `market` | `domain` | ✅ Sí | Market valida contra tipos de dominio |
| `domain` | `market` | ❌ No | El dominio no conoce proveedores de mercado |
| `renderer` | `egui` / `wgpu` | ✅ Sí | Renderer es infraestructura gráfica |
| `domain` | `egui` / `wgpu` / `winit` | ❌ No | El dominio no conoce UI ni GPU |
| `chart` | `egui` / `wgpu` / `winit` | ❌ No | Chart engine es puro de dominio |

## Cómo detectar acoplamiento indebido

Si un módulo del dominio importa de `egui`, `wgpu`, `winit` o cualquier crate de infraestructura, hay un acoplamiento indebido que debe corregirse.

Si un módulo de infraestructura importa de otro módulo de infraestructura de forma directa (sin pasar por dominio o aplicación), revisa si esa dependencia es realmente necesaria o si puede abstraerse mediante un trait en el dominio.

## Verificación automática

El script `scripts/verify_architecture.ps1` verifica que `src/domain/` no importe de crates de infraestructura.
