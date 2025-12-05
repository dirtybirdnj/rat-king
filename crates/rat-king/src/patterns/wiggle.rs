//! Wiggle fill pattern - sinusoidal wavy lines.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate wiggle (sinusoidal wave) fill for a polygon.
///
/// Creates parallel rows of smooth sine wave lines.
pub fn generate_wiggle_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    amplitude: f64,
    frequency: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let angle_rad = angle_degrees * PI / 180.0;

    // Diagonal coverage
    let diagonal = (width * width + height * height).sqrt() * 1.42;

    // Direction vectors
    let perp_x = (angle_rad + PI / 2.0).cos();
    let perp_y = (angle_rad + PI / 2.0).sin();
    let dir_x = angle_rad.cos();
    let dir_y = angle_rad.sin();

    let center_x = min_x + width / 2.0;
    let center_y = min_y + height / 2.0;
    let num_rows = (diagonal / spacing).ceil() as i32 + 1;

    // Segment length for approximating curve
    let segment_length = 2.0;
    let num_segments = (diagonal * 2.0 / segment_length) as i32;

    let mut lines = Vec::new();

    for i in -num_rows..=num_rows {
        let offset = i as f64 * spacing;
        let row_center_x = center_x + perp_x * offset;
        let row_center_y = center_y + perp_y * offset;

        for j in 0..num_segments {
            let t1 = (j - num_segments / 2) as f64 * segment_length;
            let t2 = t1 + segment_length;

            // Sinusoidal displacement
            let wave1 = amplitude * (t1 * frequency * 2.0 * PI).sin();
            let wave2 = amplitude * (t2 * frequency * 2.0 * PI).sin();

            let x1 = row_center_x + dir_x * t1 + perp_x * wave1;
            let y1 = row_center_y + dir_y * t1 + perp_y * wave1;
            let x2 = row_center_x + dir_x * t2 + perp_x * wave2;
            let y2 = row_center_y + dir_y * t2 + perp_y * wave2;

            lines.push(Line::new(x1, y1, x2, y2));
        }
    }

    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_wiggle_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_wiggle_fill(&poly, 10.0, 0.0, 5.0, 0.1);
        assert!(!lines.is_empty());
    }
}
