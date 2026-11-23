//! Orquestrador de hit-testing sobre múltiples objetos visuales.
//!
//! Recibe una lista ordenada de objetos testables (por z-order: los últimos
//! en la lista se dibujan encima, por lo que se prueban primero) y devuelve
//! el mejor hit, o `None` si ningún elemento está bajo el cursor.

use crate::chart::hit_testing::result::{
    HitKind, HitResult, HitSeriesId, HitTestContext,
};
use crate::chart::hit_testing::r#trait::HitTestable;
use crate::domain::transforms::ScreenPoint;

/// Resultado final del hit-tester: el mejor hit entre todos los objetos.
#[derive(Debug, Clone)]
pub struct HitTesterResult {
    /// El elemento más cercano al cursor (o el primero si hay empate).
    pub hit: HitResult,
    /// Índice del objeto en la lista original que produjo el hit.
    pub object_index: usize,
}

/// Ejecuta hit-testing sobre una lista de objetos testables.
///
/// `objects` está ordenado por z-order ascendente (primero = fondo,
/// último = frente). Se itera de atrás hacia adelante para que los
/// objetos del frente tengan prioridad. Devuelve el hit con menor
/// distancia; si hay empate, gana el de mayor índice (más al frente).
pub fn hit_test_objects<'a, T>(
    objects: &'a [T],
    ctx: &HitTestContext<'_>,
) -> Option<HitTesterResult>
where
    T: HitTestable,
{
    if objects.is_empty() || !ctx.is_inside_plot() {
        return None;
    }

    let mut best: Option<HitTesterResult> = None;

    // Iterar de atrás hacia adelante (z-order: frente primero).
    for (index, object) in objects.iter().enumerate().rev() {
        if let Some(hit) = object.hit_test(ctx) {
            let candidate = HitTesterResult {
                hit,
                object_index: index,
            };

            match &best {
                None => {
                    best = Some(candidate);
                }
                Some(current) => {
                    // Menor distancia gana; si hay empate, el de mayor
                    // índice (más al frente en z-order) gana.
                    if candidate.hit.distance < current.hit.distance
                        || (candidate.hit.distance - current.hit.distance).abs() < f64::EPSILON
                            && candidate.object_index > current.object_index
                    {
                        best = Some(candidate);
                    }
                }
            }
        }
    }

    best
}

/// Devuelve todos los hits (no solo el mejor), ordenados por z-order
/// descendente (frente primero).
pub fn hit_test_all<'a, T>(
    objects: &'a [T],
    ctx: &HitTestContext<'_>,
) -> Vec<(usize, HitResult)>
where
    T: HitTestable,
{
    let mut hits = Vec::new();

    if objects.is_empty() || !ctx.is_inside_plot() {
        return hits;
    }

    for (index, object) in objects.iter().enumerate().rev() {
        if let Some(hit) = object.hit_test(ctx) {
            hits.push((index, hit));
        }
    }

    hits
}

/// Ejemplo de objeto testable: una vela individual.
///
/// Ilustra cómo implementar `HitTestable` para una vela de una serie.
/// La tolerancia de selección es de 6 píxeles alrededor del centro X de
/// la vela y de todo el cuerpo en Y.
#[derive(Debug, Clone, Copy)]
pub struct CandleHitTest {
    /// Índice de la vela en la serie.
    pub index: usize,
    /// Posición X del centro de la vela en pantalla.
    pub center_x: f64,
    /// Posición Y del precio de apertura en pantalla.
    pub y_open: f64,
    /// Posición Y del precio de cierre en pantalla.
    pub y_close: f64,
    /// Precio de la vela (para el data_point del hit).
    pub price: f64,
    /// ID de serie asociada.
    pub series_id: HitSeriesId,
    /// Tolerancia de selección en píxeles.
    pub tolerance: f64,
}

