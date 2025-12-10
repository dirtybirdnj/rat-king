//! Tessellation fill pattern.
//!
//! Divides polygons into triangles using ear clipping algorithm
//! and draws the triangle edges as lines.
//!
//! This module also provides `fill_via_tessellation()` which can be used
//! by other patterns to efficiently fill complex polygons by first
//! tessellating into triangles, then filling each triangle.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};

/// Generate tessellation fill pattern for a polygon.
///
/// Triangulates the polygon using ear clipping and returns
/// the edges of all triangles as lines.
pub fn generate_tessellation_fill(
    polygon: &Polygon,
    _spacing: f64,
    _angle_degrees: f64,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    // Triangulate the polygon
    let triangles = triangulate_polygon(outer);

    // Convert triangles to lines (edges)
    let mut lines = Vec::new();
    for tri in &triangles {
        // Add all three edges of each triangle
        lines.push(Line::new(tri[0].x, tri[0].y, tri[1].x, tri[1].y));
        lines.push(Line::new(tri[1].x, tri[1].y, tri[2].x, tri[2].y));
        lines.push(Line::new(tri[2].x, tri[2].y, tri[0].x, tri[0].y));
    }

    lines
}

/// Triangulate a polygon using ear clipping algorithm.
/// Returns a list of triangles, each represented as 3 points.
fn triangulate_polygon(vertices: &[Point]) -> Vec<[Point; 3]> {
    let n = vertices.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[vertices[0], vertices[1], vertices[2]]];
    }

    let mut triangles = Vec::new();

    // Create a mutable list of vertex indices
    let mut indices: Vec<usize> = (0..n).collect();

    // Determine winding order (clockwise or counter-clockwise)
    let clockwise = is_clockwise(vertices);

    // Keep clipping ears until only 3 vertices remain
    while indices.len() > 3 {
        let len = indices.len();
        let mut ear_found = false;

        for i in 0..len {
            let prev = indices[(i + len - 1) % len];
            let curr = indices[i];
            let next = indices[(i + 1) % len];

            let a = vertices[prev];
            let b = vertices[curr];
            let c = vertices[next];

            // Check if this vertex forms an ear
            if is_ear(&a, &b, &c, &indices, vertices, clockwise) {
                // Add the triangle
                triangles.push([a, b, c]);

                // Remove the ear vertex
                indices.remove(i);
                ear_found = true;
                break;
            }
        }

        // Safety check: if no ear found, force remove a vertex
        // (can happen with degenerate polygons)
        if !ear_found {
            if indices.len() >= 3 {
                let a = vertices[indices[0]];
                let b = vertices[indices[1]];
                let c = vertices[indices[2]];
                triangles.push([a, b, c]);
                indices.remove(1);
            } else {
                break;
            }
        }
    }

    // Add the final triangle
    if indices.len() == 3 {
        triangles.push([
            vertices[indices[0]],
            vertices[indices[1]],
            vertices[indices[2]],
        ]);
    }

    triangles
}

/// Check if vertices are in clockwise order.
fn is_clockwise(vertices: &[Point]) -> bool {
    let mut sum = 0.0;
    let n = vertices.len();
    for i in 0..n {
        let p1 = &vertices[i];
        let p2 = &vertices[(i + 1) % n];
        sum += (p2.x - p1.x) * (p2.y + p1.y);
    }
    sum > 0.0
}

/// Check if vertex b forms an ear with adjacent vertices a and c.
fn is_ear(
    a: &Point,
    b: &Point,
    c: &Point,
    indices: &[usize],
    vertices: &[Point],
    clockwise: bool,
) -> bool {
    // Check if the angle at b is convex
    let cross = cross_product(a, b, c);
    let is_convex = if clockwise { cross >= 0.0 } else { cross <= 0.0 };

    if !is_convex {
        return false;
    }

    // Check that no other vertices are inside this triangle
    for &idx in indices {
        let p = &vertices[idx];
        // Skip the triangle vertices themselves
        if point_eq(p, a) || point_eq(p, b) || point_eq(p, c) {
            continue;
        }

        if point_in_triangle(p, a, b, c) {
            return false;
        }
    }

    true
}

/// Cross product of vectors (b-a) and (c-b).
fn cross_product(a: &Point, b: &Point, c: &Point) -> f64 {
    (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
}

/// Check if two points are approximately equal.
fn point_eq(p1: &Point, p2: &Point) -> bool {
    const EPSILON: f64 = 1e-10;
    (p1.x - p2.x).abs() < EPSILON && (p1.y - p2.y).abs() < EPSILON
}

/// Check if point p is inside triangle abc.
fn point_in_triangle(p: &Point, a: &Point, b: &Point, c: &Point) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);

    let has_neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

