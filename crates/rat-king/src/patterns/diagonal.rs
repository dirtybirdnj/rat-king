//! Diagonal line fill pattern.
//!
//! Simple diagonal lines at a specified angle - essentially the same as lines
//! but with a default 45-degree angle and specific diagonal behavior.

use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;
use super::util::{PatternContext, LineDirection};

/// Generate diagonal line fill for a polygon.
///
/// Creates parallel diagonal lines across the polygon.
/// Similar to lines fill but optimized for diagonal presentation.
pub fn generate_diagonal_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // Default to 45 degrees if angle is 0
    let effective_angle = if angle_degrees == 0.0 { 45.0 } else { angle_degrees };

    let Some(ctx) = PatternContext::new(polygon, spacing, effective_angle) else {
        return Vec::new();
    };

    let dir = LineDirection::from_degrees(effective_angle);
    let lines = dir.generate_parallel_lines(ctx.center, spacing, ctx.line_count(), ctx.diagonal);

    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

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
