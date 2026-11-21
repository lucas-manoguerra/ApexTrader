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
| `chart_engine` | `chart_domain` | ✅ Sí | Motor depende del dominio de charts |
| `chart_domain` | `chart_engine` | ❌ No | El dominio no conoce el motor |
| `trading_domain` | `egui` | ❌ No | El dominio de trading no conoce la UI |
| `renderer` | `chart_domain` | ✅ Sí | Renderer lee datos de dominio para dibujar |
| `storage` | `trading_domain` | ✅ Sí | Storage persiste tipos de dominio |
| `trading_domain` | `storage` | ❌ No | El dominio no sabe cómo se persiste |

## Cómo detectar acoplamiento indebido

Si un módulo del dominio importa de `egui`, `wgpu`, `winit` o cualquier crate de infraestructura, hay un acoplamiento indebido que debe corregirse.

Si un módulo de infraestructura importa de otro módulo de infraestructura de forma directa (sin pasar por dominio o aplicación), revisa si esa dependencia es realmente necesaria o si puede abstraerse mediante un trait en el dominio.