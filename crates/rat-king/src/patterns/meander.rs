//! Meander fill pattern.
//!
//! A serpentine back-and-forth pattern that visits every cell exactly once
//! without overlapping, like mowing a lawn.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Generate meander fill for a polygon.
///
/// Creates a serpentine back-and-forth pattern that covers the polygon area
/// without any overlapping lines.
pub fn generate_meander_fill(
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

    // Calculate grid size based on spacing
    let grid_size = (size / spacing).ceil() as usize;
    let grid_size = grid_size.max(3);
    let cell_size = size / grid_size as f64;

    // Generate serpentine meander points
    let points = meander_points(grid_size);

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

/// Generate serpentine meander points that visit every cell exactly once.
fn meander_points(size: usize) -> Vec<(usize, usize)> {
    let mut points = Vec::with_capacity(size * size);

    for row in 0..size {
        if row % 2 == 0 {
            for col in 0..size {
                points.push((col, row));
            }
        } else {
            for col in (0..size).rev() {
                points.push((col, row));
            }
        }
    }

    points
}
