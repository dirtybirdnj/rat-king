//! SVG parsing - extract polygons from SVG files.
//!
//! Uses usvg for complete SVG resolution (CSS, transforms, etc.)
//! then walks the tree to extract path data as polygons.
//!
//! ## Curve Flattening
//!
//! SVG paths contain Bézier curves (cubic and quadratic). These must be
//! "flattened" into line segments for polygon operations. We use lyon_geom
//! for accurate curve approximation with a configurable tolerance.

use crate::clip::point_in_polygon;
use crate::geometry::{Point, Polygon};
use lyon_geom::{CubicBezierSegment, QuadraticBezierSegment, point};

/// Error type for SVG parsing.
///
/// ## Rust Lesson #20: Error Handling
///
/// Rust uses `Result<T, E>` instead of exceptions:
/// - `Ok(value)` = success
/// - `Err(error)` = failure
///
/// You MUST handle errors - the compiler won't let you ignore them!
/// Common patterns:
/// - `?` operator: early return on error
/// - `.unwrap()`: panic on error (only use in tests!)
/// - `match`: handle each case explicitly
#[derive(Debug)]
pub enum SvgError {
    ParseError(String),
    NoPolygons,
}

impl std::fmt::Display for SvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SvgError::ParseError(msg) => write!(f, "SVG parse error: {}", msg),
            SvgError::NoPolygons => write!(f, "No polygons found in SVG"),
        }
    }
}

// Makes our error type work with the standard error trait
impl std::error::Error for SvgError {}

/// Extract all polygons from an SVG file.
///
/// ## Rust Lesson #21: The ? Operator
///
/// `expression?` is sugar for:
/// ```text
/// match expression {
///     Ok(v) => v,
///     Err(e) => return Err(e.into()),
/// }
/// ```
/// It "bubbles up" errors automatically!
pub fn extract_polygons_from_svg(svg_content: &str) -> Result<Vec<Polygon>, SvgError> {
    // Parse SVG using usvg (resolves CSS, transforms, etc.)
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_content, &options)
        .map_err(|e| SvgError::ParseError(e.to_string()))?;

    let mut polygons = Vec::new();

    // Walk the tree and collect paths (root is a Group in usvg 0.45)
    extract_from_group(tree.root(), &mut polygons);

    if polygons.is_empty() {
        Err(SvgError::NoPolygons)
    } else {
        Ok(polygons)
    }
}

/// Recursively extract polygons from a usvg Group.
fn extract_from_group(group: &usvg::Group, polygons: &mut Vec<Polygon>) {
    for child in group.children() {
        extract_from_node(child, polygons);
    }
}

/// Recursively extract polygons from a usvg node.
fn extract_from_node(node: &usvg::Node, polygons: &mut Vec<Polygon>) {
    // ## Rust Lesson #22: Pattern Matching on Enums with Data
    //
    // usvg::Node is an enum with variants that carry different data.
    // We match on the variant and destructure to get the inner data.

    match node {
        usvg::Node::Group(group) => {
            // Recurse into groups
            extract_from_group(group, polygons);
        }
        usvg::Node::Path(path) => {
            // Extract all polygons from path data (handles compound paths)
            polygons.extend(path_to_polygons(path));
        }
        // Ignore text, images, etc.
        _ => {}
    }
}

/// Tolerance for curve flattening.
/// Lower = more points, smoother curves, slower.
/// 0.1 is good for plotters (sub-pixel accuracy at typical scales).
const CURVE_TOLERANCE: f32 = 0.1;

