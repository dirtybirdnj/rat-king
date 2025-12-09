//! Common utilities shared across CLI commands.

use rat_king::{
    Line, Pattern, Polygon,
    patterns::{
        generate_lines_fill, generate_crosshatch_fill,
        generate_zigzag_fill, generate_wiggle_fill,
        generate_spiral_fill, generate_fermat_fill,
        generate_concentric_fill, generate_radial_fill,
        generate_honeycomb_fill, generate_scribble_fill,
        generate_crossspiral_fill, generate_hilbert_fill,
        generate_gyroid_fill, generate_guilloche_fill,
        generate_lissajous_fill, generate_rose_fill,
        generate_phyllotaxis_fill, generate_pentagon15_fill,
        generate_pentagon14_fill, generate_grid_fill,
        generate_brick_fill, generate_truchet_fill,
        generate_stipple_fill, generate_peano_fill,
        generate_sierpinski_fill, generate_diagonal_fill,
        generate_herringbone_fill, generate_stripe_fill,
        generate_tessellation_fill, generate_harmonograph_fill,
    },
};

/// Output format for generated lines.
#[derive(Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Svg,
    Json,
}

/// Generate pattern lines for a polygon.
pub fn generate_pattern(pattern: Pattern, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
    match pattern {
        Pattern::Lines => generate_lines_fill(polygon, spacing, angle),
        Pattern::Crosshatch => generate_crosshatch_fill(polygon, spacing, angle),
        Pattern::Zigzag => generate_zigzag_fill(polygon, spacing, angle, spacing),
        Pattern::Wiggle => generate_wiggle_fill(polygon, spacing, angle, spacing, 0.1),
        Pattern::Spiral => generate_spiral_fill(polygon, spacing, angle),
        Pattern::Fermat => generate_fermat_fill(polygon, spacing, angle),
        Pattern::Concentric => generate_concentric_fill(polygon, spacing, true),
        Pattern::Radial => generate_radial_fill(polygon, 10.0, angle),
        Pattern::Honeycomb => generate_honeycomb_fill(polygon, spacing * 4.0, angle),
        Pattern::Crossspiral => generate_crossspiral_fill(polygon, spacing, angle),
        Pattern::Hilbert => generate_hilbert_fill(polygon, spacing, angle),
        Pattern::Guilloche => generate_guilloche_fill(polygon, spacing, angle),
        Pattern::Lissajous => generate_lissajous_fill(polygon, spacing, angle),
        Pattern::Rose => generate_rose_fill(polygon, spacing, angle),
        Pattern::Phyllotaxis => generate_phyllotaxis_fill(polygon, spacing, angle),
        Pattern::Scribble => generate_scribble_fill(polygon, spacing, angle),
        Pattern::Gyroid => generate_gyroid_fill(polygon, spacing, angle),
        Pattern::Pentagon15 => generate_pentagon15_fill(polygon, spacing * 3.0, angle),
        Pattern::Pentagon14 => generate_pentagon14_fill(polygon, spacing * 3.0, angle),
        Pattern::Grid => generate_grid_fill(polygon, spacing, angle),
        Pattern::Brick => generate_brick_fill(polygon, spacing, angle),
        Pattern::Truchet => generate_truchet_fill(polygon, spacing * 2.0, angle),
        Pattern::Stipple => generate_stipple_fill(polygon, spacing, angle),
        Pattern::Peano => generate_peano_fill(polygon, spacing, angle),
        Pattern::Sierpinski => generate_sierpinski_fill(polygon, spacing, angle),
        Pattern::Diagonal => generate_diagonal_fill(polygon, spacing, angle),
        Pattern::Herringbone => generate_herringbone_fill(polygon, spacing * 2.0, angle),
        Pattern::Stripe => generate_stripe_fill(polygon, spacing * 2.0, angle),
        Pattern::Tessellation => generate_tessellation_fill(polygon, spacing, angle),
        Pattern::Harmonograph => generate_harmonograph_fill(polygon, spacing, angle),
    }
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
