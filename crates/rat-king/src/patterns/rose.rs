//! Rose curve (Rhodonea) fill pattern.
//!
//! Rose curves are defined in polar coordinates as:
//!   r = cos(k * θ)  or  r = sin(k * θ)
//!
//! When k is an integer:
//! - k odd: k petals
//! - k even: 2k petals
//!
//! When k is rational (p/q): more complex multi-lobed figures

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate rose curve fill for a polygon.
///
/// Creates flower-like petal patterns. The number of petals depends on
/// the k parameter derived from angle.
///
/// Parameters:
/// - `spacing`: Controls the number of nested rose curves
/// - `angle_degrees`: Controls the k parameter (petals). 0-360 maps to k values.
pub fn generate_rose_fill(
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

    let max_radius = width.min(height) / 2.0 * 0.9;

    // Map angle to k value (number of petals factor)
    // angle 0-60 → k=2 (4 petals), 60-120 → k=3 (3 petals), etc.
    let k = match (angle_degrees as i32 / 60) % 6 {
        0 => 2.0,
        1 => 3.0,
        2 => 4.0,
        3 => 5.0,
        4 => 6.0,
        _ => 7.0,
    };

    // Rotation offset from angle remainder
    let rotation = (angle_degrees % 60.0) * PI / 180.0;

    let mut lines = Vec::new();

    // Generate multiple nested rose curves (capped for performance)
    let num_curves = ((max_radius / spacing).ceil() as usize).min(10);

    for curve_idx in 1..=num_curves {
        let radius = (curve_idx as f64 / num_curves as f64) * max_radius;

        let curve_lines = generate_single_rose(
            center_x, center_y,
            radius,
            k,
            rotation,
            polygon,
        );

        lines.extend(curve_lines);
    }

    lines
}

/// Generate a single rose curve.
fn generate_single_rose(
    center_x: f64,
    center_y: f64,
    max_radius: f64,
    k: f64,
    rotation: f64,
    polygon: &Polygon,
) -> Vec<Line> {
    let mut lines = Vec::new();

    // For k integer, curve closes at θ = π (k odd) or θ = 2π (k even)
    // For safety, always do full 2π
    let max_theta = 2.0 * PI;
    let steps = ((max_theta * 50.0) as usize).min(400);  // Cap for performance
    let dtheta = max_theta / steps as f64;

    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for i in 0..=steps {
        let theta = i as f64 * dtheta;

        // Rose curve: r = cos(k * θ)
        let r = max_radius * (k * theta).cos().abs(); // abs to always have positive radius
        let x = center_x + r * (theta + rotation).cos();
        let y = center_y + r * (theta + rotation).sin();

        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
                // Check holes
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
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_rose_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_rose_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn different_k_values() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        // Different angle ranges should give different k values
        let lines_k2 = generate_rose_fill(&poly, 20.0, 30.0);  // k=2 (4 petals)
        let lines_k3 = generate_rose_fill(&poly, 20.0, 90.0);  // k=3 (3 petals)

        // Both should generate lines
        assert!(!lines_k2.is_empty());
        assert!(!lines_k3.is_empty());
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let sparse = generate_rose_fill(&poly, 30.0, 0.0);
        let dense = generate_rose_fill(&poly, 10.0, 0.0);

        assert!(dense.len() > sparse.len());
    }
}
