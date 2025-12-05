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
            holes: Vec::new(), // Empty vec, like []
            id: None,
        }
    }

    /// Create a polygon with holes.
    pub fn with_holes(outer: Vec<Point>, holes: Vec<Vec<Point>>) -> Self {
        Self { outer, holes, id: None }
    }

    /// Create a polygon with an ID.
    pub fn with_id(outer: Vec<Point>, id: Option<String>) -> Self {
        Self {
            outer,
            holes: Vec::new(),
            id,
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
}
