# Límites de módulos

Cada módulo tiene una responsabilidad principal y claramente identificable. Un módulo no debe asumir responsabilidades que pertenecen a otro.

## Módulos y responsabilidades

| Módulo | Responsabilidad |
|--------|-----------------|
| `domain` | Conceptos y reglas del mercado (velas, viewport, tipos de precio, datos de chart) |
| `market` | Recepción y procesamiento de datos de mercado (proveedores, símbolos, velas de mercado) |
| `chart` | Modelos y reglas de gráficos (layout, crosshair, interacción, hit-testing, escalas, transformaciones, rango visible) |
| `renderer` | Preparación y dibujo de elementos gráficos (glow, wgpu, trait de renderizado) |
| `application` | Casos de uso, coordinación y orquestación (carga de velas, eventos de input, estado de la app) |
| `errors` | Errores tipados por subsistema (thiserror) |
| `logging` | Logging estructurado (tracing) |

## Lo que NO queremos

Una función del gráfico que:

- Descarga precios desde un broker.
- Calcula indicadores.
- Modifica posiciones.
- Actualiza la interfaz.
- Guarda datos en disco.

Eso genera acoplamiento: modificar una funcionalidad puede romper otras cinco.

## Cómo respetar los límites

- Una función de renderer solo dibuja; no calcula indicadores ni guarda datos.
- Una función de dominio solo aplica reglas de negocio; no toca la UI ni el disco.
- El módulo de market solo recibe y valida datos de mercado; no interpreta reglas de trading ni persiste.
- El módulo de application coordina entre dominio, market y renderer; no contiene reglas de negocio puras.

## Cuando un módulo crece demasiado

Si un módulo empieza a asumir responsabilidades ajenas, se divide en dos módulos con límites claros, nunca se deja como un "módulo todo-en-uno".

## Regla de dependencias

Las dependencias apuntan hacia el dominio:

```
Infraestructura → Aplicación → Dominio
```

Nunca al revés. El dominio no importa de infraestructura ni de aplicación. El dominio de chart (`src/domain/chart`) no importa de `egui`, `wgpu` ni `winit`.
