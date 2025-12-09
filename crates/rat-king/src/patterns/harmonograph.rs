//! Harmonograph fill pattern.
//!
//! A harmonograph simulates the motion of a pendulum drawing on paper.
//! The pattern is created by combining multiple decaying sinusoidal oscillations:
//!
//!   x(t) = A1*sin(f1*t + p1)*exp(-d1*t) + A2*sin(f2*t + p2)*exp(-d2*t)
//!   y(t) = A3*sin(f3*t + p3)*exp(-d3*t) + A4*sin(f4*t + p4)*exp(-d4*t)
//!
//! The decay creates spiraling patterns that converge toward the center,
//! producing elegant, organic-looking curves reminiscent of spirograph drawings.

use std::f64::consts::PI;
use crate::geometry::{Line, Polygon};
use crate::clip::point_in_polygon;

/// Harmonograph pendulum configuration.
struct Pendulum {
    amplitude: f64,
    frequency: f64,
    phase: f64,
    damping: f64,
}

impl Pendulum {
    fn new(amplitude: f64, frequency: f64, phase: f64, damping: f64) -> Self {
        Self { amplitude, frequency, phase, damping }
    }

    /// Calculate pendulum position at time t.
    fn evaluate(&self, t: f64) -> f64 {
        self.amplitude * (self.frequency * t + self.phase).sin() * (-self.damping * t).exp()
    }
}

/// Generate harmonograph fill for a polygon.
///
/// Creates spirograph-like decaying pendulum patterns that scale to fit the polygon.
///
/// Parameters:
/// - `spacing`: Controls the number of harmonograph curves generated
/// - `angle_degrees`: Rotates the entire pattern and varies frequency ratios
pub fn generate_harmonograph_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let width = max_x - min_x;
    let height = max_y - min_y;

    // Scale amplitude to fit polygon
    let base_amplitude = (width.min(height) / 2.0) * 0.9;

    let mut lines = Vec::new();

    // Number of curves based on spacing (more spacing = fewer curves)
    let num_curves = ((base_amplitude / spacing).ceil() as usize).clamp(1, 12);

    // Base rotation from angle parameter
    let base_phase = angle_degrees * PI / 180.0;

    // Interesting frequency ratio presets that create pleasing patterns
    let presets: [(f64, f64, f64, f64); 6] = [
        (2.0, 3.0, 3.0, 2.0),    // Classic 2:3 ratio
        (2.0, 3.0, 3.0, 4.0),    // Asymmetric
        (3.0, 2.0, 4.0, 3.0),    // Complex interweave
        (4.0, 3.0, 3.0, 4.0),    // Near-circular decay
        (5.0, 4.0, 4.0, 5.0),    // Dense pattern
        (3.0, 4.0, 5.0, 3.0),    // Very complex
    ];

    for curve_idx in 0..num_curves {
        // Vary parameters for each curve
        let scale = 1.0 - (curve_idx as f64 / (num_curves as f64 + 1.0)) * 0.3;
        let amp = base_amplitude * scale;

        // Select preset based on curve index and angle
        let preset_idx = (curve_idx + (angle_degrees as usize / 30)) % presets.len();
        let (f1, f2, f3, f4) = presets[preset_idx];

        // Phase offsets create variety between curves
        let phase_offset = curve_idx as f64 * PI / 6.0;

        // Damping controls how quickly the pattern spirals inward
        // Lower values = longer spirals, higher = tighter
        let damping = 0.002 + (curve_idx as f64 * 0.001);

        let curve_lines = generate_single_harmonograph(
            center_x, center_y,
            amp, f1, f2, f3, f4,
            base_phase + phase_offset,
            damping,
            polygon,
        );

        lines.extend(curve_lines);
    }

    lines
}

