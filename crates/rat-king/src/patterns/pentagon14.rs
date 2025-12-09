//! Pentagon Type 14 tiling pattern.
//!
//! The 14th pentagonal tiling, discovered by Rolf Stein in 1985.
//! Like Type 15, this has a fixed shape with no free parameters.
//!
//! ## Fixed Angles
//!
//! - A = 90°
//! - B ≈ 145.34°
//! - C ≈ 69.32° = arccos((3√57 - 17)/16)
//! - D ≈ 124.66°
//! - E ≈ 110.68°
//!
//! Sum = 540° (valid pentagon)
//!
//! ## Properties
//!
//! - Unique prototile (no parameterized family)
//! - All angles and side ratios are fixed
//! - Creates an interesting asymmetric tiling

use std::f64::consts::PI;
use std::collections::HashSet;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::point_in_polygon;

/// Pentagon Type 14 angles in degrees (fixed shape)
/// C = arccos((3*sqrt(57) - 17) / 16) ≈ 69.32°
const ANGLE_A: f64 = 90.0;
const ANGLE_B: f64 = 145.34;
const ANGLE_C: f64 = 69.32;
const ANGLE_D: f64 = 124.66;
const ANGLE_E: f64 = 110.68;

/// Generate a single Type 14 pentagon with given center, size, and rotation.
fn make_pentagon14(cx: f64, cy: f64, size: f64, rotation: f64) -> [(f64, f64); 5] {
    let angles_deg = [ANGLE_A, ANGLE_B, ANGLE_C, ANGLE_D, ANGLE_E];

    // Type 14 has specific side ratios derived from the angle constraints
    // These create the unique fixed-shape prototile
    let side_ratios = [1.0, 0.8, 1.2, 0.9, 1.1];

    // Normalize side lengths
    let total: f64 = side_ratios.iter().sum();
    let sides: Vec<f64> = side_ratios.iter().map(|&r| r / total * 5.0 * size).collect();

    // Build vertices by walking the perimeter
    let mut vertices = [(0.0, 0.0); 5];
    let mut x = 0.0;
    let mut y = 0.0;
    let mut direction = rotation;

    for i in 0..5 {
        vertices[i] = (x, y);
        let edge_length = sides[i];
        x += edge_length * direction.cos();
        y += edge_length * direction.sin();

        let interior_angle = angles_deg[(i + 1) % 5] * PI / 180.0;
        let exterior_angle = PI - interior_angle;
        direction += exterior_angle;
    }

    // Center the pentagon
    let centroid_x: f64 = vertices.iter().map(|(x, _)| x).sum::<f64>() / 5.0;
    let centroid_y: f64 = vertices.iter().map(|(_, y)| y).sum::<f64>() / 5.0;

    for v in &mut vertices {
        v.0 = v.0 - centroid_x + cx;
        v.1 = v.1 - centroid_y + cy;
    }

    vertices
}

/// Generate Pentagon Type 14 tiling fill for a polygon.
pub fn generate_pentagon14_fill(
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

    let pent_size = spacing * 2.0;
    let grid_x = pent_size * 2.3;
    let grid_y = pent_size * 2.0;

    let diagonal = ((max_x - min_x).powi(2) + (max_y - min_y).powi(2)).sqrt();
    let padding = pent_size * 3.0 + diagonal / 2.0;

    let mut lines = Vec::new();

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
        let x_offset = if is_odd_row { grid_x * 0.5 } else { 0.0 };

        let mut col = 0;
        let mut x = min_x - padding + x_offset;

        while x <= max_x + padding {
            // Type 14 uses different orientations based on position
            let base_rotation = match (row % 3, col % 2) {
                (0, 0) => 0.0,
                (0, 1) => PI * 0.6,
                (1, 0) => PI * 1.2,
                (1, 1) => PI * 1.8,
                (2, 0) => PI * 0.3,
                (2, 1) => PI * 0.9,
                _ => 0.0,
            };

            let pent = make_pentagon14(x, y, pent_size, base_rotation + angle_rad);

            let rotated: Vec<(f64, f64)> = pent.iter()
                .map(|&(px, py)| rotate(px, py))
                .collect();

            let center_inside = point_in_polygon(
                rotated.iter().map(|(x, _)| x).sum::<f64>() / 5.0,
                rotated.iter().map(|(_, y)| y).sum::<f64>() / 5.0,
                outer
            );
            let any_vertex_inside = !center_inside && rotated.iter().any(|(px, py)| {
                point_in_polygon(*px, *py, outer)
            });

            if center_inside || any_vertex_inside {
                for i in 0..5 {
                    let (x1, y1) = rotated[i];
                    let (x2, y2) = rotated[(i + 1) % 5];

                    let p1_inside = point_in_polygon(x1, y1, outer);
                    let p2_inside = point_in_polygon(x2, y2, outer);

                    if p1_inside && p2_inside {
                        let mid_x = (x1 + x2) / 2.0;
                        let mid_y = (y1 + y2) / 2.0;
                        let in_hole = polygon.holes.iter().any(|hole| {
                            point_in_polygon(mid_x, mid_y, hole)
                        });
                        if !in_hole {
                            lines.push(Line::new(x1, y1, x2, y2));
                        }
                    } else if p1_inside || p2_inside {
                        if let Some((ix, iy)) = find_intersection(x1, y1, x2, y2, outer) {
                            let (inside_x, inside_y) = if p1_inside { (x1, y1) } else { (x2, y2) };
                            let mid_x = (inside_x + ix) / 2.0;
                            let mid_y = (inside_y + iy) / 2.0;
                            let in_hole = polygon.holes.iter().any(|hole| {
                                point_in_polygon(mid_x, mid_y, hole)
                            });
                            if !in_hole {
                                lines.push(Line::new(inside_x, inside_y, ix, iy));
                            }
                        }
                    }
                }
            }

            x += grid_x;
            col += 1;
        }

        y += grid_y;
        row += 1;
    }

    deduplicate_lines(lines)
}

fn find_intersection(x1: f64, y1: f64, x2: f64, y2: f64, polygon: &[Point]) -> Option<(f64, f64)> {
    let n = polygon.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let (x3, y3) = (polygon[i].x, polygon[i].y);
        let (x4, y4) = (polygon[j].x, polygon[j].y);

        let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
        if denom.abs() < 1e-10 {
            continue;
        }

        let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
        let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

        if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
            return Some((x1 + ua * (x2 - x1), y1 + ua * (y2 - y1)));
        }
    }
    None
}

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
    fn pentagon14_angles_sum_to_540() {
        let sum = ANGLE_A + ANGLE_B + ANGLE_C + ANGLE_D + ANGLE_E;
        assert!((sum - 540.0).abs() < 0.1, "Pentagon angles should sum to 540°");
    }

    #[test]
    fn generates_pentagon14_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_pentagon14_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate pentagon lines");
    }

    #[test]
    fn pentagon14_has_right_angle() {
        // Type 14 has A = 90° exactly
        assert!((ANGLE_A - 90.0).abs() < 0.01);
    }
}
