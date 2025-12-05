//! Phyllotaxis (sunflower spiral) fill pattern.
//!
//! Phyllotaxis describes the arrangement of leaves, seeds, or florets in plants.
//! The classic sunflower pattern uses the golden angle (137.5°) to achieve
//! optimal packing, creating visible Fibonacci spiral arms (parastichies).
//!
//! Algorithm:
//!   for i in 0..N:
//!     angle = i * GOLDEN_ANGLE
//!     radius = sqrt(i) * scale
//!     place_point(x, y)

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// The golden angle in radians: 360° / φ² ≈ 137.507764°
const GOLDEN_ANGLE: f64 = 2.39996322972865332; // 137.5077... degrees in radians

/// Generate phyllotaxis (sunflower) fill for a polygon.
///
/// Creates a spiral pattern using the golden angle, producing visible
/// Fibonacci spiral arms. Points are connected to create continuous lines.
///
/// Parameters:
/// - `spacing`: Controls point density (smaller = more points)
/// - `angle_degrees`: Rotation offset for the entire pattern
pub fn generate_phyllotaxis_fill(
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

    let max_radius = width.min(height) / 2.0 * 0.95;
    let rotation = angle_degrees * PI / 180.0;

    // Calculate how many points we need to fill to max_radius
    // radius = sqrt(i) * scale, so i = (radius/scale)²
    let scale = spacing / 2.0;
    let max_i = ((max_radius / scale).powi(2)) as usize;

    let mut lines = Vec::new();

    // Generate points in phyllotaxis pattern
    let points: Vec<(f64, f64, bool)> = (0..max_i)
        .map(|i| {
            let angle = i as f64 * GOLDEN_ANGLE + rotation;
            let radius = (i as f64).sqrt() * scale;
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            let inside = point_in_polygon(x, y, &polygon.outer);
            (x, y, inside)
        })
        .collect();

    // Connect points in spiral order
    for i in 1..points.len() {
        let (x1, y1, inside1) = points[i - 1];
        let (x2, y2, inside2) = points[i];

        if inside1 && inside2 {
            // Check holes
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0;
            let in_hole = polygon.holes.iter().any(|hole| {
                point_in_polygon(mid_x, mid_y, hole)
            });
            if !in_hole {
                lines.push(Line::new(x1, y1, x2, y2));
            }
        }
    }

    // Also connect along Fibonacci spiral arms for additional structure
    // The visible spirals occur at Fibonacci number intervals
    let fibonacci = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89];

    for &fib in &fibonacci {
        if fib >= points.len() {
            break;
        }

        for i in fib..points.len() {
            let (x1, y1, inside1) = points[i - fib];
            let (x2, y2, inside2) = points[i];

            if inside1 && inside2 {
                let mid_x = (x1 + x2) / 2.0;
                let mid_y = (y1 + y2) / 2.0;
                let in_hole = polygon.holes.iter().any(|hole| {
                    point_in_polygon(mid_x, mid_y, hole)
                });
                if !in_hole {
                    // Only add if the connection is short enough (local neighbors)
                    let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                    if dist < spacing * 3.0 {
                        lines.push(Line::new(x1, y1, x2, y2));
                    }
                }
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_phyllotaxis_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_phyllotaxis_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn rotation_shifts_pattern() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_0 = generate_phyllotaxis_fill(&poly, 10.0, 0.0);
        let lines_90 = generate_phyllotaxis_fill(&poly, 10.0, 90.0);

        // Both should produce lines
        assert!(!lines_0.is_empty());
        assert!(!lines_90.is_empty());

        // Lines later in the pattern should differ due to rotation
        // (early lines are near center where rotation has less visible effect)
        if lines_0.len() > 50 && lines_90.len() > 50 {
            let idx = lines_0.len() / 2;  // Check middle of pattern
            let diff_x = (lines_0[idx].x1 - lines_90[idx].x1).abs();
            let diff_y = (lines_0[idx].y1 - lines_90[idx].y1).abs();
            assert!(diff_x > 0.1 || diff_y > 0.1, "Rotation should affect line positions");
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

        let sparse = generate_phyllotaxis_fill(&poly, 15.0, 0.0);
        let dense = generate_phyllotaxis_fill(&poly, 5.0, 0.0);

        assert!(dense.len() > sparse.len());
    }

    #[test]
    fn golden_angle_is_correct() {
        // Golden angle should be approximately 137.5 degrees
        let degrees = GOLDEN_ANGLE * 180.0 / PI;
        assert!((degrees - 137.5).abs() < 0.1);
    }
}
