use std::hint::black_box;
use std::time::Instant;

/// Benchmark de línea base para el motor de gráficos (chart engine).
///
/// Mide cuánto tarda en transformar coordenadas de datos (precio, tiempo)
/// a coordenadas de pantalla para N velas, usando la lógica de `Viewport`.
///
/// Esto nos da una línea base medible del rendimiento del chart engine.
/// Cuando se agregue wgpu como backend principal, este benchmark se expande
/// para incluir subida de buffers GPU y draw calls.
///
/// No optimicemos por intuición: este benchmark es la evidencia de referencia.
/// Cualquier cambio en `domain/viewport.rs` o `renderer/` debe mantener o
/// mejorar estos números.
fn bench_transform_candles(iterations: usize) {
    // Viewport típico: zoom 2.0, pan moderado
    let zoom_level: f64 = 2.0;
    let offset_x: f64 = 50.0;
    let offset_y: f64 = 30.0;

    let screen_width: f64 = 1920.0;
    let screen_height: f64 = 1080.0;
    let price_range: f64 = 1000.0;
    let center_price: f64 = 50000.0;

    // Generamos N velas sintéticas con precios realistas (BTC/USD-style)
    let mut candles: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(iterations);
    let mut price = 50000.0;
    for i in 0..iterations {
        let change = (i as f64 * 0.1).sin() * 50.0;
        let open = price;
        let close = open + change;
        let high = open.max(close) + 20.0;
        let low = open.min(close) - 20.0;
        candles.push((open, high, low, close));
        price = close;
    }

    // Calculo de constantes fuera del bucle medido (no forman parte del camino crítico)
    let candle_spacing = 20.0 / zoom_level;
    let center_x = screen_width / 2.0;
    let center_y = screen_height / 2.0;
    let pixels_per_unit = screen_height / price_range * zoom_level;

    let start = Instant::now();

    // Camino crítico: transformar cada vela a coordenadas de pantalla
    for (i, (open, high, low, close)) in candles.iter().enumerate() {
        let _screen_x = center_x + (i as f64 * candle_spacing) + offset_x * candle_spacing;

        let _y_high = center_y - (high - center_price - offset_y * price_range) * pixels_per_unit;
        let _y_low = center_y - (low - center_price - offset_y * price_range) * pixels_per_unit;
        let _y_open = center_y - (open - center_price - offset_y * price_range) * pixels_per_unit;
        let _y_close = center_y - (close - center_price - offset_y * price_range) * pixels_per_unit;

        // black_box evita que el optimizador elimine el cálculo como código muerto.
        black_box((_screen_x, _y_high, _y_low, _y_open, _y_close));
    }

    let elapsed = start.elapsed();
    let per_candle = elapsed.as_nanos() / iterations as u128;

    println!(
        "chart_transform: {} candles in {:?} ({} ns/candle)",
        iterations, elapsed, per_candle
    );
}

fn main() {
    // Diferentes tamaños para detectar degradación no lineal (ej. O(n²) accidental)
    for count in [1_000, 10_000, 100_000, 500_000] {
        bench_transform_candles(count);
    }
}
