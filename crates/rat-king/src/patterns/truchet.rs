//! Truchet tile pattern.
//!
//! Truchet tiles are square tiles with a diagonal or curved pattern.
//! When placed randomly, they create maze-like or flowing patterns.
//!
//! This implementation uses quarter-circle arcs in each cell,
//! creating a flowing, organic appearance.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Simple pseudo-random number generator for deterministic patterns.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // XorShift64
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_bool(&mut self) -> bool {
        self.next() % 2 == 0
    }
}

/// Generate Truchet tile pattern fill for a polygon.
///
/// Creates quarter-circle arcs in a grid, randomly oriented to create
/// flowing, maze-like patterns.
pub fn generate_truchet_fill(
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

    let cell_size = spacing * 2.0;
    let arc_segments = 8; // Number of line segments per quarter-circle

    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let padding = cell_size + diagonal / 2.0;

    let mut lines = Vec::new();
    let mut rng = SimpleRng::new(42); // Fixed seed for reproducibility

    let rotate = |x: f64, y: f64| -> (f64, f64) {
        let dx = x - center_x;
        let dy = y - center_y;
        (
            center_x + dx * angle_rad.cos() - dy * angle_rad.sin(),
            center_y + dx * angle_rad.sin() + dy * angle_rad.cos(),
        )
    };

    let mut cell_y = min_y - padding;
    while cell_y <= max_y + padding {
        let mut cell_x = min_x - padding;
        while cell_x <= max_x + padding {
            // Decide tile orientation
            let flip = rng.next_bool();

            // Generate two quarter-circle arcs per cell
            // Arc 1: corner to corner
            // Arc 2: other corner to corner

            let (arc1_cx, arc1_cy, arc2_cx, arc2_cy) = if flip {
                // Arcs from top-left and bottom-right
                (cell_x, cell_y, cell_x + cell_size, cell_y + cell_size)
            } else {
                // Arcs from top-right and bottom-left
                (cell_x + cell_size, cell_y, cell_x, cell_y + cell_size)
            };

            let radius = cell_size / 2.0;

            // Generate arc 1
            for i in 0..arc_segments {
                let t1 = i as f64 / arc_segments as f64;
                let t2 = (i + 1) as f64 / arc_segments as f64;

                let (start_angle, end_angle) = if flip {
                    (0.0, PI / 2.0) // Bottom-right quadrant
                } else {
                    (PI / 2.0, PI) // Bottom-left quadrant
                };

                let a1 = start_angle + t1 * (end_angle - start_angle);
                let a2 = start_angle + t2 * (end_angle - start_angle);

                let x1 = arc1_cx + radius * a1.cos();
                let y1 = arc1_cy + radius * a1.sin();
                let x2 = arc1_cx + radius * a2.cos();
                let y2 = arc1_cy + radius * a2.sin();

                let (rx1, ry1) = rotate(x1, y1);
                let (rx2, ry2) = rotate(x2, y2);

                let mid_x = (rx1 + rx2) / 2.0;
                let mid_y = (ry1 + ry2) / 2.0;

                if point_in_polygon(mid_x, mid_y, outer) {
                    let in_hole = polygon.holes.iter().any(|hole| {
                        point_in_polygon(mid_x, mid_y, hole)
                    });
                    if !in_hole {
                        lines.push(Line::new(rx1, ry1, rx2, ry2));
                    }
                }
            }

            // Generate arc 2
            for i in 0..arc_segments {
                let t1 = i as f64 / arc_segments as f64;
                let t2 = (i + 1) as f64 / arc_segments as f64;

                let (start_angle, end_angle) = if flip {
                    (PI, PI * 3.0 / 2.0) // Top-left quadrant
                } else {
                    (PI * 3.0 / 2.0, PI * 2.0) // Top-right quadrant
                };

                let a1 = start_angle + t1 * (end_angle - start_angle);
                let a2 = start_angle + t2 * (end_angle - start_angle);

                let x1 = arc2_cx + radius * a1.cos();
                let y1 = arc2_cy + radius * a1.sin();
                let x2 = arc2_cx + radius * a2.cos();
                let y2 = arc2_cy + radius * a2.sin();

                let (rx1, ry1) = rotate(x1, y1);
                let (rx2, ry2) = rotate(x2, y2);

                let mid_x = (rx1 + rx2) / 2.0;
                let mid_y = (ry1 + ry2) / 2.0;

                if point_in_polygon(mid_x, mid_y, outer) {
                    let in_hole = polygon.holes.iter().any(|hole| {
                        point_in_polygon(mid_x, mid_y, hole)
                    });
                    if !in_hole {
                        lines.push(Line::new(rx1, ry1, rx2, ry2));
                    }
                }
            }

            cell_x += cell_size;
        }
        cell_y += cell_size;
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_truchet_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_truchet_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate truchet lines");
    }

    #[test]
    fn truchet_is_deterministic() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines1 = generate_truchet_fill(&poly, 10.0, 0.0);
        let lines2 = generate_truchet_fill(&poly, 10.0, 0.0);

        assert_eq!(lines1.len(), lines2.len(), "Same seed should produce same output");
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_small = generate_truchet_fill(&poly, 5.0, 0.0);
        let lines_large = generate_truchet_fill(&poly, 20.0, 0.0);

        assert!(lines_small.len() > lines_large.len(),
            "Smaller spacing should produce more lines");
    }
}
