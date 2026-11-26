





use crate::chart::hit_testing::result::{HitKind, HitResult, HitSeriesId, HitTestContext};
use crate::chart::hit_testing::r#trait::HitTestable;
use crate::domain::transforms::ScreenPoint;


#[derive(Debug, Clone)]
pub struct HitTesterResult {

    pub hit: HitResult,

    pub object_index: usize,
}

pub fn hit_test_objects<T>(objects: &[T], ctx: &HitTestContext<'_>) -> Option<HitTesterResult>
where
    T: HitTestable,
{
    if objects.is_empty() || !ctx.is_inside_plot() {
        return None;
    }

    let mut best: Option<HitTesterResult> = None;

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



pub fn hit_test_all<T>(objects: &[T], ctx: &HitTestContext<'_>) -> Vec<(usize, HitResult)>
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






#[derive(Debug, Clone, Copy)]
pub struct CandleHitTest {
    pub index: usize,
    pub center_x: f64,
    pub y_open: f64,
    pub y_close: f64,
    pub price: f64,
    pub series_id: HitSeriesId,
    pub tolerance: f64,
}

impl HitTestable for CandleHitTest {
    fn hit_test(&self, ctx: &HitTestContext) -> Option<HitResult> {
        let dx = (ctx.screen_x - self.center_x).abs();
        let y_min = self.y_open.min(self.y_close);
        let y_max = self.y_open.max(self.y_close);
        let dy = if ctx.screen_y >= y_min - self.tolerance && ctx.screen_y <= y_max + self.tolerance
        {
            0.0
        } else {
            (ctx.screen_y - y_min)
                .abs()
                .min((ctx.screen_y - y_max).abs())
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
        HitTestContext::new(viewport, layout, screen_x, screen_y, 100.0, 50000.0, 100)
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


        let back = CandleHitTest {
            index: 0,
            center_x: 500.0,
            y_open: 290.0,
            y_close: 310.0,
            price: 50200.0,
            series_id: HitSeriesId(1),
            tolerance: 10.0,
        };

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

        assert_eq!(hits[0].0, 1);
        assert_eq!(hits[1].0, 0);
    }
}
