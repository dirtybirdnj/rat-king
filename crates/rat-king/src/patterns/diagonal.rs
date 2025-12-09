//! Diagonal line fill pattern.
//!
//! Simple diagonal lines at a specified angle - essentially the same as lines
//! but with a default 45-degree angle and specific diagonal behavior.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate diagonal line fill for a polygon.
///
/// Creates parallel diagonal lines across the polygon.
/// Similar to lines fill but optimized for diagonal presentation.
pub fn generate_diagonal_fill(
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

    // Default to 45 degrees if angle is 0
    let angle_rad = if angle_degrees == 0.0 {
        45.0 * PI / 180.0
    } else {
        angle_degrees * PI / 180.0
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();

    // Generate parallel lines perpendicular to the angle
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Direction along the lines
    let dx = cos_a;
    let dy = sin_a;

    // Direction perpendicular to lines (for spacing)
    let px = -sin_a;
    let py = cos_a;

    let mut lines = Vec::new();
    let num_lines = (diagonal / spacing).ceil() as i32;

    for i in -num_lines..=num_lines {
        let offset = i as f64 * spacing;

        // Start and end points of line through center, offset perpendicular
        let cx = center_x + px * offset;
        let cy = center_y + py * offset;

        // Extend line in both directions
        let x1 = cx - dx * diagonal;
        let y1 = cy - dy * diagonal;
        let x2 = cx + dx * diagonal;
        let y2 = cy + dy * diagonal;

        lines.push(Line::new(x1, y1, x2, y2));
    }

    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_diagonal_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_diagonal_fill(&poly, 10.0, 45.0);
        assert!(!lines.is_empty(), "Should generate diagonal lines");
    }

    #[test]
    fn default_angle_is_45() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_default = generate_diagonal_fill(&poly, 10.0, 0.0);
        let lines_45 = generate_diagonal_fill(&poly, 10.0, 45.0);

        // Should produce similar results
        assert!(!lines_default.is_empty());
        assert!(!lines_45.is_empty());
    }

    #[test]
    fn spacing_affects_count() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_dense = generate_diagonal_fill(&poly, 5.0, 45.0);
        let lines_sparse = generate_diagonal_fill(&poly, 20.0, 45.0);

        assert!(lines_dense.len() > lines_sparse.len(),
            "Smaller spacing should produce more lines");
    }
}
