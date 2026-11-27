# Evaluación del modelo numérico financiero

## Estado

Documento de evaluación técnica. No selecciona todavía una representación numérica definitiva porque el repositorio no identifica mercados, venues, clases de instrumentos ni proveedores reales soportados. La decisión debe basarse en contratos de mercado verificables y pruebas de rango/escala, no en preferencias generales.

## Evidencia actual

- `Price` y `Quantity` encapsulan `rust_decimal::Decimal`.
- `MarketCandle` representa OHLCV mediante `f64`.
- `domain::Candle` también almacena OHLCV mediante `f64`.
- `MarketCandle::to_domain_candle` y `to_chart_data_point` transfieren esos valores flotantes al dominio gráfico sin recuperar la representación decimal original.
- El test end-to-end convierte extremos `f64` a `Decimal` mediante `Decimal::from_f64`; esa operación convierte el valor binario ya representado y no demuestra preservación del decimal recibido.
- El contrato `MarketDataProvider` solo devuelve `Vec<MarketCandle>`; no declara identidad de serie, timeframe, venue, proveedor estructurado ni metadatos de instrumento.
- La implementación inspeccionada de proveedor es mock. No se encontró en los módulos revisados un parser de payload real que permita verificar cómo se preservaría el token decimal de un proveedor externo.

## Riesgo técnico

La coexistencia de `Decimal` en algunos value objects y `f64` en las velas produce dos dominios numéricos sin una frontera explícita. La conversión posterior desde `f64` no reconstruye de forma fiable el texto decimal original ni la escala declarada por el instrumento. Esto impide afirmar que los valores OHLCV son exactos respecto de la fuente.

El riesgo no se limita a mostrar un último decimal distinto. Puede propagarse a comparaciones con tick size, cálculo de exposición, PnL, límites, agregación, reconciliación y persistencia si las velas pasan a ser entrada de lógica financiera.

Hasta que se resuelva el modelo canónico, las velas flotantes deben considerarse una representación de visualización/compatibilidad y no una fuente autoritativa para decisiones financieras.

## Alternativas a evaluar

### Decimal de precisión fija

Representación basada en decimal de precisión fija, como el tipo actualmente utilizado por `Price` y `Quantity`.

Ventajas:
- Semántica decimal directa y familiar para precios y cantidades.
- Operaciones y serialización decimal disponibles en el ecosistema Rust.
- Menor fricción inicial con los value objects existentes.

Riesgos y límites:
- Rango y escala máximos finitos que deben contrastarse con todos los instrumentos.
- División y operaciones no exactas requieren política explícita de redondeo.
- El coste debe medirse en las operaciones y tamaños de lote reales.
- No sustituye la necesidad de validar tick, lot, multiplicador y reglas específicas del instrumento.

### Unidades enteras escaladas

Representación como entero más escala explícita, o una unidad mínima definida por metadatos del instrumento.

Ventajas:
- Comparaciones y sumas exactas dentro del rango representable.
- Reglas de tick/step pueden expresarse en unidades discretas.
- El overflow puede detectarse mediante operaciones checked.

Riesgos y límites:
- Escalas variables y ticks no uniformes requieren un modelo de metadatos sólido.
- Multiplicación, división, ratios y conversiones entre monedas requieren escalas intermedias y redondeo definidos.
- Un único `i64` no cubre necesariamente todos los rangos y precisiones.
- Cambios de escala/versiones de instrumento deben ser trazables.

### Modelo híbrido explícito

Mantener una representación financiera autoritativa decimal o escalada y proyectar a `f64` únicamente en el límite de geometría/renderizado.

Ventajas:
- Separa exactitud financiera de requisitos gráficos.
- Evita exigir aritmética exacta en coordenadas de pantalla.
- Permite optimizar el renderer sin degradar el dato canónico.

Riesgos y límites:
- Requiere fronteras de conversión visibles y unidireccionales.
- La proyección gráfica debe tolerar rangos grandes y valores no representables, con política de error/degradación.
- Debe impedirse que los valores proyectados vuelvan a entrar en cálculos financieros.

## Criterios obligatorios para la decisión

La ADR numérica debe evaluar y documentar:

1. Mercados, venues, clases de instrumento y proveedores soportados.
2. Máximo/mínimo de precio, cantidad, volumen, notional y multiplicador.
3. Escala máxima de entrada y precisión contractual de cada proveedor.
4. Tick size y step size, incluidos cambios por bandas o vigencia.
5. Operaciones requeridas: suma, resta, producto, división, ratios, conversión FX, PnL y agregación.
6. Política de redondeo por operación y contexto; ninguna conversión silenciosa.
7. Detección de overflow y comportamiento ante valores fuera de rango.
8. Parseo exacto del payload original, serialización y round-trip.
9. Rendimiento medido con datos representativos y benchmarks reproducibles.
10. Interoperabilidad con persistencia, exportación y APIs externas.

No se debe seleccionar un tipo numérico solo porque sea popular o porque ya exista como dependencia.

## Arquitectura numérica objetivo

- El adaptador de proveedor conserva el valor decimal de entrada antes de cualquier conversión binaria.
- La validación de entrada aplica escala, rango, tick/step y semántica de instrumento usando metadatos versionados.
- El modelo de mercado conserva los valores financieros canónicos.
- Los cálculos financieros operan exclusivamente sobre tipos canónicos.
- La conversión a `f64` ocurre en un adaptador de visualización explícito.
- Las conversiones que pueden perder precisión son identificables, comprobables y no reversibles por diseño.
- La persistencia conserva valor, escala, unidad, identidad del instrumento y versión de metadatos suficientes para reproducir el significado original.

## Matriz mínima de pruebas

| Área | Casos requeridos | Invariante |
|---|---|---|
| Parseo | Decimal con muchos dígitos, ceros finales, notación admitida por proveedor | El valor canónico coincide con el decimal de origen |
| Round-trip | Parsear, serializar y volver a parsear | No cambia valor ni escala semántica |
| Escala | Límite permitido y un dígito por encima | Aceptación/rechazo determinista |
| Tick/step | Múltiplo exacto y valor entre incrementos | Solo se acepta según regla explícita del instrumento |
| Rango | Mínimo, máximo y valores exteriores | Sin wrap, saturación ni truncamiento silencioso |
| Overflow | Suma/producto/división en límites | Error explícito o resultado definido por contrato |
| Redondeo | Mitad exacta, negativos si aplican, divisiones periódicas | Regla documentada y estable |
| Conversión gráfica | Valores extremos y escalas muy pequeñas/grandes | Proyección finita o error explícito; nunca modifica el canónico |
| Instrumento | Cambio de tick/escala con vigencia | Se aplica la versión correcta a cada evento |
| Propiedades | Operaciones y serialización generadas por property testing | Se preservan invariantes para todo el dominio probado |

## Puertas de aceptación

No se habilitarán cálculos financieros sobre OHLCV mientras el único valor disponible provenga de `f64` sin procedencia decimal recuperable.

No se migrarán tipos públicos hasta:
- Identificar el alcance inicial de mercados/proveedores.
- Aprobar ADR con análisis de rango, escala, redondeo y benchmark.
- Definir tipos canónicos y conversiones explícitas.
- Añadir pruebas de parseo, límites, overflow y round-trip.
- Verificar que la UI solo consume una proyección y no realimenta el dominio financiero.

## Próxima decisión

La decisión pendiente no es simplemente “Decimal o enteros”. Primero se debe cerrar el universo inicial de instrumentos y proveedores, porque ese universo determina los rangos, escalas, calendarios y reglas de precisión que el modelo debe representar sin pérdida.
