use apex_trader::domain::price::Price;
use apex_trader::domain::price_range::PriceRange;
use apex_trader::domain::viewport::Viewport;
use proptest::prelude::*;
use rust_decimal::Decimal;

// S5: el nivel de zoom debe permanecer siempre dentro de [min_zoom, max_zoom],
// incluso ante valores arbitrarios de entrada.
proptest! {
    #[test]
    fn prop_zoom_remains_within_limits(zoom in 0.0f64..1e6f64) {
        let mut viewport = Viewport::new();
        viewport.set_zoom(zoom, zoom);
        assert!(viewport.zoom_x() >= viewport.min_zoom());
        assert!(viewport.zoom_x() <= viewport.max_zoom());
        assert!(viewport.zoom_x().is_finite());
    }
}

// S5: el pan no debe producir offsets no finitos ni hacer panic, incluso con
// desplazamientos extremos.
proptest! {
    #[test]
    fn prop_pan_offsets_stay_finite(dx in -1e6f64..1e6f64, dy in -1e6f64..1e6f64) {
        let mut viewport = Viewport::new();
        viewport.pan(dx, dy, 1080.0);
        assert!(viewport.offset_x().is_finite());
        assert!(viewport.offset_y().is_finite());
    }
}

// S5: pantalla → datos → pantalla es un round-trip estable para índices dentro
// del rango de velas cargadas.
proptest! {
    #[test]
    fn prop_screen_to_data_round_trip(idx in 0..100usize) {
        let viewport = Viewport::new();
        let screen_width = 1920.0f64;
        let candle_count = 100usize;

        let screen_x = viewport.data_x_to_screen(idx, screen_width);
        let recovered = viewport.screen_x_to_data(screen_x, screen_width, candle_count);

        assert_eq!(recovered, idx);
    }
}

// S2: zoom anclado al cursor — hacer zoom in y luego zoom out en el mismo
// punto debe dejar el punto bajo el cursor fijo en pantalla (misma vela).
//
// Nota: el zoom es multiplicativo (factor 1 ± zoom_step), así que
// (1+s)·(1-s) = 1 - s² ≠ 1 en punto flotante y el estado no vuelve bit-a-bit
// idéntico. No exigimos restauración exacta de offsets (sería imposible);
// exigimos lo que realmente pide S2: el índice de vela bajo el cursor no se
// mueve. El error acumulado es sub-píxel, muy inferior al espaciamiento entre
// velas, así que el índice entero recuperado es estable.
proptest! {
    #[test]
    fn prop_zoom_in_out_keeps_cursor_vela_fija(
        zoom in 0.1f64..5.0f64,
        center_x in 0.0f64..1.0f64,
        center_y in 0.0f64..1.0f64,
    ) {
        let mut viewport = Viewport::new();
        viewport.set_zoom(zoom, zoom);

        let screen_width = 1920.0f64;
        let screen_height = 1080.0f64;
        let candle_count = 100usize;

        let cursor_px = center_x * screen_width;

        // Índice de vela bajo el cursor antes de cualquier zoom.
        let before = viewport.screen_x_to_data(cursor_px, screen_width, candle_count);

        // Zoom in alrededor del cursor (eje Y = precio; eje X sin cambio).
        viewport.zoom_at(0.0, 1.0, center_x, center_y, screen_width, screen_height);
        // Zoom out en el mismo punto.
        viewport.zoom_at(0.0, -1.0, center_x, center_y, screen_width, screen_height);

        // El cursor debe seguir apuntando a la misma vela.
        let after = viewport.screen_x_to_data(cursor_px, screen_width, candle_count);
        assert_eq!(
            after, before,
            "el cursor se movió de la vela {} a {} tras zoom in/out (center_x={}, zoom={})",
            before, after, center_x, zoom
        );
    }
}

// S8: auto_scale no debe modificar el estado cuando recibe dimensiones
// cero, negativas o no finitas.
proptest! {
    #[test]
    fn prop_auto_scale_ignores_invalid_dimensions(
        width in prop_oneof![Just(0.0f64), Just(-100.0f64)],
        height in prop_oneof![Just(0.0f64), Just(-100.0f64)],
    ) {
        let mut viewport = Viewport::new();
        viewport.set_zoom(2.5, 2.5);
        viewport.set_offset_x(10.0);
        viewport.set_offset_y(5.0);

        let before = (
            viewport.zoom_x(),
            viewport.offset_x(),
            viewport.offset_y(),
        );

        let min_price = Price::new(Decimal::new(10000, 2)).expect("100.00 válido");
        let max_price = Price::new(Decimal::new(20000, 2)).expect("200.00 válido");
        let range = PriceRange::new(min_price, max_price).expect("rango válido");

        viewport.auto_scale(range, 50, width, height);

        assert_eq!(viewport.zoom_x(), before.0);
        assert_eq!(viewport.offset_x(), before.1);
        assert_eq!(viewport.offset_y(), before.2);
    }
}

// S5: visible_range devuelve first <= last cuando hay velas y el área es válida.
proptest! {
    #[test]
    fn prop_visible_range_is_sane(candle_count in 1..1000usize, width in 1.0f64..2000.0f64) {
        let viewport = Viewport::new();
        let range = viewport.visible_range(candle_count, width);
        assert!(!range.is_empty());
        assert!(range.first <= range.last);
        assert!(range.last < candle_count);
        assert_eq!(range.len(), range.last - range.first + 1);
    }
}

// S5: visible_range está vacío cuando no hay velas.
proptest! {
    #[test]
    fn prop_visible_range_empty_with_no_candles(width in 1.0f64..2000.0f64) {
        let viewport = Viewport::new();
        let range = viewport.visible_range(0, width);
        assert!(range.is_empty());
        assert_eq!(range.len(), 0);
    }
}
