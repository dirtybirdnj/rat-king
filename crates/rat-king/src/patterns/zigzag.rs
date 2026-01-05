//! Zigzag fill pattern - continuous horizontal zigzag lines.
//!
//! Creates parallel rows of continuous zigzag lines that span horizontally,
//! with sharp triangular peaks alternating up and down.

use std::f64::consts::PI;
use crate::geometry::{Line, Point, Polygon};
use crate::clip::clip_lines_to_polygon;
use crate::rng::Rng;

/// Configuration for zigzag pattern.
#[derive(Debug, Clone, Copy)]
pub struct ZigzagConfig {
    /// Vertical spacing between zigzag rows
    pub spacing: f64,
    /// Rotation angle in degrees
    pub angle_degrees: f64,
    /// Height of zigzag peaks (half the peak-to-peak amplitude)
    pub amplitude: f64,
    /// Horizontal distance between peaks
    pub wavelength: f64,
    /// Enable wild mode - randomly vary parameters
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
            amplitude: 2.5,
            wavelength: 5.0,
            wild: false,
            wildness: 0.5,
            seed: 42,
        }
    }
}

/// Generate zigzag fill for a polygon.
///
/// Creates continuous horizontal zigzag lines with sharp triangular peaks.
pub fn generate_zigzag_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
    _amplitude: f64,
) -> Vec<Line> {
    // Spacing is the vertical distance between zigzag rows
    // Amplitude is half the peak-to-trough height (set to ~40% of spacing for clean separation)
    // Wavelength is horizontal distance for one full zigzag cycle
    let config = ZigzagConfig {
        spacing: spacing * 2.5,  // More space between rows
        angle_degrees,
        amplitude: spacing * 0.8,  // Visible but not overlapping
        wavelength: spacing * 3.0, // Wider zigzags
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
        amplitude: spacing * 0.4,
        wavelength: spacing * 1.2,
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

    // Calculate bounds with rotation consideration
    let diagonal = (width * width + height * height).sqrt();
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;

    // Direction vectors for rotation
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let mut rng = Rng::new(config.seed);
    let mut all_lines = Vec::new();

    // Number of rows needed to cover the rotated area
    let half_coverage = diagonal * 0.75;
    let num_rows = (half_coverage * 2.0 / config.spacing).ceil() as i32 + 2;

    // Generate zigzag rows
    for row in -num_rows..=num_rows {
        let row_offset = row as f64 * config.spacing;

        // Apply wild variation to row position
        let actual_offset = if config.wild {
            let var = config.wildness * config.spacing * 0.15;
            row_offset + (rng.next_f64() * 2.0 - 1.0) * var
        } else {
            row_offset
        };

        // Row parameters
        let amp = if config.wild {
            let var = config.wildness * config.amplitude * 0.3;
            (config.amplitude + (rng.next_f64() * 2.0 - 1.0) * var).max(0.5)
        } else {
            config.amplitude
        };

        let wl = if config.wild {
            let var = config.wildness * config.wavelength * 0.2;
            (config.wavelength + (rng.next_f64() * 2.0 - 1.0) * var).max(1.0)
        } else {
            config.wavelength
        };

        // Generate zigzag points for this row
        // Start from left edge, go to right edge
        let half_wl = wl / 2.0;
        let num_peaks = (half_coverage * 2.0 / half_wl).ceil() as i32 + 2;

        // Alternate starting direction for adjacent rows (creates nicer pattern)
        let phase = if row % 2 == 0 { 1.0 } else { -1.0 };

        for i in -num_peaks..num_peaks {
            // Local coordinates (before rotation)
            let x1 = i as f64 * half_wl;
            let x2 = (i + 1) as f64 * half_wl;

            // Y oscillates between +amp and -amp
            let y1 = actual_offset + if i % 2 == 0 { amp } else { -amp } * phase;
            let y2 = actual_offset + if (i + 1) % 2 == 0 { amp } else { -amp } * phase;

            // Apply rotation around center
            let (rx1, ry1) = rotate_point(x1, y1, center_x, center_y, cos_a, sin_a);
            let (rx2, ry2) = rotate_point(x2, y2, center_x, center_y, cos_a, sin_a);

            all_lines.push(Line::new(rx1, ry1, rx2, ry2));
        }
    }

    // Clip all lines to polygon
    clip_lines_to_polygon(&all_lines, polygon)
}

/// Rotate a point around a center.
fn rotate_point(x: f64, y: f64, cx: f64, cy: f64, cos_a: f64, sin_a: f64) -> (f64, f64) {
    let dx = x;
    let dy = y;
    (
        cx + dx * cos_a - dy * sin_a,
        cy + dx * sin_a + dy * cos_a,
    )
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
        let lines = generate_zigzag_fill(&poly, 10.0, 0.0, 5.0);
        assert!(!lines.is_empty());
        assert!(lines.len() >= 10, "Should generate many zigzag segments");
    }

    #[test]
    fn zigzag_with_angle() {
        let poly = square_polygon();
        let lines = generate_zigzag_fill(&poly, 10.0, 45.0, 5.0);
        assert!(!lines.is_empty());
    }

    #[test]
    fn zigzag_wild_mode() {
        let poly = square_polygon();
        let lines = generate_zigzag_fill_wild(&poly, 10.0, 0.0, 0.5, 12345);
        assert!(!lines.is_empty());
    }
}
