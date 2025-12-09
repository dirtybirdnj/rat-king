//! Scribble fill pattern - organic random walk lines.
//!
//! Creates hand-drawn looking scribble fills using:
//! - Random walk with momentum for smooth direction changes
//! - Perlin-like noise for organic movement
//! - Boundary awareness to curve back inside the polygon

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;
use crate::rng::Rng;

/// Generate organic scribble fill for a polygon.
///
/// Creates natural-looking scribble patterns using random walks with momentum.
/// The pattern curves smoothly and avoids sharp mechanical turns.
///
/// Parameters:
/// - `spacing`: Controls density (lower = denser scribble)
/// - `angle_degrees`: Seed for randomness (different angles = different scribbles)
pub fn generate_scribble_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Use angle as seed for deterministic but varied results
    let seed = (angle_degrees * 1000.0) as u64;
    let mut rng = Rng::new(seed);

    let mut lines = Vec::new();

    // Calculate coverage parameters
    let step_size = spacing * 0.5;  // Small steps for smooth curves
    let area = width * height;
    let target_length = area / spacing;  // Total line length to draw
    let num_strokes = ((target_length / (width.max(height) * 2.0)) as usize).max(3);

    for stroke in 0..num_strokes {
        // Start each stroke at a random position inside the polygon
        let mut x: f64;
        let mut y: f64;
        let mut attempts = 0;

        loop {
            x = min_x + rng.next_f64() * width;
            y = min_y + rng.next_f64() * height;
            if point_in_polygon(x, y, &polygon.outer) {
                let in_hole = polygon.holes.iter().any(|h| point_in_polygon(x, y, h));
                if !in_hole {
                    break;
                }
            }
            attempts += 1;
            if attempts > 100 {
                // Fall back to center
                x = center_x;
                y = center_y;
                break;
            }
        }

        // Initial direction with some randomness
        let base_angle = (stroke as f64 / num_strokes as f64) * 2.0 * PI;
        let mut angle = base_angle + rng.next_signed() * PI * 0.5;
        let mut momentum_angle = angle;

        // Walk parameters
        let max_steps = ((width.max(height) * 4.0) / step_size) as usize;
        let momentum = 0.85;  // How much previous direction influences next
        let wiggle = 0.4;     // Random direction change amount

        for _ in 0..max_steps {
            // Calculate next position
            let nx = x + angle.cos() * step_size;
            let ny = y + angle.sin() * step_size;

            let next_inside = point_in_polygon(nx, ny, &polygon.outer);
            let next_in_hole = polygon.holes.iter().any(|h| point_in_polygon(nx, ny, h));

            if next_inside && !next_in_hole {
                // Valid move - add line segment
                lines.push(Line::new(x, y, nx, ny));
                x = nx;
                y = ny;

                // Update direction with momentum and randomness
                let random_turn = rng.next_signed() * wiggle;
                momentum_angle = momentum_angle * momentum + angle * (1.0 - momentum);
                angle = momentum_angle + random_turn;
            } else {
                // Hit boundary - turn away
                // Find direction back toward center
                let to_center = (center_y - y).atan2(center_x - x);

                // Blend toward center with some randomness
                angle = to_center + rng.next_signed() * PI * 0.5;
                momentum_angle = angle;
            }
        }
    }

    // Add some connecting loops for more organic feel
    add_organic_loops(&mut lines, polygon, spacing, &mut rng);

    lines
}

/// Add small organic loop patterns for visual interest.
fn add_organic_loops(
    lines: &mut Vec<Line>,
    polygon: &Polygon,
    spacing: f64,
    rng: &mut Rng,
) {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return;
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let num_loops = ((width * height) / (spacing * spacing * 100.0)) as usize;

    for _ in 0..num_loops {
        // Find a random starting point
        let cx = min_x + rng.next_f64() * width;
        let cy = min_y + rng.next_f64() * height;

        if !point_in_polygon(cx, cy, &polygon.outer) {
            continue;
        }
        if polygon.holes.iter().any(|h| point_in_polygon(cx, cy, h)) {
            continue;
        }

        // Draw a small spiral loop
        let loop_radius = spacing * (1.0 + rng.next_f64() * 2.0);
        let loop_turns = 1.0 + rng.next_f64() * 2.0;
        let steps = 20;

        let mut prev_x = cx;
        let mut prev_y = cy;

        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let theta = t * loop_turns * 2.0 * PI;
            let r = loop_radius * t * (1.0 - t * 0.3);  // Spiral in slightly

            let nx = cx + r * theta.cos();
            let ny = cy + r * theta.sin();

            if point_in_polygon(nx, ny, &polygon.outer) {
                let in_hole = polygon.holes.iter().any(|h| point_in_polygon(nx, ny, h));
                if !in_hole {
                    lines.push(Line::new(prev_x, prev_y, nx, ny));
                    prev_x = nx;
                    prev_y = ny;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_scribble_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_scribble_fill(&poly, 5.0, 45.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn different_angles_give_different_patterns() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_a = generate_scribble_fill(&poly, 10.0, 0.0);
        let lines_b = generate_scribble_fill(&poly, 10.0, 45.0);

        // Should produce different patterns
        assert!(!lines_a.is_empty());
        assert!(!lines_b.is_empty());

        // First lines should differ
        if !lines_a.is_empty() && !lines_b.is_empty() {
            let diff = (lines_a[0].x1 - lines_b[0].x1).abs() +
                       (lines_a[0].y1 - lines_b[0].y1).abs();
            assert!(diff > 0.1, "Different seeds should produce different patterns");
        }
    }

    #[test]
    fn scribble_has_smooth_curves() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_scribble_fill(&poly, 5.0, 0.0);

        // Check that consecutive lines are mostly connected (smooth path)
        let mut connected = 0;
        for i in 1..lines.len() {
            let prev = &lines[i - 1];
            let curr = &lines[i];
            let dist = ((prev.x2 - curr.x1).powi(2) + (prev.y2 - curr.y1).powi(2)).sqrt();
            if dist < 1.0 {
                connected += 1;
            }
        }

        // Most lines should be connected
        if lines.len() > 10 {
            assert!(connected > lines.len() / 3, "Scribble should have connected segments");
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

        let sparse = generate_scribble_fill(&poly, 20.0, 0.0);
        let dense = generate_scribble_fill(&poly, 5.0, 0.0);

        assert!(dense.len() > sparse.len());
    }
}
