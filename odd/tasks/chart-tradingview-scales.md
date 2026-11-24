# Chart: escalas y layout estilo TradingView

**Objetivo:** mejorar la presentación y la interacción de las escalas de precio y tiempo, manteniendo una única fuente de verdad para layout, coordenadas e input.

**Alcance de esta etapa:** escalas y presentación del chart Glow. Replay queda explícitamente fuera de alcance.

## Entregables

- [x] Escala de precios a la derecha, con ancho legible y área de interacción propia.
- [x] Gutter izquierdo reducido para maximizar el área de plot.
- [x] Eje temporal en una franja inferior separada del volumen.
- [x] Panel de volumen con altura suficiente y escala independiente de etiquetas.
- [x] Rejilla horizontal/vertical alineada con los ticks de precio y tiempo.
- [x] Ticks de precio adaptativos y etiquetas sin notación científica.
- [x] Etiqueta del precio actual en la escala derecha.
- [x] Etiquetas del crosshair en las bandas de precio y tiempo.
- [x] Arrastre y doble clic de la escala derecha conectados al controlador existente.
- [x] Alineación del origen vertical de velas, escala y crosshair.
- [x] Reserva de espacio para el chart y su barra de estado, evitando que los textos se superpongan con el gráfico.

## Validación

- Tests unitarios del layout y del cálculo de ticks.
- Tests de interacción para la escala derecha y el eje temporal.
- `cargo fmt --check`, `cargo check`, `cargo test --workspace`, `cargo clippy -- -D warnings` y workflow CI.


## Refinamiento de la escala temporal

- [x] Selección de intervalos temporales adaptativos a la densidad visible.
- [x] Alineación de ticks a buckets de reloj UTC (minutos, horas y días), no al índice visible inicial.
- [x] Etiquetas compactas que alternan hora, fecha, mes/año y año según el intervalo.
- [x] Separación mínima proporcional al texto para evitar colisiones entre etiquetas.
- [x] Cobertura de cambios de día, intervalos diarios y series con huecos.