impl HitTestable for CandleHitTest {
    fn hit_test(&self, ctx: &HitTestContext) -> Option<HitResult> {
        let dx = (ctx.screen_x - self.center_x).abs();
        // La vela tiene ancho en X (tolerancia) y cubre todo el rango Y
        // entre open y close (más un margen pequeño).
        let y_min = self.y_open.min(self.y_close);
        let y_max = self.y_open.max(self.y_close);
        let dy = if ctx.screen_y >= y_min - self.tolerance
            && ctx.screen_y <= y_max + self.tolerance
        {
            0.0
        } else {
            (ctx.screen_y - y_min).abs().min((ctx.screen_y - y_max).abs())
        };

        let distance = (dx * dx + dy * dy).sqrt();
        if distance > self.tolerance {
            return None;
        }

        Some(HitResult {
            kind: HitKind::Candle { index: self.index },
            screen_point: ScreenPoint {
                x: self.center_x,
                y: ctx.screen_y,
            },
            data_point: crate::domain::transforms::DataPoint {
                index: self.index as f64,
                price: self.price,
            },
            distance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::layout::ChartLayout;
    use crate::domain::viewport::Viewport;

    fn make_viewport() -> Viewport {
        let mut v = Viewport::new();
        v.set_zoom(2.0, 2.0);
        v
    }

    fn make_layout() -> ChartLayout {
        ChartLayout::compute(1920.0, 1080.0)
    }

    fn make_ctx<'a>(
        viewport: &'a Viewport,
        layout: &'a ChartLayout,
        screen_x: f64,
        screen_y: f64,
    ) -> HitTestContext<'a> {
        HitTestContext::new(
            viewport,
            layout,
            screen_x,
            screen_y,
            100.0,
            50000.0,
            100,
        )
    }

    #[test]
    fn hit_test_empty_objects_returns_none() {
        let viewport = make_viewport();
        let layout = make_layout();
        let ctx = make_ctx(&viewport, &layout, 100.0, 100.0);
        let objects: Vec<CandleHitTest> = Vec::new();
        assert!(hit_test_objects(&objects, &ctx).is_none());
    }

    #[test]
    fn hit_test_candle_inside_tolerance() {
        let viewport = make_viewport();
        let layout = make_layout();
        let ctx = make_ctx(&viewport, &layout, 500.0, 300.0);

        let candle = CandleHitTest {
            index: 3,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50250.0,
            series_id: HitSeriesId(1),
            tolerance: 6.0,
        };
        let objects = vec![candle];

        let result = hit_test_objects(&objects, &ctx).expect("debe haber hit");
        assert_eq!(result.object_index, 0);
        match result.hit.kind {
            HitKind::Candle { index } => assert_eq!(index, 3),
            _ => panic!("esperaba HitKind::Candle"),
        }
        assert!(result.hit.distance <= 6.0);
    }

    #[test]
    fn hit_test_candle_outside_tolerance() {
        let viewport = make_viewport();
        let layout = make_layout();
        // Cursor lejos de la vela (100 píxeles de distancia en X).
        let ctx = make_ctx(&viewport, &layout, 600.0, 300.0);

        let candle = CandleHitTest {
            index: 3,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50250.0,
            series_id: HitSeriesId(1),
            tolerance: 6.0,
        };
        let objects = vec![candle];

        assert!(hit_test_objects(&objects, &ctx).is_none());
    }

    #[test]
    fn hit_test_z_order_front_wins() {
        let viewport = make_viewport();
        let layout = make_layout();
        let ctx = make_ctx(&viewport, &layout, 500.0, 300.0);

        // Fondo: vela en el centro.
        let back = CandleHitTest {
            index: 0,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50200.0,
            series_id: HitSeriesId(1),
            tolerance: 10.0,
        };
        // Frente: otra vela en el mismo punto (dibujada después).
        let front = CandleHitTest {
            index: 1,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50300.0,
            series_id: HitSeriesId(2),
            tolerance: 10.0,
        };

        let objects = vec![back, front];
        let result = hit_test_objects(&objects, &ctx).expect("debe haber hit");

        // El de frente (índice 1) debe ganar por z-order.
        assert_eq!(result.object_index, 1);
        match result.hit.kind {
            HitKind::Candle { index } => assert_eq!(index, 1),
            _ => panic!("esperaba vela del frente"),
        }
    }

    #[test]
    fn hit_test_outside_plot_returns_none() {
        let viewport = make_viewport();
        let layout = make_layout();
        // Cursor fuera del plot (izquierda).
        let ctx = make_ctx(&viewport, &layout, -10.0, 300.0);

        let candle = CandleHitTest {
            index: 0,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50250.0,
            series_id: HitSeriesId(1),
            tolerance: 10.0,
        };
        let objects = vec![candle];

        assert!(hit_test_objects(&objects, &ctx).is_none());
    }

    #[test]
    fn hit_test_all_returns_multiple() {
        let viewport = make_viewport();
        let layout = make_layout();
        let ctx = make_ctx(&viewport, &layout, 500.0, 300.0);

        let c1 = CandleHitTest {
            index: 1,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50200.0,
            series_id: HitSeriesId(1),
            tolerance: 10.0,
        };
        let c2 = CandleHitTest {
            index: 2,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50300.0,
            series_id: HitSeriesId(2),
            tolerance: 10.0,
        };

        let objects = vec![c1, c2];
        let hits = hit_test_all(&objects, &ctx);
        assert_eq!(hits.len(), 2);
        // hit_test_all itera de atrás hacia adelante: c2 (índice 1) primero.
        assert_eq!(hits[0].0, 1);
        assert_eq!(hits[1].0, 0);
    }
}
