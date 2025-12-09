//! Sierpiński curve fill pattern.
//!
//! The Sierpiński curve is a space-filling curve based on the Sierpiński triangle.
//! It creates a distinctive triangular fractal pattern.
//!
//! Uses an L-system approach:
//!   Axiom: A
//!   Rules: A → B-A-B, B → A+B+A
//!   Where + = turn left 60°, - = turn right 60°

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate L-system string for Sierpiński curve.
fn generate_lsystem(depth: usize) -> String {
    let mut current = String::from("A");

    for _ in 0..depth {
        let mut next = String::with_capacity(current.len() * 3);
        for c in current.chars() {
            match c {
                'A' => next.push_str("B-A-B"),
                'B' => next.push_str("A+B+A"),
                c => next.push(c),
            }
        }
        current = next;
    }

    current
}

/// Execute L-system string as turtle graphics.
fn execute_turtle(
    commands: &str,
    start_x: f64,
    start_y: f64,
    step_size: f64,
    start_angle: f64,
) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut x = start_x;
    let mut y = start_y;
    let mut angle = start_angle;

    points.push((x, y));

    let turn_angle = PI / 3.0; // 60 degrees

    for c in commands.chars() {
        match c {
            'A' | 'B' => {
                // Move forward
                x += step_size * angle.cos();
                y += step_size * angle.sin();
                points.push((x, y));
            }
            '+' => {
                // Turn left 60°
                angle += turn_angle;
            }
            '-' => {
                // Turn right 60°
                angle -= turn_angle;
            }
            _ => {}
        }
    }

    points
}

/// Generate Sierpiński curve fill for a polygon.
///
/// Creates a triangular space-filling curve pattern.
pub fn generate_sierpinski_fill(
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
    // Sierpiński doubles segments each level
    let segments_needed = (size / spacing) as usize;
    let depth = (segments_needed as f64).log2().ceil() as usize;
    let depth = depth.max(1).min(8); // Limit for performance

    // Calculate step size to fill the area
    // Number of segments = 3^depth for Sierpiński arrowhead variant
    let num_segments = 3_usize.pow(depth as u32);
    let step_size = size / (num_segments as f64).sqrt();

    // Generate L-system commands
    let commands = generate_lsystem(depth);

    // Calculate starting position (lower left of bounding region)
    let start_x = center_x - size / 2.0;
    let start_y = center_y - size / 2.0;

    // Execute turtle graphics
    let points = execute_turtle(&commands, start_x, start_y, step_size, angle_rad);

    // Rotate points around center
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
    use crate::geometry::Point;

    #[test]
    fn generates_sierpinski_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_sierpinski_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate sierpinski lines");
    }

    #[test]
    fn lsystem_grows_correctly() {
        let depth1 = generate_lsystem(1);
        let depth2 = generate_lsystem(2);

        assert!(depth2.len() > depth1.len(), "Higher depth should produce longer string");
        assert!(depth1.contains("A") || depth1.contains("B"), "Should contain draw commands");
    }

    #[test]
    fn spacing_affects_detail() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_fine = generate_sierpinski_fill(&poly, 5.0, 0.0);
        let lines_coarse = generate_sierpinski_fill(&poly, 20.0, 0.0);

        assert!(lines_fine.len() >= lines_coarse.len(),
            "Smaller spacing should produce at least as many lines");
    }
}