/// Sign of the cross product for point-in-triangle test.
fn sign(p1: &Point, p2: &Point, p3: &Point) -> f64 {
    (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
}

// ============================================================================
// TESSELLATION-BASED FILL (Performance Optimization)
// ============================================================================

/// Fill a polygon by first tessellating into triangles.
///
/// This is significantly faster for complex polygons because:
/// - Triangles are convex, so clipping is O(1) per line
/// - Lines are only generated within each triangle's bounding box
/// - No wasted computation on lines outside the polygon
///
/// Use this for patterns that would otherwise generate lines across
/// the entire bounding box and clip afterwards.
///
/// # Arguments
/// * `polygon` - The polygon to fill
/// * `spacing` - Distance between parallel fill lines
/// * `angle_degrees` - Angle of the fill lines
/// * `line_generator` - Function that generates fill lines for a triangle
///
/// # Example
/// ```ignore
/// let lines = fill_via_tessellation(polygon, 5.0, 45.0, |tri, spacing, angle| {
///     fill_triangle_with_lines(tri, spacing, angle)
/// });
/// ```
pub fn fill_via_tessellation<F>(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    line_generator: F,
) -> Vec<Line>
where
    F: Fn(&[Point; 3], f64, f64) -> Vec<Line>,
{
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    // Triangulate the polygon
    let triangles = triangulate_polygon(outer);

    // Fill each triangle and collect all lines
    let mut all_lines = Vec::new();
    for tri in &triangles {
        let tri_lines = line_generator(tri, spacing, angle_degrees);
        all_lines.extend(tri_lines);
    }

    all_lines
}

/// Fill a triangle with parallel lines.
///
/// This is optimized for triangles (convex, 3 edges) and avoids
/// the expensive general-case polygon clipping.
///
/// # Arguments
/// * `triangle` - Three points defining the triangle
/// * `spacing` - Distance between parallel lines
/// * `angle_degrees` - Angle of the fill lines
pub fn fill_triangle_with_lines(
    triangle: &[Point; 3],
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // Triangle bounding box
    let min_x = triangle[0].x.min(triangle[1].x).min(triangle[2].x);
    let max_x = triangle[0].x.max(triangle[1].x).max(triangle[2].x);
    let min_y = triangle[0].y.min(triangle[1].y).min(triangle[2].y);
    let max_y = triangle[0].y.max(triangle[1].y).max(triangle[2].y);

    let width = max_x - min_x;
    let height = max_y - min_y;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Diagonal of triangle bbox
    let diagonal = (width * width + height * height).sqrt();

    let angle_rad = angle_degrees * PI / 180.0;
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Direction along lines and perpendicular
    let dir_x = cos_a;
    let dir_y = sin_a;
    let perp_x = -sin_a;
    let perp_y = cos_a;

    let half_diag = diagonal / 2.0 + spacing;
    let num_lines = (diagonal / spacing).ceil() as i32 + 1;

    let mut lines = Vec::new();

    // Triangle edges for intersection tests
    let edges = [
        (triangle[0], triangle[1]),
        (triangle[1], triangle[2]),
        (triangle[2], triangle[0]),
    ];

    for i in -num_lines..=num_lines {
        let offset = i as f64 * spacing;

        // Line origin offset perpendicular to line direction
        let ox = center_x + perp_x * offset;
        let oy = center_y + perp_y * offset;

        // Line extends in both directions
        let lx1 = ox - dir_x * half_diag;
        let ly1 = oy - dir_y * half_diag;
        let lx2 = ox + dir_x * half_diag;
        let ly2 = oy + dir_y * half_diag;

        // Find intersections with triangle edges
        let mut intersections: Vec<(f64, f64, f64)> = Vec::with_capacity(2);

        for (p1, p2) in &edges {
            if let Some((x, y, t)) = line_segment_intersect(
                lx1, ly1, lx2, ly2,
                p1.x, p1.y, p2.x, p2.y,
            ) {
                intersections.push((x, y, t));
            }
        }

        // Need exactly 2 intersections to form a line segment
        if intersections.len() >= 2 {
            // Sort by t parameter
            intersections.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            let (x1, y1, _) = intersections[0];
            let (x2, y2, _) = intersections[intersections.len() - 1];

            // Skip degenerate lines
            let len_sq = (x2 - x1).powi(2) + (y2 - y1).powi(2);
            if len_sq > 1e-6 {
                lines.push(Line::new(x1, y1, x2, y2));
            }
        }
    }

    lines
}

/// Line segment intersection for triangle clipping.
///
/// Returns Some((x, y, t)) if segments intersect, where t is the
/// parameter along the first line (0..1).
#[inline]
fn line_segment_intersect(
    x1: f64, y1: f64, x2: f64, y2: f64,
    x3: f64, y3: f64, x4: f64, y4: f64,
) -> Option<(f64, f64, f64)> {
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);

    if denom.abs() < 1e-10 {
        return None; // Parallel
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
    let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

    // Check if intersection is within both segments
    if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
        let x = x1 + ua * (x2 - x1);
        let y = y1 + ua * (y2 - y1);
        Some((x, y, ua))
    } else {
        None
    }
}

