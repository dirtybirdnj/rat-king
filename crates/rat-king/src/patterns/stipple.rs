//! Stipple fill pattern - dot stippling using small marks.
//!
//! Creates a stippled appearance using small line segments (dots).
//! Useful for creating tonal gradients and textures.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::point_in_polygon;

/// Simple pseudo-random number generator.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

/// Generate stipple (dot) pattern fill for a polygon.
///
/// Creates small line segments (dots) scattered across the polygon.
/// Dot density is controlled by spacing parameter.
pub fn generate_stipple_fill(
    polygon: &Polygon,
    spacing: f64,
    _angle_degrees: f64, // Unused for stipple, but kept for API consistency
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let area = width * height;

    // Calculate number of dots based on spacing
    // Lower spacing = more dots
    let dot_density = 1.0 / (spacing * spacing);
    let num_dots = (area * dot_density * 0.5) as usize;
    let num_dots = num_dots.max(10).min(50000); // Reasonable limits

    // Dot size (small line segment)
    let dot_size = spacing * 0.15;

    let mut lines = Vec::with_capacity(num_dots);
    let mut rng = SimpleRng::new(12345);

    let mut attempts = 0;
    let max_attempts = num_dots * 10;

    while lines.len() < num_dots && attempts < max_attempts {
        attempts += 1;

        // Random position within bounding box
        let x = min_x + rng.next_f64() * width;
        let y = min_y + rng.next_f64() * height;

        // Check if inside polygon
        if !point_in_polygon(x, y, outer) {
            continue;
        }

        // Check holes
        let in_hole = polygon.holes.iter().any(|hole| {
            point_in_polygon(x, y, hole)
        });
        if in_hole {
            continue;
        }

        // Random angle for the dot
        let angle = rng.next_f64() * PI * 2.0;

        // Create small line segment (dot)
        let dx = dot_size * angle.cos();
        let dy = dot_size * angle.sin();

        lines.push(Line::new(
            x - dx / 2.0,
            y - dy / 2.0,
            x + dx / 2.0,
            y + dy / 2.0,
        ));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_stipple_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_stipple_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty(), "Should generate stipple dots");
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_dense = generate_stipple_fill(&poly, 2.0, 0.0);
        let lines_sparse = generate_stipple_fill(&poly, 10.0, 0.0);

        assert!(lines_dense.len() > lines_sparse.len(),
            "Smaller spacing should produce more dots");
    }

    #[test]
    fn dots_are_small() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_stipple_fill(&poly, 5.0, 0.0);

        for line in &lines {
            let length = ((line.x2 - line.x1).powi(2) + (line.y2 - line.y1).powi(2)).sqrt();
            assert!(length < 2.0, "Dots should be small line segments");
        }
    }

    #[test]
    fn stipple_is_deterministic() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines1 = generate_stipple_fill(&poly, 5.0, 0.0);
        let lines2 = generate_stipple_fill(&poly, 5.0, 0.0);

        assert_eq!(lines1.len(), lines2.len());
    }
}
