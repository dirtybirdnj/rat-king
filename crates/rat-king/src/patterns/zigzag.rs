//! Zigzag fill pattern - parallel zigzag lines.
//!
//! The optimized version uses tessellation to avoid generating lines
//! outside the polygon boundary.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;
use super::tessellation::triangulate;

/// Generate zigzag fill for a polygon.
///
/// Creates parallel rows of sharp zigzag lines.
/// Uses tessellation for O(n) performance instead of O(n²).
pub fn generate_zigzag_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    amplitude: f64,
) -> Vec<Line> {
    // Use tessellation-based fill for complex polygons (>6 vertices)
    if polygon.outer.len() > 6 {
        return generate_zigzag_fill_tessellated(polygon, spacing, angle_degrees, amplitude);
    }

    // For simple polygons, use the direct approach
    generate_zigzag_fill_direct(polygon, spacing, angle_degrees, amplitude)
}

/// Direct zigzag fill - generates all lines then clips.
/// Used for simple polygons where tessellation overhead isn't worth it.
fn generate_zigzag_fill_direct(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    amplitude: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let angle_rad = angle_degrees * PI / 180.0;

    // Diagonal coverage
    let diagonal = (width * width + height * height).sqrt() * 1.42;

    // Direction vectors
    let perp_x = (angle_rad + PI / 2.0).cos();
    let perp_y = (angle_rad + PI / 2.0).sin();
    let dir_x = angle_rad.cos();
    let dir_y = angle_rad.sin();

    let center_x = min_x + width / 2.0;
    let center_y = min_y + height / 2.0;
    let num_rows = (diagonal / spacing).ceil() as i32 + 1;

    // Zigzag wavelength - distance between peaks
    let wavelength = amplitude * 2.0;
    let num_segments = (diagonal / wavelength) as i32 + 2;

    let mut lines = Vec::new();

    for i in -num_rows..=num_rows {
        let offset = i as f64 * spacing;
        let row_center_x = center_x + perp_x * offset;
        let row_center_y = center_y + perp_y * offset;

        for j in -num_segments..num_segments {
            let t1 = j as f64 * wavelength;
            let t2 = (j as f64 + 0.5) * wavelength;

            // Alternate amplitude direction
            let amp1 = amplitude * if j % 2 == 0 { 1.0 } else { -1.0 };
            let amp2 = -amp1;

            let x1 = row_center_x + dir_x * t1 + perp_x * amp1;
            let y1 = row_center_y + dir_y * t1 + perp_y * amp1;
            let x2 = row_center_x + dir_x * t2 + perp_x * amp2;
            let y2 = row_center_y + dir_y * t2 + perp_y * amp2;

            lines.push(Line::new(x1, y1, x2, y2));
        }
    }

    clip_lines_to_polygon(&lines, polygon)
}

/// Tessellation-based zigzag fill - much faster for complex polygons.
fn generate_zigzag_fill_tessellated(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    amplitude: f64,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let angle_rad = angle_degrees * PI / 180.0;
    let perp_x = (angle_rad + PI / 2.0).cos();
    let perp_y = (angle_rad + PI / 2.0).sin();
    let dir_x = angle_rad.cos();
    let dir_y = angle_rad.sin();

    // Zigzag wavelength
    let wavelength = amplitude * 2.0;

    // Triangulate the polygon
    let triangles = triangulate(outer);

    let mut all_lines = Vec::new();

    for tri in &triangles {
        // Triangle bounding box
        let tri_min_x = tri[0].x.min(tri[1].x).min(tri[2].x);
        let tri_max_x = tri[0].x.max(tri[1].x).max(tri[2].x);
        let tri_min_y = tri[0].y.min(tri[1].y).min(tri[2].y);
        let tri_max_y = tri[0].y.max(tri[1].y).max(tri[2].y);

        let tri_width = tri_max_x - tri_min_x;
        let tri_height = tri_max_y - tri_min_y;
        let tri_diagonal = (tri_width * tri_width + tri_height * tri_height).sqrt();
        let tri_center_x = (tri_min_x + tri_max_x) / 2.0;
        let tri_center_y = (tri_min_y + tri_max_y) / 2.0;

        let num_lines = (tri_diagonal / spacing).ceil() as i32 + 2;
        let num_segments = (tri_diagonal / wavelength).ceil() as i32 + 2;

        // Triangle edges for clipping
        let edges = [
            (tri[0], tri[1]),
            (tri[1], tri[2]),
            (tri[2], tri[0]),
        ];

        for i in -num_lines..=num_lines {
            let offset = i as f64 * spacing;
            let row_center_x = tri_center_x + perp_x * offset;
            let row_center_y = tri_center_y + perp_y * offset;

            for j in -num_segments..=num_segments {
                let t1 = j as f64 * wavelength;
                let t2 = (j as f64 + 0.5) * wavelength;

                // Alternate amplitude direction
                let amp1 = amplitude * if j % 2 == 0 { 1.0 } else { -1.0 };
                let amp2 = -amp1;

                let x1 = row_center_x + dir_x * t1 + perp_x * amp1;
                let y1 = row_center_y + dir_y * t1 + perp_y * amp1;
                let x2 = row_center_x + dir_x * t2 + perp_x * amp2;
                let y2 = row_center_y + dir_y * t2 + perp_y * amp2;

                // Quick bounding box rejection
                let seg_min_x = x1.min(x2);
                let seg_max_x = x1.max(x2);
                let seg_min_y = y1.min(y2);
                let seg_max_y = y1.max(y2);

                if seg_max_x < tri_min_x || seg_min_x > tri_max_x ||
                   seg_max_y < tri_min_y || seg_min_y > tri_max_y {
                    continue;
                }

                // Clip line segment to triangle
                if let Some(clipped) = clip_line_to_triangle(x1, y1, x2, y2, &edges) {
                    all_lines.push(clipped);
                }
            }
        }
    }

    all_lines
}

