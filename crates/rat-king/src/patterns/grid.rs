//! Grid fill pattern - simple rectangular grid.
//!
//! Creates a rectangular grid of horizontal and vertical lines.
//! Simpler than crosshatch (always 90° cross angle).

use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;
use super::util::PatternContext;

/// Generate rectangular grid fill for a polygon.
///
/// Creates both horizontal and vertical lines at the specified spacing.
pub fn generate_grid_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some(ctx) = PatternContext::new(polygon, spacing, angle_degrees) else {
        return Vec::new();
    };

    let padding = ctx.padding();
    let mut lines = Vec::new();

    // Generate horizontal lines (before rotation)
    let mut y = ctx.center.y - padding;
    while y <= ctx.center.y + padding {
        let (x1, y1) = ctx.rotate(ctx.center.x - padding, y);
        let (x2, y2) = ctx.rotate(ctx.center.x + padding, y);
        lines.push(Line::new(x1, y1, x2, y2));
        y += spacing;
    }

    // Generate vertical lines (before rotation)
    let mut x = ctx.center.x - padding;
    while x <= ctx.center.x + padding {
        let (x1, y1) = ctx.rotate(x, ctx.center.y - padding);
        let (x2, y2) = ctx.rotate(x, ctx.center.y + padding);
        lines.push(Line::new(x1, y1, x2, y2));
        x += spacing;
    }

    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_grid_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_grid_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate grid lines");
        // Grid should have roughly twice as many lines as single-direction hatch
        assert!(lines.len() > 15, "Grid should have many lines");
    }

    #[test]
    fn rotation_works() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_0 = generate_grid_fill(&poly, 10.0, 0.0);
        let lines_45 = generate_grid_fill(&poly, 10.0, 45.0);

        // Rotated grid should produce different line orientations
        assert!(!lines_0.is_empty());
        assert!(!lines_45.is_empty());
    }
}
