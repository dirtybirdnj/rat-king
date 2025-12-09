//! Peano curve fill pattern.
//!
//! The Peano curve is a space-filling curve discovered by Giuseppe Peano in 1890.
//! Unlike the Hilbert curve which uses 2x2 subdivisions, Peano uses 3x3 subdivisions.
//! This creates a denser, more intricate pattern.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate points along a Peano curve.
///
/// Uses recursive subdivision with 3x3 grid.
fn peano_points(
    x: f64, y: f64,
    size: f64,
    depth: usize,
    orientation: u8,
    points: &mut Vec<(f64, f64)>,
) {
    if depth == 0 {
        points.push((x + size / 2.0, y + size / 2.0));
        return;
    }

    let step = size / 3.0;

    // Peano curve visits 9 subcells in a specific order based on orientation
    // There are different ways to traverse; this is one common variant
    let orders: [[usize; 9]; 4] = [
        [0, 1, 2, 5, 4, 3, 6, 7, 8], // Right-up
        [0, 3, 6, 7, 4, 1, 2, 5, 8], // Up-right
        [8, 7, 6, 3, 4, 5, 2, 1, 0], // Left-down
        [8, 5, 2, 1, 4, 7, 6, 3, 0], // Down-left
    ];

    let orientations: [[u8; 9]; 4] = [
        [1, 0, 1, 0, 0, 0, 1, 0, 1],
        [0, 1, 0, 1, 1, 1, 0, 1, 0],
        [1, 0, 1, 0, 0, 0, 1, 0, 1],
        [0, 1, 0, 1, 1, 1, 0, 1, 0],
    ];

    let order = &orders[orientation as usize % 4];
    let child_orientations = &orientations[orientation as usize % 4];

    for (i, &cell) in order.iter().enumerate() {
        let cx = (cell % 3) as f64;
        let cy = (cell / 3) as f64;

        peano_points(
            x + cx * step,
            y + cy * step,
            step,
            depth - 1,
            child_orientations[i],
            points,
        );
    }
}

/// Generate Peano curve fill for a polygon.
///
/// Creates a space-filling Peano curve that covers the polygon area.
pub fn generate_peano_fill(
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

    let width = max_x - min_x;
    let height = max_y - min_y;
    let size = width.max(height);

    // Calculate depth based on spacing
    // Peano subdivides by 3, so each level has 3^depth cells
    let cells_needed = (size / spacing) as usize;
    let depth = (cells_needed as f64).log(3.0).ceil() as usize;
    let depth = depth.max(1).min(6); // Limit depth for performance

    // Adjust size to be a power of 3
    let adjusted_size = 3_f64.powi(depth as i32) * spacing;
    let offset_x = center_x - adjusted_size / 2.0;
    let offset_y = center_y - adjusted_size / 2.0;

    // Generate Peano curve points
    let mut points = Vec::new();
    peano_points(offset_x, offset_y, adjusted_size, depth, 0, &mut points);

    // Rotate points
    let rotate = |x: f64, y: f64| -> (f64, f64) {
        let dx = x - center_x;
        let dy = y - center_y;
        (
            center_x + dx * angle_rad.cos() - dy * angle_rad.sin(),
            center_y + dx * angle_rad.sin() + dy * angle_rad.cos(),
        )
    };

    let rotated_points: Vec<(f64, f64)> = points.iter()
        .map(|&(x, y)| rotate(x, y))
        .collect();

    // Convert to lines
    let mut lines = Vec::new();
    for i in 0..rotated_points.len().saturating_sub(1) {
        let (x1, y1) = rotated_points[i];
        let (x2, y2) = rotated_points[i + 1];
        lines.push(Line::new(x1, y1, x2, y2));
    }

    // Clip to polygon
    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_peano_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_peano_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate peano lines");
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_small = generate_peano_fill(&poly, 5.0, 0.0);
        let lines_large = generate_peano_fill(&poly, 20.0, 0.0);

        // Smaller spacing should produce more detailed curve
        assert!(lines_small.len() >= lines_large.len(),
            "Smaller spacing should produce at least as many lines");
    }

    #[test]
    fn rotation_works() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_0 = generate_peano_fill(&poly, 10.0, 0.0);
        let lines_45 = generate_peano_fill(&poly, 10.0, 45.0);

        assert!(!lines_0.is_empty());
        assert!(!lines_45.is_empty());
    }
}
