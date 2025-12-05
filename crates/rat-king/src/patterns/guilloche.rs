//! Guilloche fill pattern - intricate spirograph-like curves.
//!
//! Guilloche patterns are created using epitrochoid and hypotrochoid equations,
//! the same mathematics behind the Spirograph toy. Used historically on currency
//! and certificates for anti-counterfeiting.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate guilloche fill for a polygon.
///
/// Creates spirograph-like patterns using hypotrochoid curves.
///
/// Parameters:
/// - `spacing`: Controls the density of the pattern
/// - `angle_degrees`: Starting rotation angle
pub fn generate_guilloche_fill(
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

    // Scale to fit polygon
    let scale = width.min(height) / 2.0 * 0.9;

    let start_angle = angle_degrees * PI / 180.0;

    let mut lines = Vec::new();

    // Generate multiple concentric guilloche rings (capped for performance)
    let num_rings = ((scale / spacing).ceil() as usize).min(5);

    for ring in 1..=num_rings {
        let ring_scale = (ring as f64 / num_rings as f64) * scale;

        // Hypotrochoid parameters
        // R = outer circle radius, r = inner circle radius, d = pen distance
        // Classic ratio for nice patterns: R/r = 5/3, d = r
        let r_ratio = 5.0 / 3.0;
        let outer_r = ring_scale;
        let inner_r = outer_r / r_ratio;
        let pen_dist = inner_r * 0.8;

        let curve_lines = generate_hypotrochoid(
            center_x, center_y,
            outer_r, inner_r, pen_dist,
            start_angle,
            polygon,
        );

        lines.extend(curve_lines);
    }

    lines
}

/// Generate a single hypotrochoid curve.
///
/// Hypotrochoid: curve traced by a point attached to a circle rolling inside another circle.
///
/// Parametric equations:
///   x(t) = (R - r) * cos(t) + d * cos((R - r) / r * t)
///   y(t) = (R - r) * sin(t) - d * sin((R - r) / r * t)
fn generate_hypotrochoid(
    center_x: f64,
    center_y: f64,
    big_r: f64,    // Outer circle radius
    small_r: f64,  // Inner circle radius
    pen_d: f64,    // Pen distance from inner circle center
    start_angle: f64,
    polygon: &Polygon,
) -> Vec<Line> {
    let mut lines = Vec::new();

    // Calculate how many rotations needed for closed curve
    // The curve closes when t = 2π * LCM(R,r) / R
    // For simplicity, we'll do enough rotations to complete the pattern
    let ratio = big_r / small_r;
    let rotations = if (ratio - ratio.round()).abs() < 0.01 {
        ratio.round() as usize
    } else {
        // For non-integer ratios, do multiple passes
        ((ratio * 10.0).round() as usize).max(5)
    };

    let max_t = 2.0 * PI * rotations as f64;
    let steps = ((max_t * 30.0) as usize).min(1000);  // Cap for performance
    let dt = max_t / steps as f64;

    let diff = big_r - small_r;
    let freq = diff / small_r;

    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for i in 0..=steps {
        let t = i as f64 * dt;

        // Hypotrochoid equations
        let x = center_x + diff * (t + start_angle).cos() + pen_d * (freq * t + start_angle).cos();
        let y = center_y + diff * (t + start_angle).sin() - pen_d * (freq * t + start_angle).sin();

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

/// Generate an epitrochoid curve (outer rolling circle).
///
/// Parametric equations:
///   x(t) = (R + r) * cos(t) - d * cos((R + r) / r * t)
///   y(t) = (R + r) * sin(t) - d * sin((R + r) / r * t)
#[allow(dead_code)]
fn generate_epitrochoid(
    center_x: f64,
    center_y: f64,
    big_r: f64,
    small_r: f64,
    pen_d: f64,
    start_angle: f64,
    polygon: &Polygon,
) -> Vec<Line> {
    let mut lines = Vec::new();

    let ratio = big_r / small_r;
    let rotations = if (ratio - ratio.round()).abs() < 0.01 {
        ratio.round() as usize
    } else {
        ((ratio * 10.0).round() as usize).max(5)
    };

    let max_t = 2.0 * PI * rotations as f64;
    let steps = (max_t * 50.0) as usize;
    let dt = max_t / steps as f64;

    let sum = big_r + small_r;
    let freq = sum / small_r;

    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for i in 0..=steps {
        let t = i as f64 * dt;

        // Epitrochoid equations
        let x = center_x + sum * (t + start_angle).cos() - pen_d * (freq * t + start_angle).cos();
        let y = center_y + sum * (t + start_angle).sin() - pen_d * (freq * t + start_angle).sin();

        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
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
    fn generates_guilloche_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_guilloche_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn guilloche_has_smooth_curves() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(200.0, 0.0),
            Point::new(200.0, 200.0),
            Point::new(0.0, 200.0),
        ]);
        let lines = generate_guilloche_fill(&poly, 20.0, 0.0);

        // Check that consecutive lines are connected (continuous curve)
        let mut connected_count = 0;
        for i in 1..lines.len() {
            let prev = &lines[i - 1];
            let curr = &lines[i];
            let dist = ((prev.x2 - curr.x1).powi(2) + (prev.y2 - curr.y1).powi(2)).sqrt();
            if dist < 1.0 {
                connected_count += 1;
            }
        }

        // Most lines should be connected (allowing for polygon clipping breaks)
        assert!(connected_count > lines.len() / 2);
    }

    #[test]
    fn different_spacing_changes_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let sparse = generate_guilloche_fill(&poly, 30.0, 0.0);
        let dense = generate_guilloche_fill(&poly, 10.0, 0.0);

        assert!(dense.len() > sparse.len());
    }
}
