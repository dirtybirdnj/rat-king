//! Sketchy/hand-drawn line effect.
//!
//! Provides RoughJS-style line transformation to give vector graphics
//! a hand-drawn appearance. Based on the RoughJS algorithms:
//! - Endpoint randomization (roughness)
//! - Line bowing (curvature)
//! - Double-stroke effect
//!
//! # Example
//! ```ignore
//! use rat_king::sketchy::{SketchyConfig, sketchify_lines};
//! use rat_king::Line;
//!
//! let lines = vec![Line::new(0.0, 0.0, 100.0, 100.0)];
//! let config = SketchyConfig::default();
//! let sketchy_lines = sketchify_lines(&lines, &config);
//! ```

use crate::geometry::{Line, Polygon};
use crate::rng::Rng;

/// Configuration for sketchy/hand-drawn line effect.
///
/// Based on RoughJS algorithms: randomized endpoints, bowing, and double-stroke.
#[derive(Clone, Debug)]
pub struct SketchyConfig {
    /// Base roughness for endpoint randomization (0.0 = smooth, 1.0+ = rough)
    pub roughness: f64,
    /// Maximum perpendicular offset for line bowing (in pixels)
    pub bowing: f64,
    /// Whether to draw each line twice with slight offset
    pub double_stroke: bool,
    /// Seed for reproducible randomness (None = random each time)
    pub seed: Option<u64>,
}

impl Default for SketchyConfig {
    fn default() -> Self {
        Self {
            roughness: 1.0,
            bowing: 1.0,
            double_stroke: true,
            seed: None,
        }
    }
}

impl SketchyConfig {
    /// Create a new config with specified roughness.
    pub fn with_roughness(mut self, roughness: f64) -> Self {
        self.roughness = roughness;
        self
    }

    /// Create a new config with specified bowing.
    pub fn with_bowing(mut self, bowing: f64) -> Self {
        self.bowing = bowing;
        self
    }

    /// Create a new config with double stroke enabled/disabled.
    pub fn with_double_stroke(mut self, double_stroke: bool) -> Self {
        self.double_stroke = double_stroke;
        self
    }

    /// Create a new config with a specific seed.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }
}

/// Convert polygon outlines to Line segments.
///
/// Extracts all edges from the polygon's outer boundary and holes.
pub fn polygon_to_lines(polygon: &Polygon) -> Vec<Line> {
    let mut lines = Vec::new();

    // Convert outer boundary
    let outer = &polygon.outer;
    if outer.len() >= 2 {
        for i in 0..outer.len() {
            let p1 = &outer[i];
            let p2 = &outer[(i + 1) % outer.len()];
            lines.push(Line::new(p1.x, p1.y, p2.x, p2.y));
        }
    }

    // Convert holes
    for hole in &polygon.holes {
        if hole.len() >= 2 {
            for i in 0..hole.len() {
                let p1 = &hole[i];
                let p2 = &hole[(i + 1) % hole.len()];
                lines.push(Line::new(p1.x, p1.y, p2.x, p2.y));
            }
        }
    }

    lines
}

