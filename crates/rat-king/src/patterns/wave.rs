//! Wave interference fill pattern - overlapping sine waves.
//!
//! Creates patterns resembling wave interference, ripples, or moiré effects
//! by combining multiple sine waves from different origins.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate wave interference fill for a polygon.
///
/// Creates contour-like lines from interfering wave sources.
/// - `spacing`: Distance between contour lines
/// - `angle_degrees`: Controls the phase/position of wave sources
pub fn generate_wave_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let diagonal = (width * width + height * height).sqrt();

    // Wave sources positioned based on angle parameter
    let angle_rad = angle_degrees * PI / 180.0;
    let source_dist = diagonal * 0.4;

    let sources = vec![
        (center_x + source_dist * angle_rad.cos(), center_y + source_dist * angle_rad.sin()),
        (center_x + source_dist * (angle_rad + PI * 2.0/3.0).cos(),
         center_y + source_dist * (angle_rad + PI * 2.0/3.0).sin()),
        (center_x + source_dist * (angle_rad + PI * 4.0/3.0).cos(),
         center_y + source_dist * (angle_rad + PI * 4.0/3.0).sin()),
    ];

    // Wavelength based on spacing
    let wavelength = spacing * 2.0;

    // Generate contour lines using marching squares
    let resolution = (diagonal / (spacing * 0.5)).ceil() as usize;
    let step = diagonal * 1.2 / resolution as f64;

    // Sample wave field
    let mut field: Vec<Vec<f64>> = Vec::with_capacity(resolution + 1);
    let start_x = center_x - diagonal * 0.6;
    let start_y = center_y - diagonal * 0.6;

    for j in 0..=resolution {
        let mut row = Vec::with_capacity(resolution + 1);
        for i in 0..=resolution {
            let x = start_x + i as f64 * step;
            let y = start_y + j as f64 * step;

            // Sum wave contributions from all sources
            let mut value = 0.0;
            for &(sx, sy) in &sources {
                let dist = ((x - sx).powi(2) + (y - sy).powi(2)).sqrt();
                value += (dist * 2.0 * PI / wavelength).sin();
            }

            row.push(value);
        }
        field.push(row);
    }

    // Extract contour lines using marching squares
    let mut all_lines = Vec::new();
    let num_contours = 8; // Number of contour levels

    for level in 0..num_contours {
        let threshold = -2.5 + level as f64 * 5.0 / num_contours as f64;

        for j in 0..resolution {
            for i in 0..resolution {
                let lines = march_square(
                    i, j, &field, threshold,
                    start_x, start_y, step,
                );
                all_lines.extend(lines);
            }
        }
    }

    // Clip to polygon
    clip_lines_to_polygon(&all_lines, polygon)
}

/// Marching squares algorithm for a single cell.
fn march_square(
    i: usize, j: usize,
    field: &[Vec<f64>],
    threshold: f64,
    start_x: f64, start_y: f64,
    step: f64,
) -> Vec<Line> {
    let v00 = field[j][i] - threshold;
    let v10 = field[j][i + 1] - threshold;
    let v01 = field[j + 1][i] - threshold;
    let v11 = field[j + 1][i + 1] - threshold;

    // Cell corners
    let x0 = start_x + i as f64 * step;
    let x1 = start_x + (i + 1) as f64 * step;
    let y0 = start_y + j as f64 * step;
    let y1 = start_y + (j + 1) as f64 * step;

    // Classify cell (4 bits, one per corner)
    let config = ((v00 > 0.0) as u8)
        | (((v10 > 0.0) as u8) << 1)
        | (((v01 > 0.0) as u8) << 2)
        | (((v11 > 0.0) as u8) << 3);

    // Interpolate edge crossings
    let interp = |a: f64, b: f64, va: f64, vb: f64| -> f64 {
        if (va - vb).abs() < 1e-10 {
            (a + b) / 2.0
        } else {
            a + (b - a) * (-va) / (vb - va)
        }
    };

    // Edge midpoints (where contour crosses)
    let e_bottom = (interp(x0, x1, v00, v10), y0);
    let e_top = (interp(x0, x1, v01, v11), y1);
    let e_left = (x0, interp(y0, y1, v00, v01));
    let e_right = (x1, interp(y0, y1, v10, v11));

    let mut lines = Vec::new();

    match config {
        0 | 15 => {} // All same sign, no contour
        1 | 14 => lines.push(Line::new(e_bottom.0, e_bottom.1, e_left.0, e_left.1)),
        2 | 13 => lines.push(Line::new(e_bottom.0, e_bottom.1, e_right.0, e_right.1)),
        3 | 12 => lines.push(Line::new(e_left.0, e_left.1, e_right.0, e_right.1)),
        4 | 11 => lines.push(Line::new(e_left.0, e_left.1, e_top.0, e_top.1)),
        5 | 10 => {
            // Saddle case - draw both lines
            lines.push(Line::new(e_bottom.0, e_bottom.1, e_left.0, e_left.1));
            lines.push(Line::new(e_top.0, e_top.1, e_right.0, e_right.1));
        }
        6 | 9 => {
            // Another saddle case
            lines.push(Line::new(e_bottom.0, e_bottom.1, e_top.0, e_top.1));
        }
        7 | 8 => lines.push(Line::new(e_top.0, e_top.1, e_right.0, e_right.1)),
        _ => {}
    }

    lines
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
    fn generates_wave_lines() {
        let poly = square_polygon();
        let lines = generate_wave_fill(&poly, 5.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn wave_with_angle() {
        let poly = square_polygon();
        let lines = generate_wave_fill(&poly, 5.0, 60.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn wave_different_spacing() {
        let poly = square_polygon();
        let lines_wide = generate_wave_fill(&poly, 10.0, 0.0);
        let lines_narrow = generate_wave_fill(&poly, 3.0, 0.0);
        // More contours with narrower spacing
        assert!(lines_narrow.len() > lines_wide.len());
    }
}
