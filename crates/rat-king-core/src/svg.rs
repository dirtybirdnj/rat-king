//! SVG parsing - extract polygons from SVG files.
//!
//! Uses usvg for complete SVG resolution (CSS, transforms, etc.)
//! then walks the tree to extract path data as polygons.

use crate::geometry::{Point, Polygon};

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

/// Convert a usvg path to our Polygon type.
fn path_to_polygon(path: &usvg::Path) -> Option<Polygon> {
    let data = path.data();

    // ## Rust Lesson #23: Iterator Peekable & Adapters
    //
    // We need to parse SVG path commands (M, L, C, Z, etc.)
    // usvg already gives us absolute coordinates (no relative commands!)

    let mut points = Vec::new();

    for cmd in data.segments() {
        match cmd {
            usvg::tiny_skia_path::PathSegment::MoveTo(p) => {
                // Start of new subpath - if we have points, that's a polygon
                if !points.is_empty() {
                    // TODO: handle multiple subpaths (holes?)
                    break;
                }
                points.push(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(p) => {
                points.push(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(_, p) => {
                // Approximate curve with endpoint
                points.push(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(_, _, p) => {
                // Approximate curve with endpoint
                points.push(Point::new(p.x as f64, p.y as f64));
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                // Path is closed - we have a polygon!
            }
        }
    }

    if points.len() >= 3 {
        Some(Polygon::new(points))
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
}
