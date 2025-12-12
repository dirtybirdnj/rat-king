//! Gosper curve fill pattern - flowsnake space-filling curve.
//!
//! The Gosper curve (also known as the flowsnake or Peano-Gosper curve)
//! is a space-filling curve based on a hexagonal grid. It tiles the plane
//! in a self-similar fractal pattern.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate Gosper curve fill for a polygon.
///
/// Creates a space-filling Gosper curve pattern.
/// - `spacing`: Controls the overall scale/detail level
/// - `angle_degrees`: Rotation applied to the pattern
pub fn generate_gosper_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let angle_rad = angle_degrees * PI / 180.0;

    // Determine recursion depth based on spacing
    // Each recursion level scales by sqrt(7) ≈ 2.646
    let scale_factor: f64 = 2.6457513; // sqrt(7)
    let base_size = spacing * 2.0;
    let depth = (((diagonal / base_size).ln() / scale_factor.ln()).floor() as i32)
        .clamp(1, 5);

    // Generate the Gosper curve points
    let mut points = gosper_curve(depth);

    if points.is_empty() {
        return Vec::new();
    }

    // Scale and position the curve to fit the polygon
    let (curve_min_x, curve_min_y, curve_max_x, curve_max_y) = curve_bounds(&points);
    let curve_width = curve_max_x - curve_min_x;
    let curve_height = curve_max_y - curve_min_y;

    if curve_width < 0.001 || curve_height < 0.001 {
        return Vec::new();
    }

    // Scale to fit diagonal with some overlap
    let scale = diagonal * 1.2 / curve_width.max(curve_height);
    let curve_center_x = (curve_min_x + curve_max_x) / 2.0;
    let curve_center_y = (curve_min_y + curve_max_y) / 2.0;

    // Transform points: scale, rotate, translate
    for point in &mut points {
        // Center and scale
        let x = (point.0 - curve_center_x) * scale;
        let y = (point.1 - curve_center_y) * scale;

        // Rotate
        let rx = x * angle_rad.cos() - y * angle_rad.sin();
        let ry = x * angle_rad.sin() + y * angle_rad.cos();

        // Translate to polygon center
        point.0 = rx + center_x;
        point.1 = ry + center_y;
    }

    // Convert points to lines
    let mut all_lines = Vec::new();
    for i in 0..points.len() - 1 {
        all_lines.push(Line::new(
            points[i].0, points[i].1,
            points[i + 1].0, points[i + 1].1,
        ));
    }

    // Clip to polygon
    clip_lines_to_polygon(&all_lines, polygon)
}

/// Generate Gosper curve points using L-system.
///
/// Gosper curve L-system:
/// - Axiom: A
/// - Rules: A -> A-B--B+A++AA+B-, B -> +A-BB--B-A++A+B
/// - Angle: 60 degrees
fn gosper_curve(depth: i32) -> Vec<(f64, f64)> {
    // Generate the L-system string
    let mut current = String::from("A");

    for _ in 0..depth {
        let mut next = String::new();
        for c in current.chars() {
            match c {
                'A' => next.push_str("A-B--B+A++AA+B-"),
                'B' => next.push_str("+A-BB--B-A++A+B"),
                _ => next.push(c),
            }
        }
        current = next;
    }

    // Convert L-system string to points
    let mut points = Vec::new();
    let mut x = 0.0;
    let mut y = 0.0;
    let mut angle = 0.0_f64;
    let turn_angle = PI / 3.0; // 60 degrees
    let step = 1.0;

    points.push((x, y));

    for c in current.chars() {
        match c {
            'A' | 'B' => {
                // Move forward
                x += angle.cos() * step;
                y += angle.sin() * step;
                points.push((x, y));
            }
            '+' => {
                // Turn left
                angle += turn_angle;
            }
            '-' => {
                // Turn right
                angle -= turn_angle;
            }
            _ => {}
        }
    }

    points
}

/// Get bounding box of curve points.
fn curve_bounds(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for &(x, y) in points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    (min_x, min_y, max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn square_polygon() -> Polygon {
        Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ])
    }

    #[test]
    fn generates_gosper_lines() {
        let poly = square_polygon();
        let lines = generate_gosper_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn gosper_with_rotation() {
        let poly = square_polygon();
        let lines = generate_gosper_fill(&poly, 10.0, 30.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn gosper_curve_generates_points() {
        let points = gosper_curve(1);
        assert!(points.len() > 1);

        let points2 = gosper_curve(2);
        // Higher depth = more points
        assert!(points2.len() > points.len());
    }
}
