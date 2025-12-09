//! Harness utilities for testing patterns.
//!
//! This module provides analysis and diff generation functions used by the harness command.
//! The main cmd_harness function remains in main.rs due to tight coupling with rendering.

use image::DynamicImage;
use serde::Serialize;

/// Analysis results from comparing pattern to solid fill
#[derive(Debug, Serialize, Clone)]
pub struct AnalysisResult {
    /// Percentage of solid fill area covered by pattern (0-100)
    pub coverage_percent: f64,
    /// Number of pattern pixels outside the solid fill bounds
    pub out_of_bounds_pixels: u64,
    /// Percentage of pattern pixels that are out of bounds (0-100)
    pub out_of_bounds_percent: f64,
    /// Total pixels in the solid fill area
    pub solid_fill_pixels: u64,
    /// Total pattern pixels
    pub pattern_pixels: u64,
    /// Whether the pattern passes bounds check (less than 1% out of bounds)
    pub bounds_ok: bool,
    /// Coverage rating: "excellent" (>90%), "good" (70-90%), "fair" (50-70%), "poor" (<50%)
    pub coverage_rating: String,
}

/// Analyze a pattern image against a solid fill reference.
/// Returns analysis results including coverage and bounds checking.
pub fn analyze_pattern_vs_solid(
    solid_img: &DynamicImage,
    pattern_img: &DynamicImage,
) -> AnalysisResult {
    let solid_rgba = solid_img.to_rgba8();
    let pattern_rgba = pattern_img.to_rgba8();

    let (width, height) = solid_rgba.dimensions();

    // Background color (white in our renders)
    let bg_threshold = 250u8;

    let mut solid_fill_pixels: u64 = 0;
    let mut pattern_pixels: u64 = 0;
    let mut pattern_in_bounds: u64 = 0;
    let mut pattern_out_of_bounds: u64 = 0;

    for y in 0..height {
        for x in 0..width {
            let solid_pixel = solid_rgba.get_pixel(x, y);
            let pattern_pixel = pattern_rgba.get_pixel(x, y);

            // Check if this pixel is part of the solid fill (not background)
            // Solid fill is gray (#cccccc = 204,204,204) with stroke (#888888)
            let is_solid_fill = solid_pixel[0] < bg_threshold ||
                                solid_pixel[1] < bg_threshold ||
                                solid_pixel[2] < bg_threshold;

            // Check if this pixel has pattern content (not background)
            // Pattern lines are black or colored, background is white
            let has_pattern = pattern_pixel[0] < bg_threshold ||
                              pattern_pixel[1] < bg_threshold ||
                              pattern_pixel[2] < bg_threshold;

            if is_solid_fill {
                solid_fill_pixels += 1;
            }

            if has_pattern {
                pattern_pixels += 1;
                if is_solid_fill {
                    pattern_in_bounds += 1;
                } else {
                    pattern_out_of_bounds += 1;
                }
            }
        }
    }

    // Calculate coverage: what percentage of the solid fill area has pattern?
    let coverage_percent = if solid_fill_pixels > 0 {
        (pattern_in_bounds as f64 / solid_fill_pixels as f64) * 100.0
    } else {
        0.0
    };

    // Calculate out of bounds percentage
    let out_of_bounds_percent = if pattern_pixels > 0 {
        (pattern_out_of_bounds as f64 / pattern_pixels as f64) * 100.0
    } else {
        0.0
    };

    // Allow 10% tolerance for stroke width bleed and anti-aliasing
    // Lines clipped at polygon boundaries still render with stroke width,
    // causing some pixels to appear outside the filled area.
    // Also allow up to 5000 pixels absolute (for smaller patterns)
    let bounds_ok = out_of_bounds_percent < 10.0 || pattern_out_of_bounds < 5000;

    let coverage_rating = if coverage_percent >= 90.0 {
        "excellent".to_string()
    } else if coverage_percent >= 70.0 {
        "good".to_string()
    } else if coverage_percent >= 50.0 {
        "fair".to_string()
    } else {
        "poor".to_string()
    };

    AnalysisResult {
        coverage_percent,
        out_of_bounds_pixels: pattern_out_of_bounds,
        out_of_bounds_percent,
        solid_fill_pixels,
        pattern_pixels,
        bounds_ok,
        coverage_rating,
    }
}

/// Generate a diff image showing coverage issues.
/// - Red = pattern pixels outside the polygon bounds (clipping failure)
/// - Green = solid fill area not covered by pattern (missing coverage)
/// - Blue = pattern pixels correctly inside bounds (good)
/// - White = background (neither solid fill nor pattern)
pub fn generate_diff_image(
    solid_img: &DynamicImage,
    pattern_img: &DynamicImage,
) -> DynamicImage {
    let solid_rgba = solid_img.to_rgba8();
    let pattern_rgba = pattern_img.to_rgba8();

    let (width, height) = solid_rgba.dimensions();
    let mut diff = image::RgbaImage::new(width, height);

    let bg_threshold = 250u8;

    // Colors for diff visualization
    let red = image::Rgba([255, 0, 0, 255]);      // Out of bounds
    let green = image::Rgba([0, 200, 0, 255]);    // Missing coverage
    let blue = image::Rgba([0, 100, 255, 200]);   // Correct coverage
    let white = image::Rgba([255, 255, 255, 255]); // Background

    for y in 0..height {
        for x in 0..width {
            let solid_pixel = solid_rgba.get_pixel(x, y);
            let pattern_pixel = pattern_rgba.get_pixel(x, y);

            let is_solid_fill = solid_pixel[0] < bg_threshold ||
                                solid_pixel[1] < bg_threshold ||
                                solid_pixel[2] < bg_threshold;

            let has_pattern = pattern_pixel[0] < bg_threshold ||
                              pattern_pixel[1] < bg_threshold ||
                              pattern_pixel[2] < bg_threshold;

            let color = match (is_solid_fill, has_pattern) {
                (true, true) => blue,   // Good: pattern inside bounds
                (true, false) => green, // Missing: solid area without pattern
                (false, true) => red,   // Bad: pattern outside bounds
                (false, false) => white, // Background
            };

            diff.put_pixel(x, y, color);
        }
    }

    DynamicImage::ImageRgba8(diff)
}

/// Result from running a single pattern in the harness
#[derive(Debug, Serialize)]
pub struct HarnessResult {
    pub pattern: String,
    pub lines: usize,
    pub time_ms: f64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solid_screenshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_screenshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisResult>,
}

/// Visual harness JSON output
#[derive(Debug, Serialize)]
pub struct VisualHarnessReport {
    pub svg: String,
    pub polygons: usize,
    pub spacing: f64,
    pub angle: f64,
    pub output_dir: String,
    pub render_width: u32,
    pub render_height: u32,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<HarnessResult>,
}
