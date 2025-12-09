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
            // Extract polygon from path data
            if let Some(polygon) = path_to_polygon(path) {
                polygons.push(polygon);
            }
        }
        // Ignore text, images, etc.
        _ => {}
    }
}

/// Tolerance for curve flattening.
/// Lower = more points, smoother curves, slower.
/// 0.1 is good for plotters (sub-pixel accuracy at typical scales).
const CURVE_TOLERANCE: f32 = 0.1;

/// Convert a usvg path to our Polygon type.
///
/// Properly flattens Bézier curves using lyon_geom for accurate polygon boundaries.
fn path_to_polygon(path: &usvg::Path) -> Option<Polygon> {
    let data = path.data();
    let id = path.id();

    // ## Rust Lesson #23: Iterator Peekable & Adapters
    //
    // We need to parse SVG path commands (M, L, C, Z, etc.)
    // usvg already gives us absolute coordinates (no relative commands!)

    let mut points = Vec::new();
    let mut last_point: Option<(f32, f32)> = None;

    for cmd in data.segments() {
        match cmd {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                // Start of new subpath - if we have points, that's a polygon
                if !points.is_empty() {
                    // TODO: handle multiple subpaths (holes?)
                    break;
                }
                points.push(Point::new(p.x as f64, p.y as f64));
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                points.push(Point::new(p.x as f64, p.y as f64));
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
                        points.push(Point::new(segment.to.x as f64, segment.to.y as f64));
                    });
                } else {
                    // Fallback: just add endpoint if no previous point
                    points.push(Point::new(p.x as f64, p.y as f64));
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
                        points.push(Point::new(segment.to.x as f64, segment.to.y as f64));
                    });
                } else {
                    // Fallback: just add endpoint if no previous point
                    points.push(Point::new(p.x as f64, p.y as f64));
                }
                last_point = Some((p.x, p.y));
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                // Path is closed - we have a polygon!
            }
        }
    }

    // Remove duplicate consecutive points that can occur from curve flattening
    if points.len() >= 2 {
        points.dedup_by(|a, b| {
            let dx = (a.x - b.x).abs();
            let dy = (a.y - b.y).abs();
            dx < 1e-6 && dy < 1e-6
        });
    }

    if points.len() >= 3 {
        // Preserve the element's ID if it has one
        let polygon_id = if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        };
        Some(Polygon::with_id(points, polygon_id))
    } else {
        None
    }
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
}
