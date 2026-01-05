//! Peano curve fill pattern.
//!
//! The Peano curve is a space-filling curve discovered by Giuseppe Peano in 1890.
//! It uses 3x3 subdivisions (compared to Hilbert's 2x2) and creates a distinctive
//! pattern that fills space without overlapping.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate Peano curve fill for a polygon.
pub fn generate_peano_fill(
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

    // Calculate depth based on spacing (3^depth cells per side)
    let depth = ((size / spacing).log(3.0).ceil() as usize).clamp(1, 5);
    let grid_size = 3_usize.pow(depth as u32);
    let cell_size = size / grid_size as f64;

    // Generate Peano curve points using recursive algorithm
    let mut points = Vec::with_capacity(grid_size * grid_size);
    peano_recursive(0, 0, grid_size as i32, false, false, &mut points);

    // Transform points to polygon space
    let angle_rad = angle_degrees * PI / 180.0;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    let transformed: Vec<(f64, f64)> = points
        .iter()
        .map(|&(gx, gy)| {
            let x = min_x + (gx as f64 + 0.5) * cell_size;
            let y = min_y + (gy as f64 + 0.5) * cell_size;

            let dx = x - center_x;
            let dy = y - center_y;
            let rx = center_x + dx * angle_rad.cos() - dy * angle_rad.sin();
            let ry = center_y + dx * angle_rad.sin() + dy * angle_rad.cos();

            (rx, ry)
        })
        .collect();

    // Convert to line segments
    let mut lines = Vec::new();
    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for &(x, y) in &transformed {
        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
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

/// Recursively generate Peano curve points.
///
/// Uses flip states to track orientation. The key insight:
/// - flip_x controls whether we traverse columns left-to-right or right-to-left
/// - flip_y controls whether we traverse rows bottom-to-top or top-to-bottom
/// - Middle row (row 1) toggles flip_x for its children
/// - Middle column (col 1) toggles flip_y for its children
fn peano_recursive(
    x: i32, y: i32,     // Origin of current region
    size: i32,          // Size of current region
    flip_x: bool,       // Whether to flip horizontal direction
    flip_y: bool,       // Whether to flip vertical direction
    points: &mut Vec<(usize, usize)>,
) {
    if size == 1 {
        points.push((x as usize, y as usize));
        return;
    }

    let s = size / 3;

    // Visit 9 sub-regions in serpentine order
    for row_idx in 0..3i32 {
        // Actual row in grid (0, 1, 2 from bottom or top depending on flip_y)
        let row = if flip_y { 2 - row_idx } else { row_idx };

        for col_idx in 0..3i32 {
            // Serpentine: even rows go one way, odd rows go the other
            // The base direction depends on flip_x
            let go_right = (row_idx % 2 == 0) != flip_x;
            let col = if go_right { col_idx } else { 2 - col_idx };

            let sub_x = x + col * s;
            let sub_y = y + row * s;

            // Sub-region flip states based on grid position
            // Middle row (row 1) toggles x, middle column (col 1) toggles y
            let sub_flip_x = flip_x ^ (row == 1);
            let sub_flip_y = flip_y ^ (col == 1);

            peano_recursive(sub_x, sub_y, s, sub_flip_x, sub_flip_y, points);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn get_peano_points(n: usize) -> Vec<(usize, usize)> {
        let mut points = Vec::new();
        peano_recursive(0, 0, n as i32, false, false, &mut points);
        points
    }

    #[test]
    fn generates_peano_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_peano_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate lines");
    }

    #[test]
    fn peano_visits_all_cells_depth1() {
        let points = get_peano_points(3);
        assert_eq!(points.len(), 9, "Should visit 9 cells");

        let mut visited = [[false; 3]; 3];
        for (x, y) in &points {
            assert!(*x < 3 && *y < 3, "Coords out of bounds: ({}, {})", x, y);
            visited[*x][*y] = true;
        }

        for x in 0..3 {
            for y in 0..3 {
                assert!(visited[x][y], "Cell ({}, {}) not visited", x, y);
            }
        }
    }

    #[test]
    fn peano_continuous_depth1() {
        let points = get_peano_points(3);

        for i in 1..points.len() {
            let (x1, y1) = points[i - 1];
            let (x2, y2) = points[i];
            let dx = (x1 as i32 - x2 as i32).abs();
            let dy = (y1 as i32 - y2 as i32).abs();
            assert!(
                (dx == 1 && dy == 0) || (dx == 0 && dy == 1),
                "Points {} and {} are not adjacent: ({},{}) -> ({},{})",
                i-1, i, x1, y1, x2, y2
            );
        }
    }

    #[test]
    fn peano_visits_all_cells_depth2() {
        let points = get_peano_points(9);
        assert_eq!(points.len(), 81, "Should visit 81 cells");

        let mut visited = vec![vec![false; 9]; 9];
        for (x, y) in &points {
            assert!(*x < 9 && *y < 9, "Coords out of bounds: ({}, {})", x, y);
            visited[*x][*y] = true;
        }

        for x in 0..9 {
            for y in 0..9 {
                assert!(visited[x][y], "Cell ({}, {}) not visited", x, y);
            }
        }
    }

    #[test]
    fn peano_continuous_depth2() {
        let points = get_peano_points(9);

        for i in 1..points.len() {
            let (x1, y1) = points[i - 1];
            let (x2, y2) = points[i];
            let dx = (x1 as i32 - x2 as i32).abs();
            let dy = (y1 as i32 - y2 as i32).abs();
            assert!(
                (dx == 1 && dy == 0) || (dx == 0 && dy == 1),
                "Points {} and {} are not adjacent: ({},{}) -> ({},{})\nAll points: {:?}",
                i-1, i, x1, y1, x2, y2, &points[..20.min(points.len())]
            );
        }
    }
}
