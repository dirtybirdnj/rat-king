//! Stripe fill pattern with configurable line groups.
//!
//! Creates stripes with X lines of Y pen width spaced Z apart.
//! Useful for creating banded effects with grouped parallel lines.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;

/// Configuration for stripe pattern.
#[derive(Debug, Clone, Copy)]
pub struct StripeConfig {
    /// Number of lines per stripe group
    pub lines_per_stripe: usize,
    /// Spacing between lines within a stripe group (pen width equivalent)
    pub line_spacing: f64,
    /// Spacing between stripe groups
    pub stripe_spacing: f64,
    /// Rotation angle in degrees
    pub angle_degrees: f64,
}

impl Default for StripeConfig {
    fn default() -> Self {
        Self {
            lines_per_stripe: 3,
            line_spacing: 1.0,
            stripe_spacing: 10.0,
            angle_degrees: 0.0,
        }
    }
}

/// Generate stripe fill pattern for a polygon.
///
/// Creates grouped parallel lines (stripes) with configurable:
/// - Number of lines per stripe
/// - Spacing between lines within stripe
/// - Spacing between stripe groups
pub fn generate_stripe_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    // Use spacing parameter as stripe_spacing, derive others
    let config = StripeConfig {
        lines_per_stripe: 3,
        line_spacing: spacing * 0.3,
        stripe_spacing: spacing,
        angle_degrees,
    };
    generate_stripe_fill_configured(polygon, &config)
}

/// Generate stripe fill with full configuration.
pub fn generate_stripe_fill_configured(
    polygon: &Polygon,
    config: &StripeConfig,
) -> Vec<Line> {
    let outer = &polygon.outer;
    if outer.len() < 3 {
        return Vec::new();
    }

    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let angle_rad = config.angle_degrees * PI / 180.0;

    let width = max_x - min_x;
    let height = max_y - min_y;
    let diagonal = (width * width + height * height).sqrt();

    // Direction along the lines
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    // Direction perpendicular to lines (for spacing)
    let px = -sin_a;
    let py = cos_a;

    let mut lines = Vec::new();

    // Calculate total width of one stripe group
    let stripe_width = (config.lines_per_stripe - 1) as f64 * config.line_spacing;
    let total_stripe_pitch = stripe_width + config.stripe_spacing;

    // Number of stripe groups needed
    let num_stripes = (diagonal / total_stripe_pitch).ceil() as i32;

    for stripe_idx in -num_stripes..=num_stripes {
        // Position of stripe group center
        let stripe_offset = stripe_idx as f64 * total_stripe_pitch;

        for line_idx in 0..config.lines_per_stripe {
            // Position of this line within the stripe group
            // Center the group around the stripe position
            let line_offset = stripe_offset
                + (line_idx as f64 - (config.lines_per_stripe - 1) as f64 / 2.0)
                * config.line_spacing;

            // Calculate line position
            let cx = center_x + px * line_offset;
            let cy = center_y + py * line_offset;

            // Extend line in both directions
            let x1 = cx - cos_a * diagonal;
            let y1 = cy - sin_a * diagonal;
            let x2 = cx + cos_a * diagonal;
            let y2 = cy + sin_a * diagonal;

            lines.push(Line::new(x1, y1, x2, y2));
        }
    }

    clip_lines_to_polygon(&lines, polygon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_stripe_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_stripe_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate stripe lines");
    }

    #[test]
    fn configured_stripes_work() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let config = StripeConfig {
            lines_per_stripe: 5,
            line_spacing: 2.0,
            stripe_spacing: 20.0,
            angle_degrees: 45.0,
        };

        let lines = generate_stripe_fill_configured(&poly, &config);
        assert!(!lines.is_empty(), "Should generate configured stripes");
    }

    #[test]
    fn more_lines_per_stripe_increases_count() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let config_3 = StripeConfig {
            lines_per_stripe: 3,
            line_spacing: 1.0,
            stripe_spacing: 20.0,
            angle_degrees: 0.0,
        };

        let config_5 = StripeConfig {
            lines_per_stripe: 5,
            line_spacing: 1.0,
            stripe_spacing: 20.0,
            angle_degrees: 0.0,
        };

        let lines_3 = generate_stripe_fill_configured(&poly, &config_3);
        let lines_5 = generate_stripe_fill_configured(&poly, &config_5);

        assert!(lines_5.len() > lines_3.len(),
            "More lines per stripe should produce more total lines");
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines_dense = generate_stripe_fill(&poly, 5.0, 0.0);
        let lines_sparse = generate_stripe_fill(&poly, 20.0, 0.0);

        assert!(lines_dense.len() > lines_sparse.len(),
            "Smaller spacing should produce more stripes");
    }
}