/// Generate a single harmonograph curve.
fn generate_single_harmonograph(
    center_x: f64,
    center_y: f64,
    amplitude: f64,
    freq1: f64,
    freq2: f64,
    freq3: f64,
    freq4: f64,
    phase: f64,
    damping: f64,
    polygon: &Polygon,
) -> Vec<Line> {
    // Four pendulums: two for X, two for Y
    // This creates the classic "lateral" harmonograph setup
    let pend_x1 = Pendulum::new(amplitude * 0.6, freq1, phase, damping);
    let pend_x2 = Pendulum::new(amplitude * 0.4, freq2, phase + PI / 4.0, damping * 1.2);
    let pend_y1 = Pendulum::new(amplitude * 0.6, freq3, phase + PI / 2.0, damping);
    let pend_y2 = Pendulum::new(amplitude * 0.4, freq4, phase + PI * 0.75, damping * 1.2);

    let mut lines = Vec::new();

    // Calculate how long to trace based on damping
    // Continue until amplitude decays to ~5% of original
    let max_t = (-0.05_f64.ln()) / damping;
    let steps = ((max_t * 50.0) as usize).clamp(200, 2000);
    let dt = max_t / steps as f64;

    let mut prev_point: Option<(f64, f64)> = None;
    let mut prev_inside = false;

    for i in 0..=steps {
        let t = i as f64 * dt;

        // Sum pendulum contributions
        let x = center_x + pend_x1.evaluate(t) + pend_x2.evaluate(t);
        let y = center_y + pend_y1.evaluate(t) + pend_y2.evaluate(t);

        let current_inside = point_in_polygon(x, y, &polygon.outer);

        if let Some((px, py)) = prev_point {
            if prev_inside && current_inside {
                // Check we're not in a hole
                let mid_x = (px + x) / 2.0;
                let mid_y = (py + y) / 2.0;
                let in_hole = polygon.holes.iter().any(|hole| {
                    point_in_polygon(mid_x, mid_y, hole)
                });
                if !in_hole {
                    lines.push(Line::new(px, py, x, y));
                }
            }
        }

        prev_point = Some((x, y));
        prev_inside = current_inside;
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn generates_harmonograph_lines() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);
        let lines = generate_harmonograph_fill(&poly, 10.0, 0.0);
        assert!(!lines.is_empty(), "Should generate harmonograph lines");
    }

    #[test]
    fn angle_changes_pattern() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let lines_0 = generate_harmonograph_fill(&poly, 15.0, 0.0);
        let lines_45 = generate_harmonograph_fill(&poly, 15.0, 45.0);

        assert!(!lines_0.is_empty());
        assert!(!lines_45.is_empty());

        // Different angles should produce different patterns
        if lines_0.len() > 10 && lines_45.len() > 10 {
            let diff_x = (lines_0[10].x1 - lines_45[10].x1).abs();
            let diff_y = (lines_0[10].y1 - lines_45[10].y1).abs();
            assert!(diff_x > 0.1 || diff_y > 0.1, "Angle should affect pattern");
        }
    }

    #[test]
    fn spacing_affects_density() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
            Point::new(0.0, 100.0),
        ]);

        let sparse = generate_harmonograph_fill(&poly, 40.0, 0.0);
        let dense = generate_harmonograph_fill(&poly, 10.0, 0.0);

        assert!(dense.len() > sparse.len(), "Smaller spacing should produce more lines");
    }

    #[test]
    fn handles_small_polygon() {
        let poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ]);
        // Should handle small polygons gracefully (doesn't panic)
        let _lines = generate_harmonograph_fill(&poly, 5.0, 0.0);
    }

    #[test]
    fn pendulum_decays() {
        let pend = Pendulum::new(100.0, 1.0, 0.0, 0.01);
        let t0 = pend.evaluate(0.0);
        let t100 = pend.evaluate(100.0);

        // Amplitude should decay significantly over time
        assert!(t0.abs() < 100.1); // sin(0) = 0, but check it's bounded
        assert!(t100.abs() < t0.abs() + 50.0, "Pattern should decay over time");
    }
}
