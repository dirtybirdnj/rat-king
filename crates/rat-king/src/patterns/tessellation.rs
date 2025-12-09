//! Tessellation fill pattern.
//!
//! Divides polygons into triangles using ear clipping algorithm
//! and draws the triangle edges as lines.

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
}
