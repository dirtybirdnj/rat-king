//! Honeycomb fill pattern - hexagonal grid.

use std::f64::consts::PI;
use std::collections::HashSet;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::{point_in_polygon, line_polygon_intersections};

/// Generate honeycomb (hexagonal grid) fill for a polygon.
pub fn generate_honeycomb_fill(
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

    // Hexagon geometry
    let hex_size = spacing * 1.5;
    let hex_width = hex_size * 2.0;
    let hex_height = hex_size * 3.0_f64.sqrt();
    let horiz_spacing = hex_width * 0.75;
    let vert_spacing = hex_height;

    // Pre-compute hex vertex offsets
    let hex_offsets: Vec<(f64, f64)> = (0..6)
        .map(|i| {
            let hex_angle = PI / 3.0 * i as f64;
            (hex_size * hex_angle.cos(), hex_size * hex_angle.sin())
        })
        .collect();

    // Calculate padding for rotation coverage
    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let padding = hex_size * 2.0 + diagonal / 2.0;

    let mut lines = Vec::new();

    // Rotate point around center
    let rotate = |x: f64, y: f64| -> (f64, f64) {
        let dx = x - center_x;
        let dy = y - center_y;
        (
            center_x + dx * angle_rad.cos() - dy * angle_rad.sin(),
            center_y + dx * angle_rad.sin() + dy * angle_rad.cos(),
        )
    };

    let mut row = 0;
    let mut y = min_y - padding;

    while y <= max_y + padding {
        let is_odd_row = row % 2 == 1;
        let x_offset = if is_odd_row { horiz_spacing * 0.5 } else { 0.0 };

        let mut x = min_x - padding + x_offset;

        while x <= max_x + padding {
            let (rotated_cx, rotated_cy) = rotate(x, y);

            // Build hex points
            let hex_points: Vec<(f64, f64)> = hex_offsets
                .iter()
                .map(|(ox, oy)| rotate(x + ox, y + oy))
                .collect();

            // Check if hexagon overlaps polygon
            let center_inside = point_in_polygon(rotated_cx, rotated_cy, outer);
            let any_vertex_inside = !center_inside && hex_points.iter().any(|(px, py)| {
                point_in_polygon(*px, *py, outer)
            });

            if center_inside || any_vertex_inside {
                for i in 0..6 {
                    let (x1, y1) = hex_points[i];
                    let (x2, y2) = hex_points[(i + 1) % 6];

                    let p1_inside = point_in_polygon(x1, y1, outer);
                    let p2_inside = point_in_polygon(x2, y2, outer);

                    if p1_inside && p2_inside {
                        // Check holes
                        let mid_x = (x1 + x2) / 2.0;
                        let mid_y = (y1 + y2) / 2.0;
                        let in_hole = polygon.holes.iter().any(|hole| {
                            point_in_polygon(mid_x, mid_y, hole)
                        });
                        if !in_hole {
                            lines.push(Line::new(x1, y1, x2, y2));
                        }
                    } else if p1_inside || p2_inside {
                        // One inside - clip to boundary
                        let intersections = line_polygon_intersections(x1, y1, x2, y2, outer);
                        if !intersections.is_empty() {
                            let (inside_x, inside_y) = if p1_inside { (x1, y1) } else { (x2, y2) };
                            let (ix, iy, _) = intersections
                                .iter()
                                .min_by(|a, b| {
                                    let da = (a.0 - inside_x).powi(2) + (a.1 - inside_y).powi(2);
                                    let db = (b.0 - inside_x).powi(2) + (b.1 - inside_y).powi(2);
                                    da.partial_cmp(&db).unwrap()
                                })
                                .copied()
                                .unwrap();

                            let mid_x = (inside_x + ix) / 2.0;
                            let mid_y = (inside_y + iy) / 2.0;
                            let in_hole = polygon.holes.iter().any(|hole| {
                                point_in_polygon(mid_x, mid_y, hole)
                            });
                            if !in_hole {
                                lines.push(Line::new(inside_x, inside_y, ix, iy));
                            }
                        }
                    } else {
                        // Both outside - check if line passes through
                        let intersections = line_polygon_intersections(x1, y1, x2, y2, outer);
                        if intersections.len() >= 2 {
                            let mut sorted = intersections.clone();
                            sorted.sort_by(|a, b| {
                                let da = (a.0 - x1).powi(2) + (a.1 - y1).powi(2);
                                let db = (b.0 - x1).powi(2) + (b.1 - y1).powi(2);
                                da.partial_cmp(&db).unwrap()
                            });

                            for j in (0..sorted.len() - 1).step_by(2) {
                                let (ix1, iy1, _) = sorted[j];
                                let (ix2, iy2, _) = sorted[j + 1];

                                let mid_x = (ix1 + ix2) / 2.0;
                                let mid_y = (iy1 + iy2) / 2.0;

                                if point_in_polygon(mid_x, mid_y, outer) {
                                    let in_hole = polygon.holes.iter().any(|hole| {
                                        point_in_polygon(mid_x, mid_y, hole)
                                    });
                                    if !in_hole {
                                        lines.push(Line::new(ix1, iy1, ix2, iy2));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            x += horiz_spacing;
        }

        y += vert_spacing * 0.5;
        row += 1;
    }

    // Remove duplicate lines
    deduplicate_lines(lines)
}

/// Remove duplicate lines (same endpoints in either order).
fn deduplicate_lines(lines: Vec<Line>) -> Vec<Line> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for line in lines {
        let key1 = format!("{:.2},{:.2}-{:.2},{:.2}", line.x1, line.y1, line.x2, line.y2);
        let key2 = format!("{:.2},{:.2}-{:.2},{:.2}", line.x2, line.y2, line.x1, line.y1);

        if !seen.contains(&key1) && !seen.contains(&key2) {
            seen.insert(key1);
            result.push(line);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_honeycomb_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_honeycomb_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }
}
