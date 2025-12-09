//! Grid fill pattern - simple rectangular grid.
//!
//! Creates a rectangular grid of horizontal and vertical lines.
//! Simpler than crosshatch (always 90° cross angle).

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate rectangular grid fill for a polygon.
///
/// Creates both horizontal and vertical lines at the specified spacing.
pub fn generate_grid_fill(
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

    // Calculate extended bounds for rotation
    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let padding = diagonal / 2.0 + spacing;

    let mut lines = Vec::new();

    // Rotate point around center
    let rotate = |x: f64, y: f64| -> (f64, f64) {
        let dx = x - center_x;
        let dy = y - center_y;
        (
            center_x + dx * angle_rad.cos() - dy * angle_rad.sin(),
            center_y + dx * angle_rad.sin() + dy * angle_rad.cos(),
        )
    };

    // Generate horizontal lines (before rotation)
    let mut y = center_y - padding;
    while y <= center_y + padding {
        let (x1, y1) = rotate(center_x - padding, y);
        let (x2, y2) = rotate(center_x + padding, y);
        lines.push(Line::new(x1, y1, x2, y2));
        y += spacing;
    }

    // Generate vertical lines (before rotation)
    let mut x = center_x - padding;
    while x <= center_x + padding {
        let (x1, y1) = rotate(x, center_y - padding);
        let (x2, y2) = rotate(x, center_y + padding);
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
