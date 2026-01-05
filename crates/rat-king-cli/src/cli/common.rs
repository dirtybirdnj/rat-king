//! Common utilities shared across CLI commands.

use rat_king::{Chain, Line, Pattern, Polygon};

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

/// Convert lines to SVG output (individual <line> elements).
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

/// Convert chains to SVG output (polyline elements).
///
/// This produces much smaller output than individual lines by chaining
/// connected line segments into continuous polylines.
pub fn chains_to_svg(chains: &[Chain], original_svg: &str) -> String {
    let viewbox = extract_viewbox(original_svg).unwrap_or_else(|| "0 0 1000 1000".to_string());

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">
<g stroke="black" stroke-width="0.5" fill="none">
"#,
        viewbox
    ));

    for chain in chains {
        if chain.len() < 2 {
            continue;
        }

        // Build points string: "x1,y1 x2,y2 x3,y3 ..."
        let points: String = chain
            .iter()
            .map(|p| format!("{:.2},{:.2}", p.x, p.y))
            .collect::<Vec<_>>()
            .join(" ");

        svg.push_str(&format!("  <polyline points=\"{}\"/>\n", points));
    }

    svg.push_str("</g>\n</svg>\n");
    svg
}

/// A group of chains with styling information.
pub struct StyledGroup {
    pub group_id: String,
    pub chains: Vec<Chain>,
    pub color: String,
}

/// Convert grouped chains to SVG output with per-group colors.
pub fn grouped_chains_to_svg(groups: &[StyledGroup], original_svg: &str) -> String {
    let viewbox = extract_viewbox(original_svg).unwrap_or_else(|| "0 0 1000 1000".to_string());

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">
"#,
        viewbox
    ));

    for group in groups {
        svg.push_str(&format!(
            r#"<g id="{}" stroke="{}" stroke-width="0.5" fill="none" stroke-linecap="round">
"#,
            group.group_id, group.color
        ));

        for chain in &group.chains {
            if chain.len() < 2 {
                continue;
            }

            let points: String = chain
                .iter()
                .map(|p| format!("{:.2},{:.2}", p.x, p.y))
                .collect::<Vec<_>>()
                .join(" ");

            svg.push_str(&format!("  <polyline points=\"{}\"/>\n", points));
        }

        svg.push_str("</g>\n");
    }

    svg.push_str("</svg>\n");
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
