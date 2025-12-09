//! Shared utilities for pattern generation.
//!
//! This module provides common functionality used across multiple pattern
//! implementations to reduce code duplication.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::point_in_polygon;

/// Context for pattern generation with pre-computed values.
///
/// Many patterns need the same setup: bounding box, center, diagonal, etc.
/// This struct computes these once and provides convenient access.
///
/// # Example
/// ```ignore
/// let ctx = PatternContext::new(polygon, spacing, angle_degrees)?;
/// // Now use ctx.center, ctx.diagonal, ctx.rotate(), etc.
/// ```
#[derive(Clone)]
pub struct PatternContext<'a> {
    /// Reference to the polygon being filled
    pub polygon: &'a Polygon,
    /// Spacing parameter from the fill request
    pub spacing: f64,
    /// Angle in degrees
    pub angle_degrees: f64,
    /// Angle in radians (pre-computed)
    pub angle_rad: f64,
    /// Bounding box: (min_x, min_y, max_x, max_y)
    pub bounds: (f64, f64, f64, f64),
    /// Center of bounding box
    pub center: Point,
    /// Width of bounding box
    pub width: f64,
    /// Height of bounding box
    pub height: f64,
    /// Diagonal length of bounding box
    pub diagonal: f64,
}

impl<'a> PatternContext<'a> {
    /// Create a new pattern context, returning None if polygon is invalid.
    pub fn new(polygon: &'a Polygon, spacing: f64, angle_degrees: f64) -> Option<Self> {
        if polygon.outer.len() < 3 {
            return None;
        }

        let bounds = polygon.bounding_box()?;
        let (min_x, min_y, max_x, max_y) = bounds;

        let width = max_x - min_x;
        let height = max_y - min_y;

        Some(Self {
            polygon,
            spacing,
            angle_degrees,
            angle_rad: angle_degrees * PI / 180.0,
            bounds,
            center: Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0),
            width,
            height,
            diagonal: (width * width + height * height).sqrt(),
        })
    }

    /// Rotate a point around the center by the context's angle.
    #[inline]
    pub fn rotate(&self, x: f64, y: f64) -> (f64, f64) {
        let dx = x - self.center.x;
        let dy = y - self.center.y;
        let cos_a = self.angle_rad.cos();
        let sin_a = self.angle_rad.sin();
        (
            self.center.x + dx * cos_a - dy * sin_a,
            self.center.y + dx * sin_a + dy * cos_a,
        )
    }

    /// Check if a point is inside the polygon body (not in holes).
    #[inline]
    pub fn point_inside(&self, x: f64, y: f64) -> bool {
        if !point_in_polygon(x, y, &self.polygon.outer) {
            return false;
        }
        !self.polygon.holes.iter().any(|hole| point_in_polygon(x, y, hole))
    }

    /// Check if a line segment's midpoint is inside the polygon body.
    #[inline]
    pub fn line_inside(&self, line: &Line) -> bool {
        let mid_x = (line.x1 + line.x2) / 2.0;
        let mid_y = (line.y1 + line.y2) / 2.0;
        self.point_inside(mid_x, mid_y)
    }

    /// Get padding amount for generating lines beyond bounds.
    #[inline]
    pub fn padding(&self) -> f64 {
        self.diagonal / 2.0 + self.spacing
    }

    /// Calculate number of lines needed to cover the diagonal.
    #[inline]
    pub fn line_count(&self) -> i32 {
        (self.diagonal / self.spacing).ceil() as i32
    }
}

/// A 2D rotation transform around a center point.
///
/// Use this for patterns that need to rotate coordinates.
#[derive(Clone, Copy)]
pub struct RotationTransform {
    pub center_x: f64,
    pub center_y: f64,
    pub cos_a: f64,
    pub sin_a: f64,
}

impl RotationTransform {
    /// Create a new rotation transform.
    pub fn new(center_x: f64, center_y: f64, angle_rad: f64) -> Self {
        Self {
            center_x,
            center_y,
            cos_a: angle_rad.cos(),
            sin_a: angle_rad.sin(),
        }
    }

