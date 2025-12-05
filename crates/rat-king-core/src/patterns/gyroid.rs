//! Gyroid fill pattern - 3D minimal surface projection.
//!
//! TODO: Implement actual gyroid algorithm.
//! Currently returns simple wiggle as a placeholder.

use crate::geometry::{Line, Polygon};
use super::generate_wiggle_fill;

/// Generate gyroid fill for a polygon.
///
/// **STUB IMPLEMENTATION** - Returns wiggle until proper gyroid is implemented.
pub fn generate_gyroid_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // TODO: Implement gyroid minimal surface pattern
    // For now, fall back to wiggle with a message
    eprintln!("WARNING: gyroid pattern is a stub - using wiggle instead");
    generate_wiggle_fill(polygon, spacing, angle_degrees, spacing, 0.15)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn stub_generates_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_gyroid_fill(&poly, 5.0, 45.0);
        assert!(!lines.is_empty());
    }
}
