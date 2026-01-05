//! Gyroid fill pattern - 3D minimal surface projection.
//!
//! The gyroid is a triply periodic minimal surface discovered by Alan Schoen.
//! Its implicit equation is:
//!   sin(x)cos(y) + sin(y)cos(z) + sin(z)cos(x) = 0
//!
//! We create 2D patterns by taking horizontal slices at different z values
//! and contouring the resulting 2D field.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Evaluate the gyroid function at a point.
/// Returns a scalar field value - the zero-contour is the gyroid surface.
#[inline]
fn gyroid_field(x: f64, y: f64, z: f64) -> f64 {
    x.sin() * y.cos() + y.sin() * z.cos() + z.sin() * x.cos()
}

/// Generate gyroid fill for a polygon.
///
/// Creates organic, flowing patterns based on 2D slices of the 3D gyroid surface.
/// Multiple z-slices are stacked for dense coverage.
///
/// Parameters:
/// - `spacing`: Controls the scale of the gyroid pattern
/// - `angle_degrees`: Z-offset for the slice (different angles = different patterns)
pub fn generate_gyroid_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    // Scale factor to map polygon space to gyroid space
    // spacing controls the "wavelength" of the gyroid pattern
    let scale = (2.0 * PI) / (spacing * 4.0);

    // Number of z-slices to generate
    let num_slices = ((spacing * 2.0 / 3.0).ceil() as usize).max(3);

    // Z-offset based on angle
    let z_base = angle_degrees * PI / 180.0;

    let mut lines = Vec::new();

    // Generate contours at multiple z-levels
    for slice in 0..num_slices {
        let z = z_base + (slice as f64 / num_slices as f64) * PI;

        let slice_lines = contour_gyroid_slice(
            min_x, min_y, max_x, max_y,
            scale, z, spacing / 3.0,
        );

        lines.extend(slice_lines);
    }

    // Clip all lines to polygon boundary
    clip_lines_to_polygon(&lines, polygon)
}

/// Generate contour lines for a single z-slice of the gyroid.
/// Uses a simple marching squares approach.
fn contour_gyroid_slice(
    min_x: f64, min_y: f64, max_x: f64, max_y: f64,
    scale: f64, z: f64, resolution: f64,
) -> Vec<Line> {
    let mut lines = Vec::new();

    let width = max_x - min_x;
    let height = max_y - min_y;

    // Grid resolution
    let nx = ((width / resolution).ceil() as usize).max(10);
    let ny = ((height / resolution).ceil() as usize).max(10);

    let dx = width / nx as f64;
    let dy = height / ny as f64;

    // Evaluate gyroid on grid
    let mut grid: Vec<Vec<f64>> = vec![vec![0.0; ny + 1]; nx + 1];

    for i in 0..=nx {
        for j in 0..=ny {
            let x = min_x + i as f64 * dx;
            let y = min_y + j as f64 * dy;

            // Map to gyroid space
            let gx = x * scale;
            let gy = y * scale;

            grid[i][j] = gyroid_field(gx, gy, z);
        }
    }

    // Marching squares to find contours at value = 0
    for i in 0..nx {
        for j in 0..ny {
            let x0 = min_x + i as f64 * dx;
            let y0 = min_y + j as f64 * dy;
            let x1 = x0 + dx;
            let y1 = y0 + dy;

            let v00 = grid[i][j];
            let v10 = grid[i + 1][j];
            let v01 = grid[i][j + 1];
            let v11 = grid[i + 1][j + 1];

            // Classify cell by which corners are above/below zero
            let case = ((v00 > 0.0) as u8)
                     | (((v10 > 0.0) as u8) << 1)
                     | (((v01 > 0.0) as u8) << 2)
                     | (((v11 > 0.0) as u8) << 3);

            // Skip cells with no contour
            if case == 0 || case == 15 {
                continue;
            }

            // Linear interpolation to find contour crossing points
            let interp = |v1: f64, v2: f64, p1: f64, p2: f64| -> f64 {
                if (v2 - v1).abs() < 1e-10 {
                    (p1 + p2) / 2.0
                } else {
                    p1 + (p2 - p1) * (-v1) / (v2 - v1)
                }
            };

            // Edge midpoints where contour might cross
            let left   = (x0, interp(v00, v01, y0, y1));
            let right  = (x1, interp(v10, v11, y0, y1));
            let bottom = (interp(v00, v10, x0, x1), y0);
            let top    = (interp(v01, v11, x0, x1), y1);

            // Generate line segments based on case
            let segments: &[((f64, f64), (f64, f64))] = match case {
                1 | 14 => &[(left, bottom)],
                2 | 13 => &[(bottom, right)],
                3 | 12 => &[(left, right)],
                4 | 11 => &[(top, left)],
                5 | 10 => &[(left, bottom), (top, right)],  // Saddle
                6 | 9  => &[(bottom, top)],
                7 | 8  => &[(top, right)],
                _ => &[],
            };

            for &((px1, py1), (px2, py2)) in segments {
                lines.push(Line::new(px1, py1, px2, py2));
            }
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_gyroid_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_gyroid_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn gyroid_field_oscillates() {
        // Gyroid should oscillate between positive and negative
        let mut has_positive = false;
        let mut has_negative = false;

        for i in 0..10 {
            let x = i as f64 * 0.5;
            let val = gyroid_field(x, 0.0, 0.0);
            if val > 0.0 { has_positive = true; }
            if val < 0.0 { has_negative = true; }
        }

        assert!(has_positive && has_negative, "Gyroid should have both signs");
    }

    #[test]
    fn different_angles_give_different_patterns() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_a = generate_gyroid_fill(&poly, 15.0, 0.0);
        let lines_b = generate_gyroid_fill(&poly, 15.0, 90.0);

        assert!(!lines_a.is_empty());
        assert!(!lines_b.is_empty());

        // Different z-offsets should produce different patterns
        // (line counts or positions should differ)
        if !lines_a.is_empty() && !lines_b.is_empty() {
            let differs = lines_a.len() != lines_b.len() ||
                (lines_a[0].x1 - lines_b[0].x1).abs() > 0.1 ||
                (lines_a[0].y1 - lines_b[0].y1).abs() > 0.1;
            assert!(differs, "Different angles should produce different patterns");
        }
    }

    #[test]
    fn spacing_affects_pattern_scale() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let small_scale = generate_gyroid_fill(&poly, 5.0, 0.0);
        let large_scale = generate_gyroid_fill(&poly, 20.0, 0.0);

        // Smaller spacing = finer pattern = more lines
        assert!(small_scale.len() > large_scale.len());
    }

    #[test]
    fn gyroid_produces_curved_contours() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(200.0, 0.0),
            Point::new(200.0, 200.0),
            Point::new(0.0, 200.0),
        ]);

        let lines = generate_gyroid_fill(&poly, 20.0, 0.0);

        // Gyroid contours should be curved, meaning consecutive
        // line segments should have varying angles
        if lines.len() >= 3 {
            let mut has_angle_variation = false;
            for i in 1..lines.len().min(20) {
                let angle1 = (lines[i-1].y2 - lines[i-1].y1).atan2(lines[i-1].x2 - lines[i-1].x1);
                let angle2 = (lines[i].y2 - lines[i].y1).atan2(lines[i].x2 - lines[i].x1);
                if (angle1 - angle2).abs() > 0.1 {
                    has_angle_variation = true;
                    break;
                }
            }
            assert!(has_angle_variation, "Gyroid should have curved contours");
        }
    }
}
