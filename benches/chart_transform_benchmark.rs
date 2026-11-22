use std::hint::black_box;
use std::time::Instant;

use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

/// Benchmark de línea base para el motor de gráficos (chart engine).
///
/// Mide cuánto tarda en transformar coordenadas de datos (precio, tiempo)
/// a coordenadas de pantalla para N velas, usando las funciones reales de
/// `Viewport` (no fórmulas reimplementadas).
///
/// Esto nos da una línea base medible del rendimiento del chart engine.
/// Cuando se agregue wgpu como backend principal, este benchmark se expande
/// para incluir subida de buffers GPU y draw calls.
///
/// No optimicemos por intuición: este benchmark es la evidencia de referencia.
/// Cualquier cambio en `domain/viewport.rs`, `domain/transforms.rs` o
/// `renderer/` debe mantener o mejorar estos números.
fn bench_viewport_transforms(iterations: usize) {
    let mut viewport = Viewport::new();
    viewport.set_zoom(2.0, 2.0);
    viewport.set_offset_x(50.0);
    viewport.set_offset_y(30.0);

    let screen_width: f64 = 1920.0;
    let screen_height: f64 = 1080.0;

    let min_price = Price::new(Decimal::new(4950000, 2)).expect("49500.00 válido");
    let max_price = Price::new(Decimal::new(5050000, 2)).expect("50500.00 válido");
    let price_range = PriceRange::new(min_price, max_price).expect("rango válido");
    let center_price = Price::new(Decimal::new(5000000, 2)).expect("50000.00 válido");

    // Generamos N velas sintéticas con precios realistas (BTC/USD-style)
    let mut candles_prices: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(iterations);
    let mut price = 50000.0;
    for i in 0..iterations {
        let change = (i as f64 * 0.1).sin() * 50.0;
        let open = price;
        let close = open + change;
        let high = open.max(close) + 20.0;
        let low = open.min(close) - 20.0;
        candles_prices.push((open, high, low, close));
        price = close;
    }

    let start = Instant::now();

    // Camino crítico: transformar cada vela a coordenadas de pantalla
    // usando las funciones reales de Viewport.
    for (i, (open, high, low, close)) in candles_prices.iter().enumerate() {
        let _screen_x = viewport.data_x_to_screen(i, screen_width);
        let _y_high = viewport.price_to_screen(
            Price::new(Decimal::from_f64(*high).unwrap_or(Decimal::ZERO)).expect("high válido"),
            screen_height,
            price_range,
            center_price,
        );
        let _y_low = viewport.price_to_screen(
            Price::new(Decimal::from_f64(*low).unwrap_or(Decimal::ZERO)).expect("low válido"),
            screen_height,
            price_range,
            center_price,
        );
        let _y_open = viewport.price_to_screen(
            Price::new(Decimal::from_f64(*open).unwrap_or(Decimal::ZERO)).expect("open válido"),
            screen_height,
            price_range,
            center_price,
        );
        let _y_close = viewport.price_to_screen(
            Price::new(Decimal::from_f64(*close).unwrap_or(Decimal::ZERO)).expect("close válido"),
            screen_height,
            price_range,
            center_price,
        );

        // black_box evita que el optimizador elimine el cálculo como código muerto.
        black_box((_screen_x, _y_high, _y_low, _y_open, _y_close));
    }

    let elapsed = start.elapsed();
    let per_candle = elapsed.as_nanos() / iterations as u128;

    println!(
        "chart_transform_viewport: {} candles in {:?} ({} ns/candle)",
        iterations, elapsed, per_candle
    );
}

fn main() {
    // Diferentes tamaños para detectar degradación no lineal (ej. O(n²) accidental)
    for count in [1_000, 10_000, 100_000, 500_000] {
        bench_viewport_transforms(count);
    }
}