    /// Create from degrees instead of radians.
    pub fn from_degrees(center_x: f64, center_y: f64, angle_degrees: f64) -> Self {
        Self::new(center_x, center_y, angle_degrees * PI / 180.0)
    }

    /// Apply the rotation to a point.
    #[inline]
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        (
            self.center_x + dx * self.cos_a - dy * self.sin_a,
            self.center_y + dx * self.sin_a + dy * self.cos_a,
        )
    }

    /// Apply the rotation to a line.
    #[inline]
    pub fn apply_line(&self, line: &Line) -> Line {
        let (x1, y1) = self.apply(line.x1, line.y1);
        let (x2, y2) = self.apply(line.x2, line.y2);
        Line::new(x1, y1, x2, y2)
    }
}

/// Direction vectors for parallel line generation.
///
/// Used by patterns that generate parallel lines at an angle.
#[derive(Clone, Copy)]
pub struct LineDirection {
    /// Direction along the lines (unit vector)
    pub dx: f64,
    pub dy: f64,
    /// Direction perpendicular to lines (for spacing)
    pub px: f64,
    pub py: f64,
}

impl LineDirection {
    /// Create line direction from angle in radians.
    pub fn new(angle_rad: f64) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            dx: cos_a,
            dy: sin_a,
            px: -sin_a,
            py: cos_a,
        }
    }

    /// Create from angle in degrees.
    pub fn from_degrees(angle_degrees: f64) -> Self {
        Self::new(angle_degrees * PI / 180.0)
    }

    /// Generate parallel lines through a center point.
    ///
    /// Returns unclipped lines that extend `length` in both directions
    /// from the center, offset perpendicular to the line direction.
    pub fn generate_parallel_lines(
        &self,
        center: Point,
        spacing: f64,
        num_lines: i32,
        length: f64,
    ) -> Vec<Line> {
        let mut lines = Vec::with_capacity((num_lines * 2 + 1) as usize);

        for i in -num_lines..=num_lines {
            let offset = i as f64 * spacing;

            // Center point offset perpendicular to line direction
            let cx = center.x + self.px * offset;
            let cy = center.y + self.py * offset;

            // Extend line in both directions
            let x1 = cx - self.dx * length;
            let y1 = cy - self.dy * length;
            let x2 = cx + self.dx * length;
            let y2 = cy + self.dy * length;

            lines.push(Line::new(x1, y1, x2, y2));
        }

        lines
    }
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
    fn pattern_context_creation() {
        let poly = square_polygon();
        let ctx = PatternContext::new(&poly, 10.0, 45.0).unwrap();

        assert_eq!(ctx.center.x, 50.0);
        assert_eq!(ctx.center.y, 50.0);
        assert_eq!(ctx.width, 100.0);
        assert_eq!(ctx.height, 100.0);
        assert!((ctx.diagonal - 141.42).abs() < 0.1);
    }

    #[test]
    fn rotation_transform() {
        let rot = RotationTransform::from_degrees(50.0, 50.0, 90.0);

        // Point at (100, 50) should rotate to (50, 100)
        let (x, y) = rot.apply(100.0, 50.0);
        assert!((x - 50.0).abs() < 0.0001);
        assert!((y - 100.0).abs() < 0.0001);
    }

    #[test]
    fn line_direction_parallel() {
        let dir = LineDirection::from_degrees(0.0); // Horizontal lines
        let lines = dir.generate_parallel_lines(Point::new(50.0, 50.0), 10.0, 2, 100.0);

        // Should generate 5 lines (-2, -1, 0, 1, 2)
        assert_eq!(lines.len(), 5);

        // Center line should be horizontal through (50, 50)
        let center_line = &lines[2]; // i=0
        assert!((center_line.y1 - 50.0).abs() < 0.0001);
        assert!((center_line.y2 - 50.0).abs() < 0.0001);
    }

    #[test]
    fn context_point_inside() {
        let poly = square_polygon();
        let ctx = PatternContext::new(&poly, 10.0, 0.0).unwrap();

        assert!(ctx.point_inside(50.0, 50.0));
        assert!(!ctx.point_inside(150.0, 50.0));
    }
}
