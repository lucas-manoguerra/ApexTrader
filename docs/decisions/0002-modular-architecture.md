# ADR-0002: Evolución modular con dependencias dirigidas hacia el dominio

## Estado

Aceptada

## Contexto

ApexTrader integra dominio, aplicación, datos de mercado, gráficos, presentación y renderizado. La evolución independiente de estas áreas requiere límites explícitos y dependencias controladas.

## Alternativas consideradas

- Mantener todos los componentes acoplados en una capa de aplicación.
- Dividir inmediatamente cada módulo en crates independientes.
- Mantener módulos cohesivos dentro del crate actual y extraer crates solo cuando exista una frontera técnica y operativa real.

## Decisión

Mantener separación por responsabilidades y dirigir las dependencias hacia el dominio. La UI, los proveedores externos, las APIs de sistema operativo y los backends de renderizado son detalles de infraestructura. No se extraerán crates por motivos cosméticos; cada extracción debe justificar su contrato, ciclo de compilación, propiedad y beneficio de mantenimiento.

## Consecuencias

- El dominio permanece independiente de infraestructura.
- Los casos de uso coordinan contratos y adaptadores.
- Los datos externos se validan en los límites de entrada.
- La representación intermedia desacopla la escena de los backends.
- Las nuevas fronteras requieren pruebas y una decisión documentada.

## Condiciones de revisión

Revisar cuando aparezcan necesidades concretas de compilación independiente, reutilización, aislamiento de seguridad, propiedad de equipos o despliegue separado.
