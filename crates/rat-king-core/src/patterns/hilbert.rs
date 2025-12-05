//! Hilbert curve fill pattern - space-filling curve.
//!
//! TODO: Implement actual Hilbert curve algorithm.
//! Currently returns simple zigzag as a placeholder.

use crate::geometry::{Line, Polygon};
use super::generate_zigzag_fill;

/// Generate Hilbert curve fill for a polygon.
///
/// **STUB IMPLEMENTATION** - Returns zigzag until proper Hilbert curve is implemented.
pub fn generate_hilbert_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // TODO: Implement Hilbert curve space-filling pattern
    // For now, fall back to zigzag with a message
    eprintln!("WARNING: hilbert pattern is a stub - using zigzag instead");
    generate_zigzag_fill(polygon, spacing, angle_degrees, spacing)
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
        let lines = generate_hilbert_fill(&poly, 5.0, 45.0);
        assert!(!lines.is_empty());
    }
}
