# Arquitectura — Visión general

ApexTrader sigue una separación inspirada en arquitectura hexagonal (puertos y adaptadores). Las dependencias de diseño apuntan hacia el dominio, no al revés.

## Capas

| Capa | Qué contiene | Depende de |
|------|-------------|------------|
| **Dominio** | Reglas de negocio, entidades, tipos, invariantes | Nada (ninguna capa interna) |
| **Aplicación** | Casos de uso, coordinación, orquestación | Dominio |
| **Infraestructura** | GPU, broker APIs, WebSockets, archivos, base de datos | Dominio y aplicación |

## Regla clave

El dominio de trading no necesita conocer qué broker estamos utilizando. Podríamos cambiar de broker sin reescribir las reglas que determinan si una orden es válida.

## Módulos actuales

- `src/domain` — conceptos y reglas del mercado (velas, viewport).
- `src/renderer` — preparación y dibujo de elementos gráficos.
- `src/errors` — errores tipados por subsistema.
- `src/logging` — logging estructurado.

## Diagrama de dependencias (simplificado)

```
Aplicación (main.rs)
    ├─► Dominio (domain/)
    ├─► Renderer (renderer/)
    ├─► Errores (errors/)
    └─► Logging (logging/)
```

El dominio no importa de `renderer`, ni de `egui`, ni de `wgpu`.