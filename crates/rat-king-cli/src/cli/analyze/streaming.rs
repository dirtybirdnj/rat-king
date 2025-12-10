//! Streaming SVG analyzer using quick-xml.
//!
//! This module provides O(1) memory analysis of SVG files by streaming
//! through the XML without building a full DOM tree. Ideal for large files.

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

use super::types::{ColorInfo, ElementCounts, GroupInfo, SvgSummary, ViewBox};

/// Streaming analyzer for SVG files.
///
/// Uses quick-xml to parse SVG content without building a full tree,
/// enabling analysis of very large files with minimal memory usage.
pub struct StreamingAnalyzer {
    element_counts: ElementCounts,
    top_groups: Vec<GroupInfo>,
    fill_colors: HashMap<String, usize>,
    stroke_colors: HashMap<String, usize>,
    view_box: Option<ViewBox>,
    width: Option<String>,
    height: Option<String>,
    transform_count: usize,
    depth: usize,
    current_group_child_count: usize,
    in_defs: bool,
    warnings: Vec<String>,
}

impl StreamingAnalyzer {
    pub fn new() -> Self {
        Self {
            element_counts: ElementCounts::default(),
            top_groups: Vec::new(),
            fill_colors: HashMap::new(),
            stroke_colors: HashMap::new(),
            view_box: None,
            width: None,
            height: None,
            transform_count: 0,
            depth: 0,
            current_group_child_count: 0,
            in_defs: false,
            warnings: Vec::new(),
        }
    }

