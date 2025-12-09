//! Spiral fill patterns - Archimedean and Fermat spirals.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate Archimedean spiral fill for a polygon.
///
/// r = a * theta (constant spacing between arms)
pub fn generate_spiral_fill(
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
    // spacing between arms = 2 * pi * a
    let a = spacing / (2.0 * PI);
    let start_angle = angle_degrees * PI / 180.0;

    let max_theta = if a > 0.0 { max_radius / a } else { 0.0 };

    let mut lines = Vec::new();
    let mut theta = 0.0;
    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    while theta < max_theta {
        let r = a * theta;
        let x = center_x + r * (theta + start_angle).cos();
        let y = center_y + r * (theta + start_angle).sin();
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

        // Adaptive step size - smaller steps at larger radii
        theta += (spacing / r.max(1.0)).min(0.5);
    }

    lines
}

/// Generate Fermat (golden angle) spiral fill for a polygon.
///
/// Uses golden angle (137.5 deg) for natural-looking distribution.
pub fn generate_fermat_fill(
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

    // Golden angle
    let golden_angle = PI * (3.0 - 5.0_f64.sqrt());
    let start_angle = angle_degrees * PI / 180.0;

    // Fermat spiral: r = c * sqrt(n)
    let c = spacing / PI.sqrt();

    let mut lines = Vec::new();
    let mut n = 0u32;
    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    loop {
        let r = c * (n as f64).sqrt();
        if r > max_radius {
            break;
        }

        let theta = n as f64 * golden_angle + start_angle;
        let x = center_x + r * theta.cos();
        let y = center_y + r * theta.sin();
        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if n > 0 && prev_inside && current_inside {
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
        n += 1;
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_spiral_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_spiral_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn generates_fermat_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_fermat_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty());
    }
}
