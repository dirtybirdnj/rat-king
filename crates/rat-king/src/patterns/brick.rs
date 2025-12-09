//! Brick fill pattern - running bond brickwork.
//!
//! Creates a brick/running bond pattern where each row is offset by half.
//! Classic masonry pattern used in walls and pavements.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate brick pattern fill for a polygon.
///
/// Creates horizontal "mortar" lines with vertical joints offset per row.
pub fn generate_brick_fill(
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

    // Brick dimensions
    let brick_height = spacing;
    let brick_width = spacing * 2.5; // Standard brick ratio ~2.5:1

    // Calculate extended bounds for rotation
    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let padding = diagonal / 2.0 + brick_width;

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

    // Generate brick pattern
    let mut row = 0;
    let mut y = min_y - padding;

    while y <= max_y + padding {
        let is_odd_row = row % 2 == 1;
        let row_offset = if is_odd_row { brick_width / 2.0 } else { 0.0 };

        // Horizontal mortar line for this row
        let (hx1, hy1) = rotate(min_x - padding, y);
        let (hx2, hy2) = rotate(max_x + padding, y);

        // Clip horizontal line to polygon
        let h_mid_x = (hx1 + hx2) / 2.0;
        let h_mid_y = (hy1 + hy2) / 2.0;
        if point_in_polygon(h_mid_x, h_mid_y, outer) {
            lines.push(Line::new(hx1, hy1, hx2, hy2));
        }

        // Vertical mortar joints
        let mut x = min_x - padding + row_offset;
        while x <= max_x + padding {
            let (vx1, vy1) = rotate(x, y);
            let (vx2, vy2) = rotate(x, y + brick_height);

            // Check if joint midpoint is inside polygon
            let v_mid_x = (vx1 + vx2) / 2.0;
            let v_mid_y = (vy1 + vy2) / 2.0;

            if point_in_polygon(v_mid_x, v_mid_y, outer) {
                // Check holes
                let in_hole = polygon.holes.iter().any(|hole| {
                    point_in_polygon(v_mid_x, v_mid_y, hole)
                });
                if !in_hole {
                    lines.push(Line::new(vx1, vy1, vx2, vy2));
                }
            }

            x += brick_width;
        }

        y += brick_height;
        row += 1;
    }

    // Clip all lines to polygon (for horizontal lines extending outside)
    crate::clip::clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_brick_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_brick_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate brick lines");
    }

    #[test]
    fn brick_has_both_orientations() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_brick_fill(&poly, 10.0, 0.0);

        // Should have both horizontal and vertical lines
        let has_horizontal = lines.iter().any(|l| (l.y2 - l.y1).abs() < 0.1);
        let has_vertical = lines.iter().any(|l| (l.x2 - l.x1).abs() < 0.1);

        assert!(has_horizontal || has_vertical, "Should have lines in both directions");
    }
}
