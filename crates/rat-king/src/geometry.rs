//! Core geometry types for rat-king.
//!
//! ## Rust Lesson #3: Structs & Derives
//!
//! In JS you'd write: `const point = { x: 1.0, y: 2.0 }`
//! In Rust, we define a `struct` with explicit types.
//!
//! The `#[derive(...)]` macro auto-generates common functionality:
//! - `Debug` = like console.log, lets you print with `{:?}`
//! - `Clone` = can duplicate the value (like spread: `{...obj}`)
//! - `Copy` = can copy implicitly (small stack values only)
//! - `PartialEq` = can compare with `==`

/// A 2D point with x,y coordinates.
///
/// `f64` = 64-bit float (like JS's `number` but explicitly sized)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A line segment defined by two endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// A polygon with an outer boundary and optional holes.
///
/// ## Rust Lesson #4: Ownership & Vec
///
/// `Vec<Point>` is like a JS array `Point[]` - a growable list.
/// Unlike JS, Rust tracks *who owns* the data.
///
/// - This struct OWNS its points - when it's dropped, they're freed
/// - No garbage collector - memory is freed deterministically
/// - `&[Point]` would be a BORROWED slice (read-only view)
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    /// Outer boundary vertices (counter-clockwise)
    pub outer: Vec<Point>,
    /// Interior holes (clockwise winding)
    pub holes: Vec<Vec<Point>>,
    /// Optional ID from the SVG element
    pub id: Option<String>,
    /// Optional parent group ID from the SVG (for per-group styling)
    pub group_id: Option<String>,
    /// Optional pattern name from data-pattern attribute
    pub data_pattern: Option<String>,
    /// Optional shade index from data-shade attribute (0-5)
    pub data_shade: Option<u8>,
    /// Optional spacing from data-spacing attribute
    pub data_spacing: Option<f64>,
    /// Optional angle from data-angle attribute (degrees)
    pub data_angle: Option<f64>,
    /// Optional color from data-color attribute
    pub data_color: Option<String>,
    /// Original stroke color from the SVG element
    pub stroke_color: Option<String>,
    /// Original stroke width from the SVG element
    pub stroke_width: Option<f64>,
}

// ============================================================================
// IMPLEMENTATIONS (methods)
// ============================================================================
//
// ## Rust Lesson #5: impl blocks
//
// In JS you'd use class methods: `class Point { distance() {...} }`
// In Rust, we separate data (struct) from behavior (impl).
// This lets you add methods to types from other crates!

impl Point {
    /// Create a new point. This is a common pattern instead of constructors.
    ///
    /// Called as: `Point::new(1.0, 2.0)` (like static method)
    #[inline]
    pub fn new(x: f64, y: f64) -> Self {
        // `Self` = the type we're implementing (Point)
        Self { x, y }
    }

