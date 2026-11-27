# Arquitectura

## Propósito

ApexTrader se organiza alrededor de responsabilidades independientes. La arquitectura debe permitir reemplazar infraestructura, incorporar fuentes de mercado y ampliar capacidades sin propagar detalles técnicos por todo el sistema.

## Capas y responsabilidades

### Dominio

Define conceptos, invariantes y reglas de negocio. No depende de frameworks de UI, APIs de sistema operativo, proveedores externos ni backends de renderizado.

### Aplicación

Orquesta casos de uso y coordina puertos del dominio con adaptadores de infraestructura. No debe absorber reglas de negocio que pertenecen al dominio.

### Mercado

Aísla la adquisición, normalización y validación de datos externos. Los formatos de proveedores no deben filtrarse a través de los límites de aplicación y dominio.

### Gráficos

Contiene modelos y lógica de visualización independientes del backend: transformaciones, rangos visibles, layout e interacción.

### Renderizador

Convierte una representación visual estable en operaciones específicas del backend. El backend no define reglas de negocio ni modifica el estado del dominio.

### Infraestructura y presentación

La UI, ventanas, GPU, logging y proveedores concretos son detalles reemplazables. Deben conectarse mediante interfaces explícitas y permanecer fuera del dominio.

La matriz de ownership y las pruebas de responsabilidad por componente están definidas en [component-ownership.md](component-ownership.md). Ese documento es la referencia para asignar una única capacidad cohesiva a cada componente y delimitar lo que no debe hacer.

## Reglas de dependencia

1. El dominio no depende de capas externas.
2. Las capas externas dependen de contratos estables definidos hacia el interior.
3. Los datos externos se validan en el límite de entrada.
4. La representación intermedia del chart desacopla la escena del backend de dibujo.
5. No se permiten ciclos de dependencias.
6. No se extraen crates por razones cosméticas; cada crate debe justificar una frontera real.

## Estado conocido

La base actual contiene módulos de aplicación, dominio, mercado, chart y renderer. El historial del proyecto identifica glow como backend funcional y wgpu como trabajo experimental/incompleto. Esta condición debe verificarse contra la implementación antes de anunciar cambios de estado.

## Evolución

Toda modificación de límites debe incluir análisis de dependencias, impacto en tests, costo de compilación y consecuencias sobre APIs públicas. Las decisiones significativas se registran como ADR.
