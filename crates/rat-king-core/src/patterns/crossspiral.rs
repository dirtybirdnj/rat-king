//! Cross-spiral fill pattern - two opposing spirals.
//!
//! TODO: Implement actual cross-spiral algorithm.
//! Currently returns simple crosshatch as a placeholder.

use crate::geometry::{Line, Polygon};
use crate::hatch::generate_crosshatch_fill;

/// Generate cross-spiral fill for a polygon.
///
/// **STUB IMPLEMENTATION** - Returns crosshatch until proper cross-spiral is implemented.
pub fn generate_crossspiral_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // TODO: Implement dual opposing spirals
    // For now, fall back to crosshatch with a message
    eprintln!("WARNING: crossspiral pattern is a stub - using crosshatch instead");
    generate_crosshatch_fill(polygon, spacing, angle_degrees)
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
        let lines = generate_crossspiral_fill(&poly, 5.0, 45.0);
        assert!(!lines.is_empty());
    }
}