    /// Analyze SVG content using streaming parser.
    ///
    /// Returns summary statistics without building a full tree.
    pub fn analyze(&mut self, content: &str, file_size: u64) -> Result<SvgSummary, String> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    self.process_start_element(e, false)?;
                }
                Ok(Event::Empty(ref e)) => {
                    self.process_start_element(e, true)?;
                }
                Ok(Event::End(ref e)) => {
                    self.process_end_element(e);
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error at position {}: {}", reader.error_position(), e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(self.build_summary(file_size))
    }

    fn process_start_element(
        &mut self,
        e: &quick_xml::events::BytesStart,
        is_empty: bool,
    ) -> Result<(), String> {
        let name_bytes = e.name();
        let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");

        // Track depth for group hierarchy
        if !is_empty {
            self.depth += 1;
        }

        // Track if we're inside <defs>
        if name == "defs" {
            self.in_defs = true;
        }

        match name {
            "svg" => self.process_svg_attrs(e)?,
            "g" => self.process_group(e)?,
            "path" => {
                self.element_counts.paths += 1;
                self.extract_colors(e)?;
            }
            "circle" => {
                self.element_counts.circles += 1;
                self.extract_colors(e)?;
            }
            "rect" => {
                self.element_counts.rects += 1;
                self.extract_colors(e)?;
            }
            "ellipse" => {
                self.element_counts.ellipses += 1;
                self.extract_colors(e)?;
            }
            "line" => {
                self.element_counts.lines += 1;
                self.extract_colors(e)?;
            }
            "polyline" => {
                self.element_counts.polylines += 1;
                self.extract_colors(e)?;
            }
            "polygon" => {
                self.element_counts.polygons += 1;
                self.extract_colors(e)?;
            }
            "text" | "tspan" => {
                self.element_counts.text += 1;
            }
            "image" => {
                self.element_counts.images += 1;
            }
            "use" => {
                self.element_counts.use_elements += 1;
            }
            "defs" => {
                self.element_counts.defs += 1;
            }
            "clipPath" => {
                self.element_counts.clip_paths += 1;
            }
            "mask" => {
                self.element_counts.masks += 1;
            }
            "linearGradient" | "radialGradient" => {
                self.element_counts.gradients += 1;
            }
            "pattern" => {
                self.element_counts.patterns += 1;
            }
            _ => {}
        }

        // Check for transform attribute on any element
        for attr in e.attributes().flatten() {
            if attr.key.as_ref() == b"transform" {
                self.transform_count += 1;
            }
        }

        self.element_counts.total += 1;
        Ok(())
    }

    fn process_end_element(&mut self, e: &quick_xml::events::BytesEnd) {
        let name_bytes = e.name();
        let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");

        if name == "defs" {
            self.in_defs = false;
        }

        self.depth = self.depth.saturating_sub(1);
    }

    fn process_svg_attrs(&mut self, e: &quick_xml::events::BytesStart) -> Result<(), String> {
        for attr in e.attributes().flatten() {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = std::str::from_utf8(&attr.value).unwrap_or("");

            match key {
                "viewBox" | "viewbox" => {
                    self.view_box = parse_viewbox(value);
                    if self.view_box.is_none() {
                        self.warnings.push(format!("Invalid viewBox: {}", value));
                    }
                }
                "width" => {
                    self.width = Some(value.to_string());
                }
                "height" => {
                    self.height = Some(value.to_string());
                }
                _ => {}
            }
        }

        // Warn if no viewBox
        if self.view_box.is_none() && self.width.is_none() && self.height.is_none() {
            self.warnings.push("SVG has no viewBox or dimensions".to_string());
        }

        Ok(())
    }

    fn process_group(&mut self, e: &quick_xml::events::BytesStart) -> Result<(), String> {
        self.element_counts.groups += 1;

        // Track top-level groups (depth == 2: svg > g)
        if self.depth == 2 && !self.in_defs {
            let mut id: Option<String> = None;
            let mut has_transform = false;

            for attr in e.attributes().flatten() {
                let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                match key {
                    "id" => {
                        id = Some(std::str::from_utf8(&attr.value).unwrap_or("").to_string());
                    }
                    "transform" => {
                        has_transform = true;
                    }
                    _ => {}
                }
            }

            self.top_groups.push(GroupInfo {
                id,
                child_count: 0, // Will be updated later or left as estimate
                has_transform,
            });
        }

        Ok(())
    }

    fn extract_colors(&mut self, e: &quick_xml::events::BytesStart) -> Result<(), String> {
        // Skip elements inside <defs> for color counting
        if self.in_defs {
            return Ok(());
        }

        for attr in e.attributes().flatten() {
            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
            let value = std::str::from_utf8(&attr.value).unwrap_or("");

            match key {
                "fill" => {
                    if !value.is_empty() && value != "none" {
                        let normalized = normalize_color(value);
                        *self.fill_colors.entry(normalized).or_insert(0) += 1;
                    }
                }
                "stroke" => {
                    if !value.is_empty() && value != "none" {
                        let normalized = normalize_color(value);
                        *self.stroke_colors.entry(normalized).or_insert(0) += 1;
                    }
                }
                "style" => {
                    // Parse inline CSS style for fill/stroke
                    self.parse_style_attr(value);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn parse_style_attr(&mut self, style: &str) {
        // Simple CSS parsing for fill: and stroke: properties
        for part in style.split(';') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix("fill:") {
                let value = value.trim();
                if !value.is_empty() && value != "none" {
                    let normalized = normalize_color(value);
                    *self.fill_colors.entry(normalized).or_insert(0) += 1;
                }
            } else if let Some(value) = part.strip_prefix("stroke:") {
                let value = value.trim();
                if !value.is_empty() && value != "none" {
                    let normalized = normalize_color(value);
                    *self.stroke_colors.entry(normalized).or_insert(0) += 1;
                }
            }
        }
    }

    fn build_summary(&self, file_size: u64) -> SvgSummary {
        let mut summary = SvgSummary::new(file_size);

        summary.view_box = self.view_box.clone();
        summary.width = self.width.clone();
        summary.height = self.height.clone();
        summary.element_counts = self.element_counts.clone();
        summary.top_level_groups = self.top_groups.clone();
        summary.transform_count = self.transform_count;
        summary.warnings = self.warnings.clone();

        // Convert color maps to sorted vectors
        summary.fill_colors = colors_to_sorted_vec(&self.fill_colors);
        summary.stroke_colors = colors_to_sorted_vec(&self.stroke_colors);

        summary
    }
}

/// Parse viewBox attribute value.
fn parse_viewbox(value: &str) -> Option<ViewBox> {
    // viewBox can be comma or space separated
    let parts: Vec<&str> = value.split(|c| c == ',' || c == ' ')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if parts.len() == 4 {
        Some(ViewBox {
            min_x: parts[0].parse().ok()?,
            min_y: parts[1].parse().ok()?,
            width: parts[2].parse().ok()?,
            height: parts[3].parse().ok()?,
        })
    } else {
        None
    }
}

/// Normalize color value (lowercase, trim whitespace).
fn normalize_color(color: &str) -> String {
    let color = color.trim();

    // Check for url() references (gradients, patterns)
    if color.starts_with("url(") {
        return "url-reference".to_string();
    }

    // Normalize hex colors to lowercase
    if color.starts_with('#') {
        return color.to_lowercase();
    }

    // Keep named colors as-is but lowercase
    color.to_lowercase()
}

/// Convert color HashMap to sorted Vec by count (descending).
fn colors_to_sorted_vec(colors: &HashMap<String, usize>) -> Vec<ColorInfo> {
    let mut vec: Vec<ColorInfo> = colors
        .iter()
        .map(|(color, &count)| ColorInfo {
            color: color.clone(),
            count,
        })
        .collect();

    // Sort by count descending, then by color name
    vec.sort_by(|a, b| {
        b.count.cmp(&a.count).then_with(|| a.color.cmp(&b.color))
    });

    vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_viewbox() {
        let vb = parse_viewbox("0 0 100 200").unwrap();
        assert_eq!(vb.min_x, 0.0);
        assert_eq!(vb.min_y, 0.0);
        assert_eq!(vb.width, 100.0);
        assert_eq!(vb.height, 200.0);

        let vb = parse_viewbox("0, 0, 100, 200").unwrap();
        assert_eq!(vb.width, 100.0);
    }

    #[test]
    fn test_normalize_color() {
        assert_eq!(normalize_color("#FF0000"), "#ff0000");
        assert_eq!(normalize_color("  red  "), "red");
        assert_eq!(normalize_color("url(#gradient1)"), "url-reference");
    }

    #[test]
    fn test_basic_analysis() {
        let svg = concat!(
            "<svg viewBox=\"0 0 100 100\">",
            "<g id=\"Layer1\">",
            "<path fill=\"#FF0000\" d=\"M0,0 L100,100\"/>",
            "<rect fill=\"#00FF00\" x=\"0\" y=\"0\" width=\"10\" height=\"10\"/>",
            "</g>",
            "</svg>"
        );

        let mut analyzer = StreamingAnalyzer::new();
        let summary = analyzer.analyze(svg, svg.len() as u64).unwrap();

        assert_eq!(summary.element_counts.paths, 1);
        assert_eq!(summary.element_counts.rects, 1);
        assert_eq!(summary.element_counts.groups, 1);
        assert_eq!(summary.top_level_groups.len(), 1);
        assert_eq!(summary.top_level_groups[0].id, Some("Layer1".to_string()));
        assert_eq!(summary.fill_colors.len(), 2);
    }
}
