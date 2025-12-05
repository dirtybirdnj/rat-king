//! Line clipping algorithms for polygon intersection.
//!
//! This is the HOT PATH - the code that runs millions of times.
//! Every microsecond saved here matters!

use crate::geometry::{Line, Point, Polygon};

// ============================================================================
// POINT IN POLYGON (Ray Casting Algorithm)
// ============================================================================
//
// ## Rust Lesson #8: References & Slices
//
// `&[Point]` is a "slice" - a borrowed view into a contiguous sequence.
// It works with Vec<Point>, arrays, or any contiguous memory.
//
// Think of it like: function takes a read-only window into your array.
// The `&` means borrowed - we don't take ownership, just look at it.

/// Test if a point is inside a polygon using ray casting.
///
/// Casts a ray to the right and counts edge crossings.
/// Odd crossings = inside, even = outside.
#[inline]
pub fn point_in_polygon(px: f64, py: f64, polygon: &[Point]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = n - 1;

    // ## Rust Lesson #9: Iterators vs Indexing
    //
    // We could use `for i in 0..n` (range iterator)
    // But here we need both current and previous vertex.
    // Classic loop with indices is fine for this.

    for i in 0..n {
        let (xi, yi) = (polygon[i].x, polygon[i].y);
        let (xj, yj) = (polygon[j].x, polygon[j].y);

        // Ray casting: check if horizontal ray from (px, py) crosses this edge
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }

        j = i;
    }

    inside
}

// ============================================================================
// LINE-LINE INTERSECTION
// ============================================================================

/// Result of a line-line intersection test.
///
/// ## Rust Lesson #10: Enums (Sum Types)
///
/// Unlike TypeScript's union types, Rust enums can carry data!
/// This is called a "sum type" or "tagged union".
///
/// ```text
/// // TypeScript:
/// type Result = { intersects: false } | { intersects: true, x: number, y: number }
///
/// // Rust:
/// enum Intersection { None, Point { x: f64, y: f64 } }
/// ```
///
/// The compiler ensures you handle ALL variants (exhaustive matching).
#[derive(Debug, Clone, Copy)]
pub enum Intersection {
    None,
    Point { x: f64, y: f64, t: f64 },
}

/// Find intersection point between two line segments.
///
/// Returns the intersection point and `t` parameter (0..1 means on first segment).
#[inline]
pub fn line_segment_intersection(
    x1: f64, y1: f64, x2: f64, y2: f64,
    x3: f64, y3: f64, x4: f64, y4: f64,
) -> Intersection {
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);

    // Parallel or coincident lines
    if denom.abs() < 1e-10 {
        return Intersection::None;
    }

    let ua = ((x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3)) / denom;
    let ub = ((x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3)) / denom;

    // Check if intersection is within both segments
    if (0.0..=1.0).contains(&ua) && (0.0..=1.0).contains(&ub) {
        let ix = x1 + ua * (x2 - x1);
        let iy = y1 + ua * (y2 - y1);
        Intersection::Point { x: ix, y: iy, t: ua }
    } else {
        Intersection::None
    }
}

// ============================================================================
// LINE-POLYGON CLIPPING
// ============================================================================

/// Find all intersections between a line and a polygon boundary.
///
/// Returns intersection points sorted by parameter `t` along the line.
pub fn line_polygon_intersections(
    lx1: f64, ly1: f64, lx2: f64, ly2: f64,
    polygon: &[Point],
) -> Vec<(f64, f64, f64)> {
    let n = polygon.len();
    if n < 3 {
        return Vec::new();
    }

    // ## Rust Lesson #11: Capacity Hints
    //
    // Vec::with_capacity(n) pre-allocates memory.
    // Like new Array(n) in JS but smarter - doesn't initialize elements.
    // Avoids reallocations as we push items.

    let mut intersections = Vec::with_capacity(n / 2);

    let dx = lx2 - lx1;
    let dy = ly2 - ly1;

    for i in 0..n {
        let j = (i + 1) % n;
        let (x3, y3) = (polygon[i].x, polygon[i].y);
        let (x4, y4) = (polygon[j].x, polygon[j].y);

        // ## Rust Lesson #12: Pattern Matching
        //
        // `if let` is like destructuring assignment but for enums.
        // Only executes if the pattern matches.
        //
        // Full version: match result { Point{x,y,t} => ..., None => ... }

        if let Intersection::Point { x, y, t: _ } = line_segment_intersection(
            lx1, ly1, lx2, ly2,
            x3, y3, x4, y4,
        ) {
            // Calculate t parameter for sorting
            let t = if dx.abs() > dy.abs() {
                (x - lx1) / dx
            } else if dy != 0.0 {
                (y - ly1) / dy
            } else {
                0.0
            };
            intersections.push((x, y, t));
        }
    }

    // Sort by t parameter (position along line)
    intersections.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    intersections
}