/// Fill polygon with parallel lines using tessellation for speed.
///
/// This is the main entry point for patterns that want fast parallel line fill.
/// It tessellates the polygon into triangles and fills each one.
pub fn fill_polygon_with_lines_fast(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    fill_via_tessellation(polygon, spacing, angle_degrees, fill_triangle_with_lines)
}

/// Make the triangulate_polygon function public for use by other modules.
pub fn triangulate(vertices: &[Point]) -> Vec<[Point; 3]> {
    triangulate_polygon(vertices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulates_square() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_tessellation_fill(&poly, 10.0, 0.0);
        // Square -> 2 triangles -> 6 edges (some may overlap with polygon edges)
        assert!(!lines.is_empty(), "Should generate tessellation lines");
        // 2 triangles * 3 edges = 6 lines
        assert_eq!(lines.len(), 6, "Square should produce 2 triangles (6 edges)");
    }

    #[test]
    fn triangulates_triangle() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(50.0, 100.0),
        ]);
        let lines = generate_tessellation_fill(&poly, 10.0, 0.0);
        // Triangle -> 1 triangle -> 3 edges
        assert_eq!(lines.len(), 3, "Triangle should produce 1 triangle (3 edges)");
    }

    #[test]
    fn triangulates_pentagon() {
        let poly = Polygon::new(vec![
            Point::new(50.0, 0.0),
            Point::new(100.0, 38.0),
            Point::new(81.0, 100.0),
            Point::new(19.0, 100.0),
            Point::new(0.0, 38.0),
        ]);
        let lines = generate_tessellation_fill(&poly, 10.0, 0.0);
        // Pentagon -> 3 triangles -> 9 edges
        assert_eq!(lines.len(), 9, "Pentagon should produce 3 triangles (9 edges)");
    }

    #[test]
    fn handles_concave_polygon() {
        // L-shaped polygon (concave)
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 50.0),
            Point::new(50.0, 50.0),
            Point::new(50.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_tessellation_fill(&poly, 10.0, 0.0);
        // 6-gon -> 4 triangles -> 12 edges
        assert!(!lines.is_empty(), "Should handle concave polygon");
        assert_eq!(lines.len(), 12, "L-shape should produce 4 triangles (12 edges)");
    }

    #[test]
    fn fill_triangle_generates_lines() {
        let tri = [
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(50.0, 86.6), // Equilateral triangle
        ];
        let lines = fill_triangle_with_lines(&tri, 10.0, 0.0);

        assert!(!lines.is_empty(), "Should generate fill lines");
        // At 10px spacing in ~86px height, expect ~8-9 lines
        assert!(lines.len() >= 5, "Should have reasonable number of lines");
        assert!(lines.len() <= 15, "Should not have too many lines");
    }

    #[test]
    fn fill_triangle_respects_angle() {
        let tri = [
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(50.0, 86.6),
        ];

        let lines_0 = fill_triangle_with_lines(&tri, 10.0, 0.0);
        let lines_45 = fill_triangle_with_lines(&tri, 10.0, 45.0);

        assert!(!lines_0.is_empty());
        assert!(!lines_45.is_empty());

        // Lines at different angles should have different coordinates
        // (unless by coincidence, which is unlikely)
        let coords_differ = lines_0.iter().zip(lines_45.iter()).any(|(a, b)| {
            (a.x1 - b.x1).abs() > 1.0 || (a.y1 - b.y1).abs() > 1.0
        });
        assert!(coords_differ, "Different angles should produce different line coordinates");
    }

    #[test]
    fn fill_polygon_with_lines_fast_works() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines = fill_polygon_with_lines_fast(&poly, 10.0, 45.0);

        assert!(!lines.is_empty(), "Should generate lines");
        // 100x100 square at 10px spacing, diagonal ~141px
        // Expect roughly 14 lines per triangle * 2 triangles = ~20-30 lines
        assert!(lines.len() >= 10, "Should have reasonable line count");
    }

    #[test]
    fn fill_via_tessellation_custom_generator() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        // Custom generator that just draws triangle edges
        let lines = fill_via_tessellation(&poly, 10.0, 0.0, |tri, _, _| {
            vec![
                Line::new(tri[0].x, tri[0].y, tri[1].x, tri[1].y),
                Line::new(tri[1].x, tri[1].y, tri[2].x, tri[2].y),
                Line::new(tri[2].x, tri[2].y, tri[0].x, tri[0].y),
            ]
        });

        // Square -> 2 triangles -> 6 edges
        assert_eq!(lines.len(), 6);
    }
}
