//! Herringbone fill pattern.
//!
//! Creates a classic herringbone (chevron) pattern with alternating
//! diagonal lines that create a V-shaped or zigzag arrangement.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;
use super::util::{PatternContext, RotationTransform};

/// Generate herringbone fill pattern for a polygon.
///
/// Creates alternating diagonal lines in a chevron/V pattern.
/// The angle parameter controls the base angle of the pattern.
pub fn generate_herringbone_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some(ctx) = PatternContext::new(polygon, spacing, angle_degrees) else {
        return Vec::new();
    };

    // Herringbone uses two alternating angles (typically +45 and -45 from base)
    let herring_angle = 45.0 * PI / 180.0;
    let angle1 = ctx.angle_rad + herring_angle;
    let angle2 = ctx.angle_rad - herring_angle;

    let mut lines = Vec::new();

    // Row spacing for herringbone pattern
    let row_spacing = spacing * 2.0;
    let segment_length = spacing * 3.0;

    let num_rows = (ctx.diagonal / row_spacing).ceil() as i32;
    let num_cols = (ctx.diagonal / segment_length).ceil() as i32;

    for row in -num_rows..=num_rows {
        let y_base = ctx.center.y + (row as f64 * row_spacing);

        for col in -num_cols..=num_cols {
            let x_base = ctx.center.x + (col as f64 * segment_length);

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
    let rot = RotationTransform::new(ctx.center.x, ctx.center.y, ctx.angle_rad);
    let rotated_lines: Vec<Line> = lines.iter().map(|line| rot.apply_line(line)).collect();

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