/// Clip a line segment to a triangle.
fn clip_line_to_triangle(
    x1: f64, y1: f64, x2: f64, y2: f64,
    edges: &[(Point, Point); 3],
) -> Option<Line> {
    let p1_inside = point_in_triangle(x1, y1, edges);
    let p2_inside = point_in_triangle(x2, y2, edges);

    if p1_inside && p2_inside {
        return Some(Line::new(x1, y1, x2, y2));
    }

    let mut intersections: Vec<(f64, f64, f64)> = Vec::with_capacity(2);

    for (p1, p2) in edges {
        if let Some((ix, iy, t)) = line_segment_intersect(
            x1, y1, x2, y2,
            p1.x, p1.y, p2.x, p2.y,
        ) {
            intersections.push((ix, iy, t));
        }
    }

    let mut points: Vec<(f64, f64, f64)> = Vec::new();

    if p1_inside {
        points.push((x1, y1, 0.0));
    }
    points.extend(&intersections);
    if p2_inside {
        points.push((x2, y2, 1.0));
    }

    if points.len() < 2 {
        return None;
    }

    points.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let (ax, ay, _) = points[0];
    let (bx, by, _) = points[points.len() - 1];

    let mid_x = (ax + bx) / 2.0;
    let mid_y = (ay + by) / 2.0;
    if !point_in_triangle(mid_x, mid_y, edges) {
        return None;
    }

    let len_sq = (bx - ax).powi(2) + (by - ay).powi(2);
    if len_sq < 1e-6 {
        return None;
    }

    Some(Line::new(ax, ay, bx, by))
}

#[inline]
fn point_in_triangle(px: f64, py: f64, edges: &[(Point, Point); 3]) -> bool {
    let (a, b) = &edges[0];
    let (_, c) = &edges[1];

    let d1 = sign_tri(px, py, a.x, a.y, b.x, b.y);
    let d2 = sign_tri(px, py, b.x, b.y, c.x, c.y);
    let d3 = sign_tri(px, py, c.x, c.y, a.x, a.y);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

#[inline]
fn sign_tri(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (px - x2) * (y1 - y2) - (x1 - x2) * (py - y2)
}

#[inline]
fn line_segment_intersect(
    x1: f64, y1: f64, x2: f64, y2: f64,
    x3: f64, y3: f64, x4: f64, y4: f64,
) -> Option<(f64, f64, f64)> {
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);

    if denom.abs() < 1e-10 {
        return None;
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
    let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

    if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
        let x = x1 + ua * (x2 - x1);
        let y = y1 + ua * (y2 - y1);
        Some((x, y, ua))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_zigzag_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 5.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn zigzag_complex_polygon() {
        // L-shaped polygon (concave, 6 vertices)
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 50.0),
            Point::new(50.0, 50.0),
            Point::new(50.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_zigzag_fill(&poly, 10.0, 45.0, 5.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn zigzag_star_polygon() {
        // Star shape (10 vertices - will use tessellation)
        let mut points = Vec::new();
        for i in 0..10 {
            let angle = std::f64::consts::PI * 2.0 * i as f64 / 10.0;
            let r = if i % 2 == 0 { 100.0 } else { 50.0 };
            points.push(Point::new(
                150.0 + r * angle.cos(),
                150.0 + r * angle.sin(),
            ));
        }
        let poly = Polygon::new(points);
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 5.0);
        assert!(!lines.is_empty());
    }
}
