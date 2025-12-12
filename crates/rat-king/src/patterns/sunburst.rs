//! Sunburst fill pattern - radial rays emanating from centroid.
//!
//! Creates a pattern of rays or spokes radiating outward from the
//! polygon's centroid, with optional tapering and density control.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Generate sunburst fill for a polygon.
///
/// Creates radial rays from the polygon centroid.
/// - `spacing`: Controls the number of rays (smaller = more rays)
/// - `angle_degrees`: Rotation offset for the entire pattern
pub fn generate_sunburst_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();

    // Calculate polygon centroid
    let centroid = polygon_centroid(&polygon.outer);
    let cx = centroid.0;
    let cy = centroid.1;

    let base_angle = angle_degrees * PI / 180.0;

    // Number of rays based on spacing
    // Circumference at max radius / spacing = number of rays
    let max_radius = diagonal * 0.75;
    let num_rays = ((2.0 * PI * max_radius) / spacing).ceil() as i32;
    let num_rays = num_rays.clamp(8, 360);

    let angle_step = 2.0 * PI / num_rays as f64;

    let mut all_lines = Vec::new();

    for i in 0..num_rays {
        let angle = base_angle + i as f64 * angle_step;

        // Create ray from center to max radius
        let end_x = cx + max_radius * angle.cos();
        let end_y = cy + max_radius * angle.sin();

        all_lines.push(Line::new(cx, cy, end_x, end_y));

        // Add intermediate rays for alternating pattern
        if spacing < 15.0 {
            let half_angle = angle + angle_step / 2.0;
            let half_radius = max_radius * 0.6;
            let half_end_x = cx + half_radius * half_angle.cos();
            let half_end_y = cy + half_radius * half_angle.sin();
            all_lines.push(Line::new(cx, cy, half_end_x, half_end_y));
        }
    }

    // Add concentric circles for visual interest if dense enough
    if spacing < 10.0 {
        let ring_spacing = spacing * 3.0;
        let num_rings = (max_radius / ring_spacing) as i32;

        for r in 1..=num_rings {
            let radius = r as f64 * ring_spacing;
            let ring_lines = generate_circle_lines(cx, cy, radius, num_rays * 2);
            all_lines.extend(ring_lines);
        }
    }

    // Clip to polygon
    clip_lines_to_polygon(&all_lines, polygon)
}

/// Calculate the centroid of a polygon.
fn polygon_centroid(points: &[Point]) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }

    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut signed_area = 0.0;

    let n = points.len();
    for i in 0..n {
        let j = (i + 1) % n;
        let a = points[i].x * points[j].y - points[j].x * points[i].y;
        signed_area += a;
        cx += (points[i].x + points[j].x) * a;
        cy += (points[i].y + points[j].y) * a;
    }

    signed_area *= 0.5;

    if signed_area.abs() < 1e-10 {
        // Fallback to simple average for degenerate polygons
        let sum_x: f64 = points.iter().map(|p| p.x).sum();
        let sum_y: f64 = points.iter().map(|p| p.y).sum();
        return (sum_x / n as f64, sum_y / n as f64);
    }

    cx /= 6.0 * signed_area;
    cy /= 6.0 * signed_area;

    (cx, cy)
}

/// Generate line segments approximating a circle.
fn generate_circle_lines(cx: f64, cy: f64, radius: f64, segments: i32) -> Vec<Line> {
    let mut lines = Vec::new();
    let angle_step = 2.0 * PI / segments as f64;

    for i in 0..segments {
        let a1 = i as f64 * angle_step;
        let a2 = (i + 1) as f64 * angle_step;

        let x1 = cx + radius * a1.cos();
        let y1 = cy + radius * a1.sin();
        let x2 = cx + radius * a2.cos();
        let y2 = cy + radius * a2.sin();

        lines.push(Line::new(x1, y1, x2, y2));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_polygon() -> Polygon {
        Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ])
    }

    #[test]
    fn generates_sunburst_lines() {
        let poly = square_polygon();
        let lines = generate_sunburst_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn sunburst_with_rotation() {
        let poly = square_polygon();
        let lines = generate_sunburst_fill(&poly, 10.0, 45.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn sunburst_density_varies() {
        let poly = square_polygon();
        let lines_sparse = generate_sunburst_fill(&poly, 20.0, 0.0);
        let lines_dense = generate_sunburst_fill(&poly, 5.0, 0.0);
        // Denser spacing = more rays
        assert!(lines_dense.len() > lines_sparse.len());
    }

    #[test]
    fn centroid_calculation() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ];
        let (cx, cy) = polygon_centroid(&points);
        assert!((cx - 50.0).abs() < 0.1);
        assert!((cy - 50.0).abs() < 0.1);
    }
}
