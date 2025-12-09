//! Common utilities shared across CLI commands.

use rat_king::{Line, Pattern, Polygon};

/// Output format for generated lines.
#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Svg,
    Json,
}

/// Generate pattern lines for a polygon.
///
/// This is a thin wrapper around Pattern::generate() for CLI use.
pub fn generate_pattern(pattern: Pattern, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
    pattern.generate(polygon, spacing, angle)
}

/// Convert lines to SVG output.
pub fn lines_to_svg(lines: &[Line], original_svg: &str) -> String {
    let viewbox = extract_viewbox(original_svg).unwrap_or_else(|| "0 0 1000 1000".to_string());

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">
<g stroke="black" stroke-width="0.5" fill="none">
"#,
        viewbox
    ));

    for line in lines {
        svg.push_str(&format!(
            "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
            line.x1, line.y1, line.x2, line.y2
        ));
    }

    svg.push_str("</g>\n</svg>\n");
    svg
}

/// Extract viewBox from SVG content.
pub fn extract_viewbox(svg: &str) -> Option<String> {
    // Try viewBox (camelCase)
    if let Some(start) = svg.find("viewBox=\"") {
        let rest = &svg[start + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    // Try viewbox (lowercase)
    if let Some(start) = svg.find("viewbox=\"") {
        let rest = &svg[start + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}
