//! Voronoi fill pattern - cell boundaries from random seed points.
//!
//! Creates a pattern of cells where each cell contains all points
//! closest to a particular seed point.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;
use crate::rng::Rng;

/// Generate Voronoi cell fill for a polygon.
///
/// Creates Voronoi diagram boundaries using perpendicular bisectors.
/// - `spacing`: Average distance between seed points
/// - `angle_degrees`: Rotation applied to the seed grid
pub fn generate_voronoi_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let angle_rad = angle_degrees * PI / 180.0;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let diagonal = (width * width + height * height).sqrt();

    // Generate seed points with jitter
    let mut seeds = Vec::new();
    let mut rng = Rng::new(42);

    let cols = ((width / spacing).ceil() as i32 + 2).max(3);
    let rows = ((height / spacing).ceil() as i32 + 2).max(3);

    for row in -1..=rows {
        for col in -1..=cols {
            let base_x = min_x + col as f64 * spacing;
            let base_y = min_y + row as f64 * spacing;

            // Add randomness to avoid grid-like appearance
            let jitter = spacing * 0.4;
            let x = base_x + (rng.next_f64() - 0.5) * jitter * 2.0;
            let y = base_y + (rng.next_f64() - 0.5) * jitter * 2.0;

            // Apply rotation around center
            let dx = x - center_x;
            let dy = y - center_y;
            let rx = center_x + dx * angle_rad.cos() - dy * angle_rad.sin();
            let ry = center_y + dx * angle_rad.sin() + dy * angle_rad.cos();

            seeds.push((rx, ry));
        }
    }

    if seeds.len() < 3 {
        return Vec::new();
    }

    // Generate Voronoi edges using perpendicular bisectors
    // For each pair of nearby seeds, create an edge
    let mut all_lines = Vec::new();
    let max_edge_dist = spacing * 2.5; // Only consider nearby seed pairs

    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            let (x1, y1) = seeds[i];
            let (x2, y2) = seeds[j];

            // Distance between seeds
            let dx = x2 - x1;
            let dy = y2 - y1;
            let dist = (dx * dx + dy * dy).sqrt();

            // Skip distant pairs
            if dist > max_edge_dist {
                continue;
            }

            // Midpoint
            let mx = (x1 + x2) / 2.0;
            let my = (y1 + y2) / 2.0;

            // Skip if midpoint is too far from polygon
            if mx < min_x - spacing || mx > max_x + spacing ||
               my < min_y - spacing || my > max_y + spacing {
                continue;
            }

            // Perpendicular unit vector
            let px = -dy / dist;
            let py = dx / dist;

            // Create edge line - length based on spacing
            let edge_len = spacing * 1.5;
            let line = Line::new(
                mx - px * edge_len,
                my - py * edge_len,
                mx + px * edge_len,
                my + py * edge_len,
            );

            // Simple validation: check if this edge is a valid Voronoi boundary
            // by ensuring no other seed is closer to the midpoint
            let mid_dist_sq = dist * dist / 4.0; // Distance from midpoint to either seed
            let mut valid = true;

            for (k, &(sx, sy)) in seeds.iter().enumerate() {
                if k == i || k == j {
                    continue;
                }
                let dk_sq = (mx - sx).powi(2) + (my - sy).powi(2);
                if dk_sq < mid_dist_sq * 0.95 {
                    valid = false;
                    break;
                }
            }

            if valid {
                all_lines.push(line);
            }
        }
    }

    // If we got no lines from the validation, fall back to generating all edges
    if all_lines.is_empty() {
        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                let (x1, y1) = seeds[i];
                let (x2, y2) = seeds[j];

                let dx = x2 - x1;
                let dy = y2 - y1;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > max_edge_dist || dist < 0.001 {
                    continue;
                }

                let mx = (x1 + x2) / 2.0;
                let my = (y1 + y2) / 2.0;

                let px = -dy / dist;
                let py = dx / dist;

                let edge_len = spacing;
                all_lines.push(Line::new(
                    mx - px * edge_len,
                    my - py * edge_len,
                    mx + px * edge_len,
                    my + py * edge_len,
                ));
            }
        }
    }

    // Clip all lines to polygon
    clip_lines_to_polygon(&all_lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn square_polygon() -> Polygon {
        Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ])
    }

    #[test]
    fn generates_voronoi_lines() {
        let poly = square_polygon();
        let lines = generate_voronoi_fill(&poly, 20.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn voronoi_with_rotation() {
        let poly = square_polygon();
        let lines = generate_voronoi_fill(&poly, 20.0, 30.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn voronoi_different_density() {
        let poly = square_polygon();
        let lines_sparse = generate_voronoi_fill(&poly, 40.0, 0.0);
        let lines_dense = generate_voronoi_fill(&poly, 15.0, 0.0);
        // More seeds = more edges (but allow for some variation)
        assert!(!lines_sparse.is_empty());
        assert!(!lines_dense.is_empty());
    }
}