/// Clip a line segment to a polygon (outer boundary only).
///
/// Returns a list of line segments that lie inside the polygon.
pub fn clip_line_to_polygon(line: Line, polygon: &Polygon) -> Vec<Line> {
    // Fast bounding box rejection
    if let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() {
        let line_min_x = line.x1.min(line.x2);
        let line_max_x = line.x1.max(line.x2);
        let line_min_y = line.y1.min(line.y2);
        let line_max_y = line.y1.max(line.y2);

        if line_max_x < min_x || line_min_x > max_x ||
           line_max_y < min_y || line_min_y > max_y {
            return Vec::new();
        }
    }

    let p1_inside = point_in_polygon(line.x1, line.y1, &polygon.outer);
    let p2_inside = point_in_polygon(line.x2, line.y2, &polygon.outer);

    let intersections = line_polygon_intersections(
        line.x1, line.y1, line.x2, line.y2,
        &polygon.outer,
    );

    // ## Rust Lesson #13: Match Expressions
    //
    // match is like switch but:
    // - Must be exhaustive (handle all cases)
    // - Can match on patterns, not just values
    // - Returns a value (it's an expression!)

    match intersections.len() {
        0 => {
            // No intersections - entirely inside or outside
            if p1_inside && p2_inside {
                vec![line]
            } else {
                Vec::new()
            }
        }
        1 => {
            // One intersection - enters or exits
            let (ix, iy, _) = intersections[0];
            if p1_inside {
                vec![Line::new(line.x1, line.y1, ix, iy)]
            } else if p2_inside {
                vec![Line::new(ix, iy, line.x2, line.y2)]
            } else {
                Vec::new()
            }
        }
        _ => {
            // Multiple intersections - build segments from sorted points
            clip_line_multiple_intersections(
                line, p1_inside, p2_inside, &intersections, &polygon.outer
            )
        }
    }
}

/// Handle the complex case of multiple intersections.
fn clip_line_multiple_intersections(
    line: Line,
    p1_inside: bool,
    p2_inside: bool,
    intersections: &[(f64, f64, f64)],
    polygon: &[Point],
) -> Vec<Line> {
    // Build all points along the line
    let mut points: Vec<(f64, f64, f64)> = Vec::with_capacity(intersections.len() + 2);

    if p1_inside {
        points.push((line.x1, line.y1, 0.0));
    }

    points.extend(intersections.iter().copied());

    if p2_inside {
        points.push((line.x2, line.y2, 1.0));
    }

    // Sort by t (should already be sorted, but ensure it)
    points.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

    // ## Rust Lesson #14: Iterators & Collecting
    //
    // .windows(2) gives sliding windows of size 2: [a,b], [b,c], [c,d], ...
    // .filter_map() combines filter + map: return Some(x) to keep, None to skip
    // .collect() gathers results into a Vec

    points
        .windows(2)
        .filter_map(|pair| {
            let (x1, y1, _) = pair[0];
            let (x2, y2, _) = pair[1];

            // Keep segment if midpoint is inside polygon
            let mid_x = (x1 + x2) / 2.0;
            let mid_y = (y1 + y2) / 2.0;

            if point_in_polygon(mid_x, mid_y, polygon) {
                Some(Line::new(x1, y1, x2, y2))
            } else {
                None
            }
        })
        .collect()
}

/// Clip a line to a polygon with holes.
///
/// Lines inside holes are excluded.
pub fn clip_line_to_polygon_with_holes(line: Line, polygon: &Polygon) -> Vec<Line> {
    // First clip to outer boundary
    let mut segments = clip_line_to_polygon(line, polygon);

    // Then exclude segments inside any hole
    for hole in &polygon.holes {
        if hole.len() < 3 {
            continue;
        }

        // ## Rust Lesson #15: Retain
        //
        // .retain() is like .filter() but modifies in-place.
        // More efficient than creating new Vec when removing items.

        segments.retain(|seg| {
            let mid = seg.midpoint();
            !point_in_polygon(mid.x, mid.y, hole)
        });
    }

    segments
}

// ============================================================================
// BATCH OPERATIONS (for performance)
// ============================================================================

/// Clip multiple lines to a polygon.
///
/// This is the main entry point for hatch line clipping.
pub fn clip_lines_to_polygon(lines: &[Line], polygon: &Polygon) -> Vec<Line> {
    // ## Rust Lesson #16: flat_map
    //
    // For each input line, we get 0 or more output segments.
    // flat_map flattens: [[a,b], [c], [d,e,f]] -> [a,b,c,d,e,f]
    //
    // This is LAZY - nothing happens until .collect().
    // Zero intermediate allocations!

    lines
        .iter()
        .flat_map(|line| clip_line_to_polygon_with_holes(*line, polygon))
        .collect()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Polygon {
        Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ])
    }

    #[test]
    fn point_inside_square() {
        let sq = square();
        assert!(point_in_polygon(5.0, 5.0, &sq.outer));
        assert!(!point_in_polygon(15.0, 5.0, &sq.outer));
        assert!(!point_in_polygon(-1.0, 5.0, &sq.outer));
    }

    #[test]
    fn line_entirely_inside() {
        let sq = square();
        let line = Line::new(2.0, 5.0, 8.0, 5.0);
        let result = clip_line_to_polygon(line, &sq);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], line);
    }

    #[test]
    fn line_entirely_outside() {
        let sq = square();
        let line = Line::new(15.0, 5.0, 20.0, 5.0);
        let result = clip_line_to_polygon(line, &sq);
        assert!(result.is_empty());
    }

    #[test]
    fn line_crosses_polygon() {
        let sq = square();
        let line = Line::new(-5.0, 5.0, 15.0, 5.0);
        let result = clip_line_to_polygon(line, &sq);
        assert_eq!(result.len(), 1);
        // Should be clipped to [0, 5] -> [10, 5]
        assert!((result[0].x1 - 0.0).abs() < 1e-10);
        assert!((result[0].x2 - 10.0).abs() < 1e-10);
    }

    #[test]
    fn line_segment_intersection_test() {
        // Crossing lines
        let result = line_segment_intersection(
            0.0, 0.0, 10.0, 10.0,
            0.0, 10.0, 10.0, 0.0,
        );
        if let Intersection::Point { x, y, .. } = result {
            assert!((x - 5.0).abs() < 1e-10);
            assert!((y - 5.0).abs() < 1e-10);
        } else {
            panic!("Expected intersection");
        }

        // Parallel lines
        let result = line_segment_intersection(
            0.0, 0.0, 10.0, 0.0,
            0.0, 5.0, 10.0, 5.0,
        );
        assert!(matches!(result, Intersection::None));
    }
}
