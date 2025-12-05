//! Lissajous curve fill pattern.
//!
//! Lissajous curves are the family of curves described by:
//!   x(t) = A * sin(a*t + δ)
//!   y(t) = B * sin(b*t)
//!
//! Different a:b ratios create different figures (1:1 = ellipse, 1:2 = figure-8, etc.)

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate Lissajous curve fill for a polygon.
///
/// Creates oscilloscope-style Lissajous figures that scale to fit the polygon.
/// Multiple nested curves are generated based on spacing.
///
/// Parameters:
/// - `spacing`: Controls the number of nested curves
/// - `angle_degrees`: Phase shift (δ) that morphs the figure shape
pub fn generate_lissajous_fill(
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

    // Scale to fit polygon with some margin
    let scale_x = width / 2.0 * 0.9;
    let scale_y = height / 2.0 * 0.9;

    let phase = angle_degrees * PI / 180.0;

    let mut lines = Vec::new();

    // Generate multiple nested Lissajous curves (capped for performance)
    let num_curves = ((scale_x.min(scale_y) / spacing).ceil() as usize).min(8);

    // Classic Lissajous ratios that create interesting patterns
    let ratios: [(f64, f64); 4] = [
        (3.0, 2.0),  // Classic 3:2
        (5.0, 4.0),  // More complex
        (3.0, 4.0),  // Another classic
        (5.0, 6.0),  // Dense pattern
    ];

    for curve_idx in 1..=num_curves {
        let t_scale = curve_idx as f64 / num_curves as f64;
        let ax = scale_x * t_scale;
        let ay = scale_y * t_scale;

        // Pick ratio based on curve index for variety
        let (a, b) = ratios[curve_idx % ratios.len()];

        let curve_lines = generate_single_lissajous(
            center_x, center_y,
            ax, ay,
            a, b,
            phase,
            polygon,
        );

        lines.extend(curve_lines);
    }

    lines
}

/// Generate a single Lissajous curve.
fn generate_single_lissajous(
    center_x: f64,
    center_y: f64,
    amp_x: f64,
    amp_y: f64,
    freq_a: f64,
    freq_b: f64,
    phase: f64,
    polygon: &Polygon,
) -> Vec<Line> {
    let mut lines = Vec::new();

    // Calculate period - curve repeats when both sin functions complete
    // LCM of periods, but we'll just do enough cycles
    let max_t = 2.0 * PI * freq_a.max(freq_b);
    let steps = ((max_t * 30.0) as usize).min(500);  // Cap steps for performance
    let dt = max_t / steps as f64;

    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for i in 0..=steps {
        let t = i as f64 * dt;

        // Lissajous equations
        let x = center_x + amp_x * (freq_a * t + phase).sin();
        let y = center_y + amp_y * (freq_b * t).sin();

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
    fn generates_lissajous_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_lissajous_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn phase_changes_pattern() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_0 = generate_lissajous_fill(&poly, 20.0, 0.0);
        let lines_90 = generate_lissajous_fill(&poly, 20.0, 90.0);

        // Both phases should produce lines
        assert!(!lines_0.is_empty());
        assert!(!lines_90.is_empty());

        // Lines should differ in position (check a sample point in a later line)
        if lines_0.len() > 10 && lines_90.len() > 10 {
            let diff_x = (lines_0[10].x1 - lines_90[10].x1).abs();
            let diff_y = (lines_0[10].y1 - lines_90[10].y1).abs();
            assert!(diff_x > 0.1 || diff_y > 0.1, "Phase should affect line positions");
        }
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let sparse = generate_lissajous_fill(&poly, 30.0, 0.0);
        let dense = generate_lissajous_fill(&poly, 10.0, 0.0);

        assert!(dense.len() > sparse.len());
    }
}
