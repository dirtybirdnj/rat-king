//! Zigzag fill pattern - continuous 90-degree turn lines.
//!
//! Creates a single continuous polyline that zigzags back and forth
//! across the polygon, like a snake or maze path.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;
use crate::rng::Rng;

/// Configuration for zigzag pattern.
#[derive(Debug, Clone, Copy)]
pub struct ZigzagConfig {
    /// Base spacing between parallel runs
    pub spacing: f64,
    /// Rotation angle in degrees
    pub angle_degrees: f64,
    /// Enable wild mode - randomly vary segment lengths
    pub wild: bool,
    /// Randomness amount for wild mode (0.0 to 1.0)
    pub wildness: f64,
    /// Seed for wild mode randomness
    pub seed: u64,
}

impl Default for ZigzagConfig {
    fn default() -> Self {
        Self {
            spacing: 5.0,
            angle_degrees: 0.0,
            wild: false,
            wildness: 0.5,
            seed: 42,
        }
    }
}

/// Generate zigzag fill for a polygon.
///
/// Creates continuous lines with 90-degree turns that zigzag across the polygon.
/// The pattern creates connected back-and-forth paths.
pub fn generate_zigzag_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    _amplitude: f64, // Kept for API compatibility, now uses spacing
) -> Vec<Line> {
    let config = ZigzagConfig {
        spacing,
        angle_degrees,
        wild: false,
        ..Default::default()
    };
    generate_zigzag_fill_configured(polygon, &config)
}

/// Generate zigzag fill with wild mode.
pub fn generate_zigzag_fill_wild(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    wildness: f64,
    seed: u64,
) -> Vec<Line> {
    let config = ZigzagConfig {
        spacing,
        angle_degrees,
        wild: true,
        wildness: wildness.clamp(0.0, 1.0),
        seed,
    };
    generate_zigzag_fill_configured(polygon, &config)
}

/// Generate zigzag fill with full configuration.
pub fn generate_zigzag_fill_configured(polygon: &Polygon, config: &ZigzagConfig) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let width = max_x - min_x;
    let height = max_y - min_y;
    let angle_rad = config.angle_degrees * PI / 180.0;

    // Diagonal coverage needed
    let diagonal = (width * width + height * height).sqrt();

    // Direction vectors
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    // Perpendicular direction
    let perp_cos = (angle_rad + PI / 2.0).cos();
    let perp_sin = (angle_rad + PI / 2.0).sin();

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Extended bounds for line generation
    let half_diag = diagonal * 0.75;
    let num_lines = (diagonal / config.spacing).ceil() as i32 + 2;

    let mut rng = Rng::new(config.seed);
    let mut all_lines = Vec::new();

    // Generate a continuous zigzag path
    // Start from one corner and zigzag across
    let mut going_positive = true;

    for i in -num_lines..=num_lines {
        let offset = i as f64 * config.spacing;

        // Apply wild mode variation to offset
        let actual_offset = if config.wild && i != -num_lines {
            let variation = config.wildness * config.spacing * 0.3;
            let random_factor = rng.next_f64() * 2.0 - 1.0;
            offset + random_factor * variation
        } else {
            offset
        };

        // Calculate the center point of this line
        let line_center_x = center_x + perp_cos * actual_offset;
        let line_center_y = center_y + perp_sin * actual_offset;

        // Line endpoints extended beyond bounds
        let half_len = if config.wild {
            let variation = config.wildness * half_diag * 0.2;
            let random_factor = rng.next_f64() * 2.0 - 1.0;
            half_diag + random_factor * variation
        } else {
            half_diag
        };

        let x1 = line_center_x - cos_a * half_len;
        let y1 = line_center_y - sin_a * half_len;
        let x2 = line_center_x + cos_a * half_len;
        let y2 = line_center_y + sin_a * half_len;

        // Add horizontal/main direction line
        if going_positive {
            all_lines.push(Line::new(x1, y1, x2, y2));
        } else {
            all_lines.push(Line::new(x2, y2, x1, y1));
        }

        // Add connecting vertical/perpendicular segment to next row
        if i < num_lines {
            let next_offset = (i + 1) as f64 * config.spacing;
            let next_actual_offset = if config.wild {
                let variation = config.wildness * config.spacing * 0.3;
                let random_factor = rng.next_f64() * 2.0 - 1.0;
                next_offset + random_factor * variation
            } else {
                next_offset
            };

            // Connect from end of current line to start of next
            let connect_x = if going_positive { x2 } else { x1 };
            let connect_y = if going_positive { y2 } else { y1 };

            let next_center_x = center_x + perp_cos * next_actual_offset;
            let next_center_y = center_y + perp_sin * next_actual_offset;

            let next_half_len = if config.wild {
                let variation = config.wildness * half_diag * 0.2;
                let random_factor = rng.next_f64() * 2.0 - 1.0;
                half_diag + random_factor * variation
            } else {
                half_diag
            };

            // Next line's endpoint that we connect to
            let next_end_x = if going_positive {
                next_center_x + cos_a * next_half_len
            } else {
                next_center_x - cos_a * next_half_len
            };
            let next_end_y = if going_positive {
                next_center_y + sin_a * next_half_len
            } else {
                next_center_y - sin_a * next_half_len
            };

            all_lines.push(Line::new(connect_x, connect_y, next_end_x, next_end_y));
        }

        going_positive = !going_positive;
    }

    // Clip all lines to polygon
    clip_lines_to_polygon(&all_lines, polygon)
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
    fn generates_zigzag_lines() {
        let poly = square_polygon();
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 10.0);
        assert!(!lines.is_empty());
        // Should generate multiple lines for a 100x100 polygon with 10.0 spacing
        assert!(lines.len() >= 5);
    }

    #[test]
    fn zigzag_with_angle() {
        let poly = square_polygon();
        let lines = generate_zigzag_fill(&poly, 10.0, 45.0, 10.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn zigzag_wild_mode() {
        let poly = square_polygon();
        let lines = generate_zigzag_fill_wild(&poly, 10.0, 0.0, 0.5, 12345);
        assert!(!lines.is_empty());

        // Wild mode should produce varied segment lengths
        // Run again with different seed - should get different results
        let lines2 = generate_zigzag_fill_wild(&poly, 10.0, 0.0, 0.5, 54321);
        assert!(!lines2.is_empty());
    }

    #[test]
    fn zigzag_wild_extreme() {
        let poly = square_polygon();
        // High wildness
        let lines = generate_zigzag_fill_wild(&poly, 10.0, 0.0, 1.0, 42);
        assert!(!lines.is_empty());

        // Zero wildness should be like regular mode
        let lines_zero = generate_zigzag_fill_wild(&poly, 10.0, 0.0, 0.0, 42);
        assert!(!lines_zero.is_empty());
    }

    #[test]
    fn zigzag_complex_polygon() {
        // L-shaped polygon
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 50.0),
            Point::new(50.0, 50.0),
            Point::new(50.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 10.0);
        assert!(!lines.is_empty());
    }
}