    /// Distance to another point.
    ///
    /// `&self` = borrow self (read-only access, like `this` in JS)
    /// `other: Point` = takes ownership of other (consumes it)
    ///
    /// But wait - Point has `Copy`, so it's implicitly copied!
    #[inline]
    pub fn distance(&self, other: Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Line {
    #[inline]
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Get the start point of the line.
    #[inline]
    pub fn start(&self) -> Point {
        Point::new(self.x1, self.y1)
    }

    /// Get the end point of the line.
    #[inline]
    pub fn end(&self) -> Point {
        Point::new(self.x2, self.y2)
    }

    /// Get the midpoint of the line.
    #[inline]
    pub fn midpoint(&self) -> Point {
        Point::new(
            (self.x1 + self.x2) / 2.0,
            (self.y1 + self.y2) / 2.0,
        )
    }

    /// Length of the line segment.
    #[inline]
    pub fn length(&self) -> f64 {
        self.start().distance(self.end())
    }
}

impl Polygon {
    /// Create a simple polygon with no holes.
    pub fn new(outer: Vec<Point>) -> Self {
        Self {
            outer,
            holes: Vec::new(),
            id: None,
            group_id: None,
            data_pattern: None,
            data_shade: None,
            data_spacing: None,
            data_angle: None,
            data_color: None,
            stroke_color: None,
            stroke_width: None,
        }
    }

    /// Create a polygon with holes.
    pub fn with_holes(outer: Vec<Point>, holes: Vec<Vec<Point>>) -> Self {
        Self {
            outer, holes,
            id: None, group_id: None,
            data_pattern: None, data_shade: None, data_spacing: None, data_angle: None,
            data_color: None, stroke_color: None, stroke_width: None,
        }
    }

    /// Create a polygon with an ID.
    pub fn with_id(outer: Vec<Point>, id: Option<String>) -> Self {
        Self {
            outer,
            holes: Vec::new(),
            id,
            group_id: None,
            data_pattern: None,
            data_shade: None,
            data_spacing: None,
            data_angle: None,
            data_color: None,
            stroke_color: None,
            stroke_width: None,
        }
    }

    /// Create a polygon with ID and group ID.
    pub fn with_id_and_group(outer: Vec<Point>, id: Option<String>, group_id: Option<String>) -> Self {
        Self {
            outer,
            holes: Vec::new(),
            id,
            group_id,
            data_pattern: None,
            data_shade: None,
            data_spacing: None,
            data_angle: None,
            data_color: None,
            stroke_color: None,
            stroke_width: None,
        }
    }

    /// Create a polygon with all metadata fields.
    pub fn with_metadata(
        outer: Vec<Point>,
        id: Option<String>,
        group_id: Option<String>,
        data_pattern: Option<String>,
        data_shade: Option<u8>,
        data_spacing: Option<f64>,
        data_angle: Option<f64>,
        data_color: Option<String>,
        stroke_color: Option<String>,
        stroke_width: Option<f64>,
    ) -> Self {
        Self {
            outer,
            holes: Vec::new(),
            id,
            group_id,
            data_pattern,
            data_shade,
            data_spacing,
            data_angle,
            data_color,
            stroke_color,
            stroke_width,
        }
    }

    /// Get the bounding box as (min_x, min_y, max_x, max_y).
    ///
    /// ## Rust Lesson #6: Option<T>
    ///
    /// Rust has no `null` or `undefined`. Instead, we use `Option<T>`:
    /// - `Some(value)` = we have a value
    /// - `None` = no value
    ///
    /// This is checked at compile time - you CAN'T forget to handle None!
    pub fn bounding_box(&self) -> Option<(f64, f64, f64, f64)> {
        if self.outer.is_empty() {
            return None; // Early return, like JS
        }

        // Iterators! Like JS's .map().filter().reduce() but zero-cost.
        // The compiler turns this into a simple loop.
        let min_x = self.outer.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let min_y = self.outer.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        let max_x = self.outer.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
        let max_y = self.outer.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);

        Some((min_x, min_y, max_x, max_y))
    }

    /// Check if a point is inside the polygon body (inside outer, not in any hole).
    ///
    /// This is a convenience method that combines the outer boundary check
    /// with hole exclusion, which is a very common pattern in fill algorithms.
    ///
    /// # Arguments
    /// * `x` - X coordinate to test
    /// * `y` - Y coordinate to test
    /// * `point_in_polygon_fn` - Function to test point-in-polygon (typically `clip::point_in_polygon`)
    ///
    /// # Example
    /// ```ignore
    /// use rat_king::clip::point_in_polygon;
    /// if polygon.point_in_body(x, y, point_in_polygon) {
    ///     // Point is inside the filled region
    /// }
    /// ```
    #[inline]
    pub fn point_in_body<F>(&self, x: f64, y: f64, point_in_polygon_fn: F) -> bool
    where
        F: Fn(f64, f64, &[Point]) -> bool,
    {
        // Must be inside outer boundary
        if !point_in_polygon_fn(x, y, &self.outer) {
            return false;
        }
        // Must not be in any hole
        !self.holes.iter().any(|hole| point_in_polygon_fn(x, y, hole))
    }

    /// Get the center point of the polygon's bounding box.
    #[inline]
    pub fn center(&self) -> Option<Point> {
        self.bounding_box().map(|(min_x, min_y, max_x, max_y)| {
            Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
        })
    }

    /// Get the diagonal length of the bounding box.
    #[inline]
    pub fn diagonal(&self) -> Option<f64> {
        self.bounding_box().map(|(min_x, min_y, max_x, max_y)| {
            let width = max_x - min_x;
            let height = max_y - min_y;
            (width * width + height * height).sqrt()
        })
    }

    /// Calculate signed area using the shoelace formula.
    ///
    /// Returns:
    /// - Positive value for counter-clockwise winding
    /// - Negative value for clockwise winding
    /// - The absolute value is the polygon area
    ///
    /// This is useful for determining winding direction and for
    /// detecting which polygons are holes (opposite winding from outer).
    #[inline]
    pub fn signed_area(&self) -> f64 {
        signed_area_of_points(&self.outer)
    }

    /// Check if the outer boundary has clockwise winding.
    ///
    /// In SVG coordinate space (Y increases downward):
    /// - Clockwise winding typically indicates a hole
    /// - Counter-clockwise winding typically indicates an outer boundary
    #[inline]
    pub fn is_clockwise(&self) -> bool {
        self.signed_area() < 0.0
    }
}

/// Calculate signed area of a point sequence using the shoelace formula.
///
/// Returns:
/// - Positive value for counter-clockwise winding
/// - Negative value for clockwise winding
pub fn signed_area_of_points(points: &[Point]) -> f64 {
    let n = points.len();
    if n < 3 {
        return 0.0;
    }

    let mut area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += points[i].x * points[j].y;
        area -= points[j].x * points[i].y;
    }
    area / 2.0
}

