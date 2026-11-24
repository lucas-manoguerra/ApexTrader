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
- [x] Arrastre, rueda del mouse y doble clic de la escala derecha conectados al controlador existente.
- [x] Zoom vertical con la rueda sobre el eje de precios, anclado al cursor y respetando el bloqueo vertical.
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


## Refinamiento de la escala de precios

- [x] Mantener rango y centro de referencia dentro del Viewport.
- [x] Autoescala calculada con máximos/mínimos de las velas visibles.
- [x] Modo manual: conservar rango vertical al llegar nuevas velas realtime.
- [x] Bloqueo vertical independiente del modo Auto/Manual.
- [x] Arrastre vertical y zoom manual desactivan Auto sin saltos del rango de referencia.
- [x] Doble clic en la escala de precios vuelve al modo Auto.
- [x] Controles visibles para Auto y bloqueo vertical.
- [x] Ticks alineados al incremento mínimo del instrumento y precisión decimal coherente.
- [x] Separadores de miles consistentes en etiquetas del eje, marcador actual y etiqueta del crosshair.
- [x] Pruebas de persistencia manual, bloqueo, transiciones y precisión de ticks.

- [x] Criterio de autoescala seleccionable: extremos OHLC o solo cierres.
- [x] El criterio seleccionado se aplica al rango de velas visibles y se conserva en el viewport.
- [x] Cobertura de pruebas para ambos criterios y para el valor predeterminado.
- [x] Márgenes superior e inferior configurables e independientes (0% a 45% cada uno).
- [x] Autoescala asimétrica: los extremos visibles se ubican en los límites definidos.
- [x] Controles de margen en porcentaje y restablecimiento rápido a 5% / 5%.
- [x] Validación de límites y preservación de la escala manual al cambiar la configuración.
- [x] Modos de escala Lineal, Logarítmica y Porcentual.
- [x] Base porcentual inicial vinculada al primer cierre del historial y actualizada al cambiar de símbolo.
- [x] Base porcentual reconfigurable desde la interfaz entre el primer cierre y el último cierre cargado.
- [x] El cambio de base porcentual restablece la autoescala y elimina el desplazamiento vertical anterior.
- [x] Ticks y etiquetas consistentes con cada transformación.

**Nota:** el incremento mínimo se infiere con reglas limitadas para el mock (BTC/USDT, futuros CME comunes y pares FX). El proveedor real deberá suministrar el tick size exacto del contrato/instrumento.
