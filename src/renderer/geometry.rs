use crate::renderer::{
    CandleRender, RenderColor, RenderGeometry, RenderLayer, RenderVertex, VolumeBarRender,
};

fn push_rectangle(
    vertices: &mut Vec<RenderVertex>,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    color: RenderColor,
) {
    if ![left, top, right, bottom]
        .iter()
        .all(|value| value.is_finite())
    {
        return;
    }

    let (left, right) = (left.min(right), left.max(right));
    let (top, bottom) = (top.min(bottom), top.max(bottom));

    if right <= left || bottom <= top {
        return;
    }

    let top_left = RenderVertex {
        position: [left, top],
        color,
    };
    let top_right = RenderVertex {
        position: [right, top],
        color,
    };
    let bottom_right = RenderVertex {
        position: [right, bottom],
        color,
    };
    let bottom_left = RenderVertex {
        position: [left, bottom],
        color,
    };

    vertices.extend([
        top_left,
        top_right,
        bottom_right,
        top_left,
        bottom_right,
        bottom_left,
    ]);
}

fn push_wick(vertices: &mut Vec<RenderVertex>, candle: &CandleRender) {
    push_rectangle(
        vertices,
        candle.center_x - 0.5,
        candle.high_y,
        candle.center_x + 0.5,
        candle.low_y,
        candle.color,
    );
}

fn push_body(vertices: &mut Vec<RenderVertex>, candle: &CandleRender) {
    if !candle.open_y.is_finite() || !candle.close_y.is_finite() {
        return;
    }

    let center_y = (candle.open_y + candle.close_y) * 0.5;
    let half_height = ((candle.open_y - candle.close_y).abs() * 0.5).max(0.5);
    let half_width = candle.body_width.abs() * 0.5;

    push_rectangle(
        vertices,
        candle.center_x - half_width,
        center_y - half_height,
        candle.center_x + half_width,
        center_y + half_height,
        candle.color,
    );
}

fn push_volume(vertices: &mut Vec<RenderVertex>, bar: &VolumeBarRender, width: f32) {
    if !bar.height.is_finite() || bar.height <= 0.0 || !bar.base_y.is_finite() {
        return;
    }

    let half_width = width.abs() * 0.5;
    push_rectangle(
        vertices,
        bar.center_x - half_width,
        bar.base_y - bar.height,
        bar.center_x + half_width,
        bar.base_y,
        bar.color,
    );
}

pub fn build_geometry(layer: &RenderLayer) -> RenderGeometry {
    let mut geometry = RenderGeometry {
        vertices: Vec::with_capacity(
            layer
                .candles
                .len()
                .saturating_add(layer.volume_bars.len())
                .saturating_mul(18),
        ),
    };

    for candle in &layer.candles {
        push_wick(&mut geometry.vertices, candle);
        push_body(&mut geometry.vertices, candle);
    }

    for (index, bar) in layer.volume_bars.iter().enumerate() {
        let width = layer
            .candles
            .get(index)
            .filter(|candle| candle.index == bar.index)
            .map(|candle| candle.body_width)
            .unwrap_or(1.0);
        push_volume(&mut geometry.vertices, bar, width);
    }

    geometry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::RenderLayer;

    fn sample_layer() -> RenderLayer {
        RenderLayer {
            candles: vec![CandleRender {
                index: 7,
                open_y: 30.0,
                high_y: 10.0,
                low_y: 50.0,
                close_y: 40.0,
                center_x: 20.0,
                body_width: 8.0,
                color: RenderColor::GREEN,
                volume: 500.0,
            }],
            volume_bars: vec![VolumeBarRender {
                index: 7,
                center_x: 20.0,
                height: 12.0,
                base_y: 90.0,
                color: RenderColor::GREEN,
            }],
            ..RenderLayer::default()
        }
    }

    #[test]
    fn builds_wick_body_and_volume_as_triangle_list_quads() {
        let geometry = build_geometry(&sample_layer());

        assert_eq!(geometry.vertices.len(), 18);
        assert!(geometry.vertices.iter().all(|vertex| {
            vertex
                .position
                .iter()
                .all(|coordinate| coordinate.is_finite())
        }));
        assert!(
            geometry
                .vertices
                .iter()
                .all(|vertex| vertex.color == RenderColor::GREEN)
        );
    }

    #[test]
    fn empty_layer_produces_empty_geometry() {
        assert!(build_geometry(&RenderLayer::default()).vertices.is_empty());
    }

    #[test]
    fn invalid_coordinates_are_not_uploaded_as_vertices() {
        let mut layer = sample_layer();
        layer.candles[0].high_y = f32::NAN;
        layer.candles[0].low_y = f32::INFINITY;
        layer.candles[0].open_y = f32::NAN;
        layer.candles[0].close_y = f32::INFINITY;
        layer.volume_bars[0].height = f32::NAN;

        assert!(build_geometry(&layer).vertices.is_empty());
    }

    #[test]
    fn zero_height_candles_keep_a_visible_body() {
        let mut layer = sample_layer();
        layer.candles[0].open_y = 25.0;
        layer.candles[0].close_y = 25.0;

        let geometry = build_geometry(&layer);

        assert_eq!(geometry.vertices.len(), 18);
        let body = &geometry.vertices[6..12];
        let min_y = body
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = body
            .iter()
            .map(|vertex| vertex.position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(max_y - min_y, 1.0);
    }
}
