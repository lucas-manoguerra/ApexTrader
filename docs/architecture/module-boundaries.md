# Límites de módulos

Cada módulo tiene una responsabilidad principal y claramente identificable. Un módulo no debe asumir responsabilidades que pertenecen a otro.

## Módulos y responsabilidades

| Módulo | Responsabilidad |
|--------|-----------------|
| `market_domain` | Conceptos y reglas del mercado |
| `market_data` | Recepción y procesamiento de datos de mercado |
| `chart_domain` | Modelos y reglas de gráficos |
| `chart_engine` | Comportamiento e interacciones del chart |
| `chart_renderer` | Preparación de elementos gráficos |
| `trading_domain` | Órdenes, posiciones y operaciones |
| `storage` | Persistencia y recuperación de datos |

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
- El módulo de storage solo lee/escribe; no interpreta reglas de trading.

## Cuando un módulo crece demasiado

Si un módulo empieza a asumir responsabilidades ajenas, se divide en dos módulos con límites claros, nunca se deja como un "módulo todo-en-uno".