//! Zigzag fill pattern - parallel zigzag lines.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate zigzag fill for a polygon.
///
/// Creates parallel rows of sharp zigzag lines.
pub fn generate_zigzag_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    amplitude: f64,
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

    // Zigzag wavelength - distance between peaks
    let wavelength = amplitude * 2.0;
    let num_segments = (diagonal / wavelength) as i32 + 2;

    let mut lines = Vec::new();

    for i in -num_rows..=num_rows {
        let offset = i as f64 * spacing;
        let row_center_x = center_x + perp_x * offset;
        let row_center_y = center_y + perp_y * offset;

        for j in -num_segments..num_segments {
            let t1 = j as f64 * wavelength;
            let t2 = (j as f64 + 0.5) * wavelength;

            // Alternate amplitude direction
            let amp1 = amplitude * if j % 2 == 0 { 1.0 } else { -1.0 };
            let amp2 = -amp1;

            let x1 = row_center_x + dir_x * t1 + perp_x * amp1;
            let y1 = row_center_y + dir_y * t1 + perp_y * amp1;
            let x2 = row_center_x + dir_x * t2 + perp_x * amp2;
            let y2 = row_center_y + dir_y * t2 + perp_y * amp2;

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
    fn generates_zigzag_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 5.0);
        assert!(!lines.is_empty());
    }
}