/// Apply sketchy/hand-drawn effect to a single line.
///
/// Returns one or two lines depending on double_stroke setting.
/// Each line is split at the midpoint with bowing applied.
pub fn sketchify_line(line: &Line, config: &SketchyConfig, rng: &mut Rng) -> Vec<Line> {
    let mut result = Vec::new();

    // Calculate line length for dampening factor (longer lines = less roughness per unit)
    let dx = line.x2 - line.x1;
    let dy = line.y2 - line.y1;
    let length = (dx * dx + dy * dy).sqrt();

    if length < 0.001 {
        return result; // Skip zero-length lines
    }

    // Dampening: reduce roughness effect for longer lines
    let dampen = 1.0 / (length / 50.0 + 1.0);
    let effective_roughness = config.roughness * dampen;

    // Calculate perpendicular direction for bowing
    let perp_x = -dy / length;
    let perp_y = dx / length;

    // First stroke
    let offset1 = effective_roughness * rng.next_signed();
    let offset2 = effective_roughness * rng.next_signed();
    let bow_offset = config.bowing * rng.next_signed() * dampen;

    // Randomize endpoints slightly
    let x1 = line.x1 + offset1;
    let y1 = line.y1 + offset1;
    let x2 = line.x2 + offset2;
    let y2 = line.y2 + offset2;

    // Apply bowing at midpoint (simplified - we create two segments)
    let mid_x = (x1 + x2) / 2.0 + perp_x * bow_offset;
    let mid_y = (y1 + y2) / 2.0 + perp_y * bow_offset;

    // First half
    result.push(Line::new(x1, y1, mid_x, mid_y));
    // Second half
    result.push(Line::new(mid_x, mid_y, x2, y2));

    // Double stroke with slight offset
    if config.double_stroke {
        let offset3 = effective_roughness * rng.next_signed() * 0.5;
        let offset4 = effective_roughness * rng.next_signed() * 0.5;
        let bow_offset2 = config.bowing * rng.next_signed() * dampen * 0.7;

        let x1_2 = line.x1 + offset3;
        let y1_2 = line.y1 + offset3;
        let x2_2 = line.x2 + offset4;
        let y2_2 = line.y2 + offset4;

        let mid_x2 = (x1_2 + x2_2) / 2.0 + perp_x * bow_offset2;
        let mid_y2 = (y1_2 + y2_2) / 2.0 + perp_y * bow_offset2;

        result.push(Line::new(x1_2, y1_2, mid_x2, mid_y2));
        result.push(Line::new(mid_x2, mid_y2, x2_2, y2_2));
    }

    result
}

/// Apply sketchy effect to all lines.
///
/// Transforms a collection of lines to have a hand-drawn appearance.
pub fn sketchify_lines(lines: &[Line], config: &SketchyConfig) -> Vec<Line> {
    let seed = config.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42)
    });

    let mut rng = Rng::new(seed);
    let mut result = Vec::with_capacity(lines.len() * 4);

    for line in lines {
        result.extend(sketchify_line(line, config, &mut rng));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn sketchify_produces_more_lines() {
        let lines = vec![
            Line::new(0.0, 0.0, 100.0, 0.0),
            Line::new(100.0, 0.0, 100.0, 100.0),
        ];

        let config = SketchyConfig::default().with_seed(42);
        let result = sketchify_lines(&lines, &config);

        // Each line becomes 4 lines (2 segments * 2 strokes)
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn sketchify_without_double_stroke() {
        let lines = vec![Line::new(0.0, 0.0, 100.0, 0.0)];

        let config = SketchyConfig::default()
            .with_double_stroke(false)
            .with_seed(42);
        let result = sketchify_lines(&lines, &config);

        // Each line becomes 2 lines (split at midpoint)
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn deterministic_with_seed() {
        let lines = vec![Line::new(0.0, 0.0, 100.0, 100.0)];
        let config = SketchyConfig::default().with_seed(12345);

        let result1 = sketchify_lines(&lines, &config);
        let result2 = sketchify_lines(&lines, &config);

        assert_eq!(result1.len(), result2.len());
        for (l1, l2) in result1.iter().zip(result2.iter()) {
            assert_eq!(l1.x1, l2.x1);
            assert_eq!(l1.y1, l2.y1);
        }
    }

    #[test]
    fn polygon_to_lines_works() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines = polygon_to_lines(&poly);
        assert_eq!(lines.len(), 4); // Square has 4 edges
    }

    #[test]
    fn roughness_affects_variation() {
        let lines = vec![Line::new(0.0, 0.0, 100.0, 0.0)];

        let smooth = SketchyConfig::default()
            .with_roughness(0.0)
            .with_bowing(0.0)
            .with_seed(42);
        let rough = SketchyConfig::default()
            .with_roughness(5.0)
            .with_bowing(5.0)
            .with_seed(42);

        let smooth_result = sketchify_lines(&lines, &smooth);
        let rough_result = sketchify_lines(&lines, &rough);

        // With zero roughness/bowing, lines should be close to original
        // With high roughness/bowing, lines should vary more
        let smooth_deviation: f64 = smooth_result.iter()
            .map(|l| (l.x1 - 0.0).abs() + (l.y1 - 0.0).abs())
            .sum();
        let rough_deviation: f64 = rough_result.iter()
            .map(|l| (l.x1 - 0.0).abs() + (l.y1 - 0.0).abs())
            .sum();

        assert!(rough_deviation > smooth_deviation);
    }
}
