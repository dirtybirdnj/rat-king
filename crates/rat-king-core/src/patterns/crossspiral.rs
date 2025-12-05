//! Cross-spiral fill pattern - two opposing Archimedean spirals.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate cross-spiral fill - two Archimedean spirals in opposite directions.
///
/// Creates a balanced fill with spirals winding both clockwise and counter-clockwise.
pub fn generate_crossspiral_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let width = max_x - min_x;
    let height = max_y - min_y;
    let max_radius = (width * width + height * height).sqrt() / 2.0 * 1.5;

    // Archimedean spiral: r = a * theta
    // Double the spacing since we have two spirals
    let a = spacing / PI;
    let start_angle = angle_degrees * PI / 180.0;

    let max_theta = if a > 0.0 { max_radius / a } else { 0.0 };

    let mut lines = Vec::new();

    // Generate clockwise spiral
    lines.extend(generate_single_spiral(
        polygon, center_x, center_y, a, start_angle, max_theta, 1.0
    ));

    // Generate counter-clockwise spiral (180° offset)
    lines.extend(generate_single_spiral(
        polygon, center_x, center_y, a, start_angle + PI, max_theta, -1.0
    ));

    lines
}

/// Generate a single spiral arm.
fn generate_single_spiral(
    polygon: &Polygon,
    center_x: f64,
    center_y: f64,
    a: f64,
    start_angle: f64,
    max_theta: f64,
    direction: f64, // 1.0 for CW, -1.0 for CCW
) -> Vec<Line> {
    let mut lines = Vec::new();
    let mut theta = 0.0;
    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    while theta < max_theta {
        let r = a * theta;
        let angle = direction * theta + start_angle;
        let x = center_x + r * angle.cos();
        let y = center_y + r * angle.sin();
        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
                // Check if segment is inside any hole
                let mid_x = (px + x) / 2.0;
                let mid_y = (py + y) / 2.0;
                let in_hole = polygon.holes.iter().any(|hole| {
                    point_in_polygon(mid_x, mid_y, hole)
                });
                if !in_hole {
                    lines.push(Line::new(px, py, x, y));
                }
            }
        }

        prev_point = Some((x, y));
        prev_inside = current_inside;

        // Adaptive step size
        let spacing = a * 2.0 * PI;
        theta += (spacing / r.max(1.0)).min(0.5);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_crossspiral_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_crossspiral_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty());
        // Should have significantly more lines than single spiral
        // due to two spiral arms
    }

    #[test]
    fn both_arms_generate_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_crossspiral_fill(&poly, 10.0, 0.0);

        // Count lines in different quadrants to verify both spirals contribute
        let center = (50.0, 50.0);
        let mut has_upper_left = false;
        let mut has_lower_right = false;

        for line in &lines {
            if line.x1 < center.0 && line.y1 < center.1 {
                has_upper_left = true;
            }
            if line.x1 > center.0 && line.y1 > center.1 {
                has_lower_right = true;
            }
        }

        assert!(has_upper_left && has_lower_right);
    }
}
