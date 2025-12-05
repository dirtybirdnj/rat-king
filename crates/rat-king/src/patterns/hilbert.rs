//! Hilbert curve fill pattern - space-filling curve.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate Hilbert curve fill for a polygon.
///
/// The Hilbert curve is a continuous space-filling curve that visits every
/// cell in a grid exactly once. Great for single-stroke fills.
pub fn generate_hilbert_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let size = width.max(height);

    // Calculate depth based on spacing
    // Each depth level doubles the grid size
    let depth = ((size / spacing).log2().ceil() as usize).clamp(1, 8);
    let grid_size = 1 << depth; // 2^depth
    let cell_size = size / grid_size as f64;

    // Generate Hilbert curve points
    let points = hilbert_points(depth);

    // Transform points to polygon space
    let start_angle = angle_degrees * PI / 180.0;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let transformed: Vec<(f64, f64)> = points
        .iter()
        .map(|&(gx, gy)| {
            // Map grid coordinates to world coordinates
            let x = min_x + (gx as f64 + 0.5) * cell_size;
            let y = min_y + (gy as f64 + 0.5) * cell_size;

            // Rotate around center
            let dx = x - center_x;
            let dy = y - center_y;
            let rx = center_x + dx * start_angle.cos() - dy * start_angle.sin();
            let ry = center_y + dx * start_angle.sin() + dy * start_angle.cos();

            (rx, ry)
        })
        .collect();

    // Convert to line segments, clipping to polygon
    let mut lines = Vec::new();
    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for &(x, y) in &transformed {
        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
                // Check holes
                let mid_x = (px + x) / 2.0;
                let mid_y = (py + y) / 2.0;
                let in_hole = polygon.holes.iter().any(|hole| {
                    point_in_polygon(mid_x, mid_y, hole)
                });
                if !in_hole {
                    lines.push(Line::new(px, py, x, y));
                }
            }
        }

        prev_point = Some((x, y));
        prev_inside = current_inside;
    }

    lines
}

/// Generate Hilbert curve points using recursive algorithm.
///
/// Returns grid coordinates (x, y) where 0 <= x, y < 2^depth
fn hilbert_points(depth: usize) -> Vec<(usize, usize)> {
    let n = 1 << depth; // 2^depth
    let mut points = Vec::with_capacity(n * n);

    for i in 0..(n * n) {
        let (x, y) = d2xy(n, i);
        points.push((x, y));
    }

    points
}

/// Convert distance along Hilbert curve to (x, y) coordinates.
///
/// Uses the standard algorithm for Hilbert curve generation.
fn d2xy(n: usize, d: usize) -> (usize, usize) {
    let mut x = 0;
    let mut y = 0;
    let mut d = d;
    let mut s = 1;

    while s < n {
        let rx = 1 & (d / 2);
        let ry = 1 & (d ^ rx);

        // Rotate quadrant
        if ry == 0 {
            if rx == 1 {
                x = s - 1 - x;
                y = s - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }

        x += s * rx;
        y += s * ry;
        d /= 4;
        s *= 2;
    }

    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_hilbert_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_hilbert_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn hilbert_d2xy_basic() {
        // For n=2 (depth=1), the Hilbert curve visits 4 cells
        let points: Vec<_> = (0..4).map(|d| d2xy(2, d)).collect();

        // Should visit all 4 cells exactly once
        let mut visited = [[false; 2]; 2];
        for (x, y) in points {
            assert!(!visited[x][y], "Cell visited twice");
            visited[x][y] = true;
        }

        // Verify all visited
        for row in visited {
            for cell in row {
                assert!(cell, "Cell not visited");
            }
        }
    }

    #[test]
    fn hilbert_d2xy_larger() {
        // For n=4 (depth=2), 16 cells
        let n = 4;
        let points: Vec<_> = (0..(n * n)).map(|d| d2xy(n, d)).collect();

        let mut visited = [[false; 4]; 4];
        for (x, y) in points {
            assert!(x < n && y < n, "Coordinates out of bounds");
            assert!(!visited[x][y], "Cell visited twice");
            visited[x][y] = true;
        }
    }

    #[test]
    fn depth_scales_with_spacing() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_sparse = generate_hilbert_fill(&poly, 20.0, 0.0);
        let lines_dense = generate_hilbert_fill(&poly, 5.0, 0.0);

        // Denser spacing should produce more lines
        assert!(lines_dense.len() > lines_sparse.len());
    }
}
