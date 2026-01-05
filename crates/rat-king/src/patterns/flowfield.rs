//! Flow field fill pattern - lines following Perlin-like noise gradients.
//!
//! Creates organic, flowing lines that follow a procedural noise field,
//! similar to wind patterns or topographic contours.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;
use crate::rng::Rng;

/// Generate flow field fill for a polygon.
///
/// Creates streamlines that follow a procedural noise field.
/// - `spacing`: Distance between seed points
/// - `angle_degrees`: Base rotation applied to the entire field
pub fn generate_flowfield_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let base_angle = angle_degrees * PI / 180.0;

    // Scale for noise sampling
    let noise_scale = 0.02 / spacing.max(1.0) * 10.0;

    let mut lines = Vec::new();
    let mut rng = Rng::new(42);

    // Grid of seed points
    let cols = ((width / spacing).ceil() as i32).max(1);
    let rows = ((height / spacing).ceil() as i32).max(1);

    for row in 0..=rows {
        for col in 0..=cols {
            // Offset every other row for better coverage
            let x_offset = if row % 2 == 0 { 0.0 } else { spacing * 0.5 };
            let start_x = min_x + col as f64 * spacing + x_offset;
            let start_y = min_y + row as f64 * spacing;

            // Add slight randomness to seed positions
            let jitter = spacing * 0.2;
            let seed_x = start_x + (rng.next_f64() - 0.5) * jitter;
            let seed_y = start_y + (rng.next_f64() - 0.5) * jitter;

            if !point_in_polygon(seed_x, seed_y, &polygon.outer) {
                continue;
            }
            if polygon.holes.iter().any(|h| point_in_polygon(seed_x, seed_y, h)) {
                continue;
            }

            // Trace streamline from this seed point
            let streamline = trace_streamline(
                seed_x, seed_y,
                polygon,
                spacing,
                noise_scale,
                base_angle,
            );

            lines.extend(streamline);
        }
    }

    lines
}

/// Trace a single streamline from a seed point.
fn trace_streamline(
    start_x: f64,
    start_y: f64,
    polygon: &Polygon,
    spacing: f64,
    noise_scale: f64,
    base_angle: f64,
) -> Vec<Line> {
    let mut lines = Vec::new();
    let step_size = spacing * 0.5;
    let max_steps = 50; // Reduced for cleaner preview

    // Trace in both directions from seed
    for direction in [-1.0, 1.0] {
        let mut x = start_x;
        let mut y = start_y;
        let mut prev_x = x;
        let mut prev_y = y;

        for _ in 0..max_steps {
            // Get angle from noise field
            let angle = noise_angle(x, y, noise_scale) + base_angle;

            // Step in flow direction
            let dx = angle.cos() * step_size * direction;
            let dy = angle.sin() * step_size * direction;

            let new_x = x + dx;
            let new_y = y + dy;

            // Check if still inside polygon
            if !point_in_polygon(new_x, new_y, &polygon.outer) {
                break;
            }
            if polygon.holes.iter().any(|h| point_in_polygon(new_x, new_y, h)) {
                break;
            }

            // Add line segment
            if (prev_x - new_x).abs() > 0.01 || (prev_y - new_y).abs() > 0.01 {
                lines.push(Line::new(prev_x, prev_y, new_x, new_y));
            }

            prev_x = new_x;
            prev_y = new_y;
            x = new_x;
            y = new_y;
        }
    }

    lines
}

/// Get flow angle from noise field at a point.
/// Uses simple value noise for smooth gradients.
fn noise_angle(x: f64, y: f64, scale: f64) -> f64 {
    let nx = x * scale;
    let ny = y * scale;

    // Simple value noise using sine functions (faster than Perlin)
    let n1 = (nx * 1.0).sin() * (ny * 1.0).cos();
    let n2 = (nx * 2.3 + 1.7).sin() * (ny * 2.1 + 0.9).cos();
    let n3 = (nx * 0.7 + ny * 0.5).sin();

    let combined = n1 * 0.5 + n2 * 0.3 + n3 * 0.2;

    // Map noise to angle (full circle)
    combined * PI
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
    fn generates_flowfield_lines() {
        let poly = square_polygon();
        let lines = generate_flowfield_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn flowfield_with_angle() {
        let poly = square_polygon();
        let lines = generate_flowfield_fill(&poly, 10.0, 45.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn flowfield_different_spacing() {
        let poly = square_polygon();
        let lines_sparse = generate_flowfield_fill(&poly, 20.0, 0.0);
        let lines_dense = generate_flowfield_fill(&poly, 5.0, 0.0);
        // Denser spacing should generally produce more lines
        assert!(lines_dense.len() > lines_sparse.len());
    }
}
