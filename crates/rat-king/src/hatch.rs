//! Hatch line generation for fill patterns.
//!
//! Generates parallel lines that can be clipped to polygons
//! to create fill patterns like hatching, cross-hatching, etc.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};

// Re-import Point only for tests
#[cfg(test)]
use crate::geometry::Point;
use crate::clip::clip_lines_to_polygon;

/// Generate parallel hatch lines covering a polygon's bounding box.
///
/// ## Rust Lesson #17: f64 Methods
///
/// Rust's f64 has methods for math: `.sin()`, `.cos()`, `.sqrt()`, etc.
/// These compile to native CPU instructions - zero overhead.
///
/// Unlike JS, there's no Math.sin() - it's just `angle.sin()`.
pub fn generate_hatch_lines(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        // ## Rust Lesson #18: let-else
        //
        // `let Some(x) = expr else { return }` is a clean way to
        // unwrap an Option and handle the None case.
        // New in Rust 2024!
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let angle_rad = angle_degrees * PI / 180.0;

    // Diagonal covers rotated bbox (sqrt(2) ≈ 1.42)
    let diagonal = (width * width + height * height).sqrt() * 1.42;

    // Direction vectors
    let perp_x = (angle_rad + PI / 2.0).cos();
    let perp_y = (angle_rad + PI / 2.0).sin();
    let dir_x = angle_rad.cos();
    let dir_y = angle_rad.sin();

    // Center of bounding box
    let center_x = min_x + width / 2.0;
    let center_y = min_y + height / 2.0;

    // ## Rust Lesson #19: Integer Math
    //
    // `.ceil()` returns f64, we need to cast to integer.
    // `as i32` is explicit type coercion (no implicit conversions!)
    let num_lines = (diagonal / spacing).ceil() as i32 + 1;

    let mut lines = Vec::with_capacity((num_lines * 2 + 1) as usize);

    for i in -num_lines..=num_lines {
        let offset = i as f64 * spacing;
        let line_center_x = center_x + perp_x * offset;
        let line_center_y = center_y + perp_y * offset;

        lines.push(Line::new(
            line_center_x - dir_x * diagonal,
            line_center_y - dir_y * diagonal,
            line_center_x + dir_x * diagonal,
            line_center_y + dir_y * diagonal,
        ));
    }

    lines
}

/// Generate hatch lines and clip them to a polygon.
///
/// This is the main "lines" pattern function.
pub fn generate_lines_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let hatch_lines = generate_hatch_lines(polygon, spacing, angle_degrees);
    clip_lines_to_polygon(&hatch_lines, polygon)
}

/// Generate crosshatch pattern (two sets of perpendicular lines).
pub fn generate_crosshatch_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let mut lines = generate_lines_fill(polygon, spacing, angle_degrees);
    let perpendicular = generate_lines_fill(polygon, spacing, angle_degrees + 90.0);
    lines.extend(perpendicular);
    lines
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square() -> Polygon {
        Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ])
    }

    #[test]
    fn generates_horizontal_lines() {
        let poly = unit_square();
        let lines = generate_hatch_lines(&poly, 10.0, 0.0);
        // Should have many lines covering the square
        assert!(lines.len() > 10);
    }

    #[test]
    fn lines_fill_clips_to_polygon() {
        let poly = unit_square();
        let lines = generate_lines_fill(&poly, 10.0, 0.0);

        // All lines should be clipped to the polygon bounds
        for line in &lines {
            assert!(line.x1 >= -0.01 && line.x1 <= 100.01);
            assert!(line.x2 >= -0.01 && line.x2 <= 100.01);
        }

        // Should have roughly 10 lines (100 / 10 spacing)
        assert!(lines.len() >= 8 && lines.len() <= 12, "got {} lines", lines.len());
    }

    #[test]
    fn crosshatch_has_double_lines() {
        let poly = unit_square();
        let single = generate_lines_fill(&poly, 10.0, 0.0);
        let cross = generate_crosshatch_fill(&poly, 10.0, 0.0);

        // Crosshatch should have roughly 2x as many lines
        assert!(cross.len() >= single.len() * 2 - 2);
        assert!(cross.len() <= single.len() * 2 + 2);
    }
}
