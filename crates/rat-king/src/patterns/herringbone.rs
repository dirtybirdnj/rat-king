//! Herringbone fill pattern.
//!
//! Creates a classic herringbone (chevron) pattern with alternating
//! diagonal lines that create a V-shaped or zigzag arrangement.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate herringbone fill pattern for a polygon.
///
/// Creates alternating diagonal lines in a chevron/V pattern.
/// The angle parameter controls the base angle of the pattern.
pub fn generate_herringbone_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let angle_rad = angle_degrees * PI / 180.0;

    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();

    // Herringbone uses two alternating angles (typically +45 and -45 from base)
    let herring_angle = 45.0 * PI / 180.0;
    let angle1 = angle_rad + herring_angle;
    let angle2 = angle_rad - herring_angle;

    let mut lines = Vec::new();

    // Row spacing for herringbone pattern
    let row_spacing = spacing * 2.0;
    let segment_length = spacing * 3.0;

    let num_rows = (diagonal / row_spacing).ceil() as i32;
    let num_cols = (diagonal / segment_length).ceil() as i32;

    for row in -num_rows..=num_rows {
        let y_base = center_y + (row as f64 * row_spacing);

        for col in -num_cols..=num_cols {
            let x_base = center_x + (col as f64 * segment_length);

            // Alternate angle based on column
            let angle = if (row + col) % 2 == 0 { angle1 } else { angle2 };

            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Create short diagonal segment
            let half_len = segment_length / 2.0;
            let x1 = x_base - cos_a * half_len;
            let y1 = y_base - sin_a * half_len;
            let x2 = x_base + cos_a * half_len;
            let y2 = y_base + sin_a * half_len;

            lines.push(Line::new(x1, y1, x2, y2));
        }
    }

    // Rotate all lines around center
    let rotated_lines: Vec<Line> = lines.iter().map(|line| {
        let rotate_point = |x: f64, y: f64| -> (f64, f64) {
            let dx = x - center_x;
            let dy = y - center_y;
            (
                center_x + dx * angle_rad.cos() - dy * angle_rad.sin(),
                center_y + dx * angle_rad.sin() + dy * angle_rad.cos(),
            )
        };
        let (x1, y1) = rotate_point(line.x1, line.y1);
        let (x2, y2) = rotate_point(line.x2, line.y2);
        Line::new(x1, y1, x2, y2)
    }).collect();

    clip_lines_to_polygon(&rotated_lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_herringbone_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_herringbone_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate herringbone lines");
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_dense = generate_herringbone_fill(&poly, 5.0, 0.0);
        let lines_sparse = generate_herringbone_fill(&poly, 15.0, 0.0);

        assert!(lines_dense.len() > lines_sparse.len(),
            "Smaller spacing should produce more lines");
    }

    #[test]
    fn rotation_works() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_0 = generate_herringbone_fill(&poly, 10.0, 0.0);
        let lines_45 = generate_herringbone_fill(&poly, 10.0, 45.0);

        assert!(!lines_0.is_empty());
        assert!(!lines_45.is_empty());
    }
}
