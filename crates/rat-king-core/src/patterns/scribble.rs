//! Scribble fill pattern - random scribble lines.
//!
//! TODO: Implement actual scribble algorithm.
//! Currently returns simple diagonal lines as a placeholder.

use crate::geometry::{Line, Polygon};
use crate::hatch::generate_lines_fill;

/// Generate scribble fill for a polygon.
///
/// **STUB IMPLEMENTATION** - Returns simple lines until proper scribble is implemented.
pub fn generate_scribble_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // TODO: Implement random/organic scribble pattern
    // For now, fall back to lines with a message
    eprintln!("WARNING: scribble pattern is a stub - using lines instead");
    generate_lines_fill(polygon, spacing, angle_degrees)
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
        let lines = generate_scribble_fill(&poly, 5.0, 45.0);
        assert!(!lines.is_empty());
    }
}