/// Convert a usvg path to polygons.
///
/// A single SVG path can contain multiple subpaths (separated by MoveTo commands).
/// Each subpath becomes a separate polygon. This properly handles compound paths
/// like stamps with multiple circles or text with multiple characters.
///
/// Properly flattens Bézier curves using lyon_geom for accurate polygon boundaries.
/// Applies the path's absolute transform (including all parent group transforms).
fn path_to_polygons(path: &usvg::Path) -> Vec<Polygon> {
    let data = path.data();
    let id = path.id();
    let transform = path.abs_transform();

    // ## Rust Lesson #23: Iterator Peekable & Adapters
    //
    // We need to parse SVG path commands (M, L, C, Z, etc.)
    // usvg already gives us absolute coordinates (no relative commands!)

    let mut polygons = Vec::new();
    let mut points = Vec::new();
    let mut last_point: Option<(f32, f32)> = None;
    let mut subpath_index = 0;

    // Helper to apply transform to a point and create a Point
    let transform_point = |x: f32, y: f32| {
        let mut pt = usvg::tiny_skia_path::Point { x, y };
        transform.map_point(&mut pt);
        Point::new(pt.x as f64, pt.y as f64)
    };

    // Helper to finalize current subpath as a polygon
    let finalize_subpath = |points: &mut Vec<Point>, subpath_idx: usize| {
        // Remove duplicate consecutive points that can occur from curve flattening
        if points.len() >= 2 {
            points.dedup_by(|a, b| {
                let dx = (a.x - b.x).abs();
                let dy = (a.y - b.y).abs();
                dx < 1e-6 && dy < 1e-6
            });
        }

        if points.len() >= 3 {
            // Preserve the element's ID, appending subpath index for compound paths
            let polygon_id = if id.is_empty() {
                None
            } else if subpath_idx == 0 {
                Some(id.to_string())
            } else {
                Some(format!("{}_{}", id, subpath_idx))
            };
            let polygon = Polygon::with_id(std::mem::take(points), polygon_id);
            return Some(polygon);
        }
        points.clear();
        None
    };

    for cmd in data.segments() {
        match cmd {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                // Start of new subpath - finalize previous if any
                if !points.is_empty() {
                    if let Some(polygon) = finalize_subpath(&mut points, subpath_index) {
                        polygons.push(polygon);
                    }
                    subpath_index += 1;
                }
                points.push(transform_point(p.x, p.y));
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                points.push(transform_point(p.x, p.y));
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(ctrl, p) => {
                // Properly flatten quadratic Bézier curve
                if let Some((lx, ly)) = last_point {
                    let curve = QuadraticBezierSegment {
                        from: point(lx, ly),
                        ctrl: point(ctrl.x, ctrl.y),
                        to: point(p.x, p.y),
                    };

                    // Flatten curve to line segments
                    // Callback receives LineSegment, we take the endpoint of each segment
                    curve.for_each_flattened(CURVE_TOLERANCE, &mut |segment| {
                        points.push(transform_point(segment.to.x, segment.to.y));
                    });
                } else {
                    // Fallback: just add endpoint if no previous point
                    points.push(transform_point(p.x, p.y));
                }
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(ctrl1, ctrl2, p) => {
                // Properly flatten cubic Bézier curve
                if let Some((lx, ly)) = last_point {
                    let curve = CubicBezierSegment {
                        from: point(lx, ly),
                        ctrl1: point(ctrl1.x, ctrl1.y),
                        ctrl2: point(ctrl2.x, ctrl2.y),
                        to: point(p.x, p.y),
                    };

                    // Flatten curve to line segments
                    // Callback receives LineSegment, we take the endpoint of each segment
                    curve.for_each_flattened(CURVE_TOLERANCE, &mut |segment| {
                        points.push(transform_point(segment.to.x, segment.to.y));
                    });
                } else {
                    // Fallback: just add endpoint if no previous point
                    points.push(transform_point(p.x, p.y));
                }
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                // Path is closed - finalize this subpath
                if let Some(polygon) = finalize_subpath(&mut points, subpath_index) {
                    polygons.push(polygon);
                }
                subpath_index += 1;
            }
        }
    }

    // Finalize any remaining points (unclosed path)
    if !points.is_empty() {
        if let Some(polygon) = finalize_subpath(&mut points, subpath_index) {
            polygons.push(polygon);
        }
    }

    // Detect and assemble holes for compound paths
    assemble_holes(polygons)
}

/// Detect containment relationships between polygons and assemble holes.
///
/// When a compound path (multiple subpaths in one <path> element) contains
/// shapes with opposite winding directions, the inner shapes are typically holes.
/// This function:
/// 1. Detects which polygons are fully contained within others
/// 2. Checks for opposite winding direction (hole indicator)
/// 3. Moves hole polygons into their parent's `holes` field
/// 4. Returns only the outer polygons (with holes attached)
fn assemble_holes(polygons: Vec<Polygon>) -> Vec<Polygon> {
    if polygons.len() <= 1 {
        return polygons;
    }

    // Calculate signed areas and bounding boxes for all polygons
    let polygon_data: Vec<(f64, Option<(f64, f64, f64, f64)>)> = polygons
        .iter()
        .map(|p| (p.signed_area(), p.bounding_box()))
        .collect();

    // Track which polygons are holes (and their parent index)
    let mut hole_of: Vec<Option<usize>> = vec![None; polygons.len()];

    // For each polygon, check if it's contained within another
    for i in 0..polygons.len() {
        let (area_i, bbox_i) = polygon_data[i];
        let Some((min_x_i, min_y_i, max_x_i, max_y_i)) = bbox_i else {
            continue;
        };

        for j in 0..polygons.len() {
            if i == j {
                continue;
            }

            let (area_j, bbox_j) = polygon_data[j];
            let Some((min_x_j, min_y_j, max_x_j, max_y_j)) = bbox_j else {
                continue;
            };

            // Quick bounding box rejection: i must be inside j's bbox
            if min_x_i < min_x_j || max_x_i > max_x_j ||
               min_y_i < min_y_j || max_y_i > max_y_j {
                continue;
            }

            // Check for opposite winding direction (indicates hole relationship)
            // In SVG coordinate space: outer is typically CCW (positive area),
            // holes are typically CW (negative area)
            let opposite_winding = (area_i > 0.0) != (area_j > 0.0);
            if !opposite_winding {
                continue;
            }

            // Polygon i (smaller) should be contained in polygon j (larger)
            // Check if j has larger absolute area (is the outer)
            if area_i.abs() >= area_j.abs() {
                continue;
            }

            // Check if all vertices of i are inside j
            let all_inside = polygons[i].outer.iter().all(|p| {
                point_in_polygon(p.x, p.y, &polygons[j].outer)
            });

            if all_inside {
                // i is a hole of j
                // If i is already marked as a hole of something else, prefer
                // the smallest containing polygon (most immediate parent)
                if let Some(existing_parent) = hole_of[i] {
                    let existing_area = polygon_data[existing_parent].0.abs();
                    if area_j.abs() < existing_area {
                        hole_of[i] = Some(j);
                    }
                } else {
                    hole_of[i] = Some(j);
                }
            }
        }
    }

    // Collect holes for each outer polygon
    let mut holes_for: Vec<Vec<Vec<Point>>> = vec![Vec::new(); polygons.len()];
    for (i, parent) in hole_of.iter().enumerate() {
        if let Some(p) = parent {
            // Clone the hole's outer boundary
            holes_for[*p].push(polygons[i].outer.clone());
        }
    }

    // Build result: only outer polygons (with their holes attached)
    let mut result = Vec::new();
    for (i, polygon) in polygons.into_iter().enumerate() {
        if hole_of[i].is_none() {
            // This is an outer polygon - attach its holes
            let holes = std::mem::take(&mut holes_for[i]);
            result.push(Polygon {
                outer: polygon.outer,
                holes,
                id: polygon.id,
            });
        }
    }

    result
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rect() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <rect x="10" y="10" width="80" height="80"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1);
        assert_eq!(polygons[0].outer.len(), 4); // rect = 4 points
    }

    #[test]
    fn parse_polygon_element() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <polygon points="10,10 90,10 90,90 10,90"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1);
    }

    #[test]
    fn no_polygons_error() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
            </svg>
        "#;

        let result = extract_polygons_from_svg(svg);
        assert!(matches!(result, Err(SvgError::NoPolygons)));
    }

    #[test]
    fn curve_flattening_circle() {
        // A circle uses cubic Bézier curves - without proper flattening,
        // this would only have 4-5 points (just the endpoints)
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <circle cx="50" cy="50" r="40"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1);
        // A properly flattened circle should have many points (not just 4)
        // With tolerance 0.1 and radius 40, we expect ~50+ points
        assert!(polygons[0].outer.len() > 20,
            "Circle should have many points from curve flattening, got {}",
            polygons[0].outer.len());
    }

    #[test]
    fn curve_flattening_ellipse() {
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <ellipse cx="50" cy="50" rx="40" ry="20"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1);
        assert!(polygons[0].outer.len() > 20,
            "Ellipse should have many points from curve flattening, got {}",
            polygons[0].outer.len());
    }

    #[test]
    fn curve_flattening_path_with_bezier() {
        // Path with cubic Bézier curve
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <path d="M 10,10 C 40,10 60,90 90,90 L 90,10 Z"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1);
        // The cubic curve should be flattened to multiple points
        assert!(polygons[0].outer.len() > 5,
            "Path with Bézier should have multiple points, got {}",
            polygons[0].outer.len());
    }

    #[test]
    fn compound_path_donut_detects_hole() {
        // A donut shape: CCW outer square, CW inner square (hole)
        // SVG convention: CW inner subpath = hole
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <path d="M 10,10 L 90,10 L 90,90 L 10,90 Z
                         M 30,30 L 30,70 L 70,70 L 70,30 Z"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();

        // Should be 1 polygon (outer) with 1 hole (inner)
        assert_eq!(polygons.len(), 1, "Should have 1 outer polygon, got {}", polygons.len());
        assert_eq!(polygons[0].holes.len(), 1, "Should have 1 hole, got {}", polygons[0].holes.len());

        // Outer should be the larger square (10-90)
        assert_eq!(polygons[0].outer.len(), 4, "Outer should be a square");

        // Hole should be the smaller square (30-70)
        assert_eq!(polygons[0].holes[0].len(), 4, "Hole should be a square");
    }

    #[test]
    fn separate_paths_remain_separate() {
        // Two separate <path> elements should NOT be merged (even if one contains the other)
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <path d="M 10,10 L 90,10 L 90,90 L 10,90 Z"/>
                <path d="M 30,30 L 30,70 L 70,70 L 70,30 Z"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();

        // Should be 2 separate polygons, not 1 polygon with 1 hole
        assert_eq!(polygons.len(), 2, "Should have 2 separate polygons, got {}", polygons.len());
        assert!(polygons[0].holes.is_empty(), "First polygon should have no holes");
        assert!(polygons[1].holes.is_empty(), "Second polygon should have no holes");
    }

    #[test]
    fn same_winding_nested_shapes_remain_separate() {
        // Two subpaths with same winding direction should NOT be treated as hole
        // Both CCW (outer then another outer inside)
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <path d="M 10,10 L 90,10 L 90,90 L 10,90 Z
                         M 30,30 L 70,30 L 70,70 L 30,70 Z"/>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();

        // Same winding = both are outer shapes, not parent/hole
        assert_eq!(polygons.len(), 2, "Should have 2 separate polygons (same winding), got {}", polygons.len());
    }

    #[test]
    fn group_transforms_are_applied() {
        // Test that transforms from parent groups are applied to path coordinates
        // The path coordinates are at 110-190, but the group transform should
        // shift them to 10-90 (matching the viewBox 0-100)
        let svg = r#"
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                <g transform="translate(-100, -100)">
                    <path d="M 110,110 L 190,110 L 190,190 L 110,190 Z"/>
                </g>
            </svg>
        "#;

        let polygons = extract_polygons_from_svg(svg).unwrap();
        assert_eq!(polygons.len(), 1, "Should have 1 polygon");

        // Check that coordinates are transformed (should be around 10-90, not 110-190)
        let polygon = &polygons[0];
        for point in &polygon.outer {
            assert!(point.x >= 9.0 && point.x <= 91.0,
                "X coordinate {} should be in range 10-90 after transform", point.x);
            assert!(point.y >= 9.0 && point.y <= 91.0,
                "Y coordinate {} should be in range 10-90 after transform", point.y);
        }
    }
}