// ============================================================================
// TESTS
// ============================================================================
//
// Tests live right next to the code! Run with `cargo test`.
// The #[cfg(test)] means this only compiles during testing.

#[cfg(test)]
mod tests {
    use super::*; // Import everything from parent module

    #[test]
    fn point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance(p2), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn line_length() {
        let line = Line::new(0.0, 0.0, 3.0, 4.0);
        assert_eq!(line.length(), 5.0);
    }

    #[test]
    fn polygon_bbox() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 5.0),
            Point::new(0.0, 5.0),
        ]);
        assert_eq!(poly.bounding_box(), Some((0.0, 0.0, 10.0, 5.0)));
    }

    #[test]
    fn empty_polygon_bbox() {
        let poly = Polygon::new(vec![]);
        assert_eq!(poly.bounding_box(), None);
    }

    #[test]
    fn polygon_center() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);
        let center = poly.center().unwrap();
        assert_eq!(center.x, 5.0);
        assert_eq!(center.y, 5.0);
    }

    #[test]
    fn polygon_diagonal() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(3.0, 0.0),
            Point::new(3.0, 4.0),
            Point::new(0.0, 4.0),
        ]);
        let diag = poly.diagonal().unwrap();
        assert_eq!(diag, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn point_in_body_basic() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);

        // Simple point_in_polygon for testing
        fn pip(x: f64, y: f64, pts: &[Point]) -> bool {
            let mut inside = false;
            let n = pts.len();
            let mut j = n - 1;
            for i in 0..n {
                let yi = pts[i].y;
                let yj = pts[j].y;
                let xi = pts[i].x;
                let xj = pts[j].x;
                if ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi) {
                    inside = !inside;
                }
                j = i;
            }
            inside
        }

        assert!(poly.point_in_body(5.0, 5.0, pip));
        assert!(!poly.point_in_body(15.0, 5.0, pip));
    }

    #[test]
    fn signed_area_ccw_positive() {
        // Counter-clockwise square: positive area
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);
        let area = poly.signed_area();
        assert!(area > 0.0, "CCW polygon should have positive signed area, got {}", area);
        assert!((area - 100.0).abs() < 1e-10, "10x10 square should have area 100, got {}", area);
        assert!(!poly.is_clockwise(), "CCW polygon should not be clockwise");
    }

    #[test]
    fn signed_area_cw_negative() {
        // Clockwise square: negative area
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 0.0),
        ]);
        let area = poly.signed_area();
        assert!(area < 0.0, "CW polygon should have negative signed area, got {}", area);
        assert!((area + 100.0).abs() < 1e-10, "10x10 square should have area -100, got {}", area);
        assert!(poly.is_clockwise(), "CW polygon should be clockwise");
    }

    #[test]
    fn signed_area_triangle() {
        // CCW triangle
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 10.0),
        ]);
        let area = poly.signed_area();
        assert!(area > 0.0, "CCW triangle should have positive area");
        assert!((area - 50.0).abs() < 1e-10, "Triangle area should be 50, got {}", area);
    }
}
