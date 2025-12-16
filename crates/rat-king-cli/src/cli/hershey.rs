//! Hershey single-line font parser.
//!
//! Parses JHF (Jim Hershey Font) format files to generate SVG paths
//! suitable for plotters and CNC machines. These fonts are single-stroke,
//! meaning they draw the character outline with a single line rather than
//! filled shapes.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A parsed Hershey font containing glyph paths.
#[derive(Debug, Clone)]
pub struct HersheyFont {
    /// Map from ASCII character code to SVG path data
    glyphs: HashMap<u8, GlyphData>,
}

/// Data for a single glyph.
#[derive(Debug, Clone)]
pub struct GlyphData {
    /// SVG path commands (M/L only)
    pub path: String,
    /// Left bound (for spacing calculation)
    pub left: i8,
    /// Right bound (for advance width)
    pub right: i8,
}

impl HersheyFont {
    /// Load a Hershey font from a JHF file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read font file: {}", e))?;
        Self::parse(&content)
    }

    /// Load the default futural font from the known location.
    pub fn load_futural() -> Result<Self, String> {
        // Try common locations
        let paths = [
            std::env::var("HOME").unwrap_or_default() + "/Code/hershey-fonts/hershey-fonts/futural.jhf",
            std::env::var("HOME").unwrap_or_default() + "/writetyper/hershey-fonts/hershey-fonts/futural.jhf",
        ];

        for path in &paths {
            if Path::new(path).exists() {
                return Self::load(path);
            }
        }

        Err("Could not find futural.jhf font file".to_string())
    }

    /// Parse JHF format content.
    fn parse(content: &str) -> Result<Self, String> {
        let mut glyphs = HashMap::new();

        for (index, line) in content.lines().filter(|l| !l.trim().is_empty()).enumerate() {
            // JHF format: "NNNNN  <count><left><right><coords>"
            // Example: "12345  9MWRFRT RRYQZR[SZRY"
            let trimmed = line.trim();

            // Skip lines that don't start with digits
            if !trimmed.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                continue;
            }

            // Find where the data starts (after the line number and spaces)
            let data_start = trimmed.find(|c: char| c.is_ascii_digit())
                .and_then(|_| {
                    // Skip past the digits and spaces
                    let mut chars = trimmed.chars().peekable();
                    let mut pos = 0;
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() {
                            chars.next();
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    // Skip spaces
                    while let Some(&c) = chars.peek() {
                        if c == ' ' {
                            chars.next();
                            pos += 1;
                        } else {
                            break;
                        }
                    }
                    Some(pos)
                });

            let data_start = match data_start {
                Some(p) => p,
                None => continue,
            };

            let data = &trimmed[data_start..];
            if data.len() < 3 {
                continue;
            }

            // First characters are stroke count (variable length digits)
            let stroke_end = data.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
            if stroke_end == 0 {
                continue;
            }

            let rest = &data[stroke_end..];
            if rest.len() < 2 {
                continue;
            }

            // Next two characters are left/right bounds
            let left = rest.chars().next().unwrap() as i8 - 'R' as i8;
            let right = rest.chars().nth(1).unwrap() as i8 - 'R' as i8;
            let coords = &rest[2..];

            // Parse coordinates into path
            let path = parse_hershey_coords(coords);

            // Glyph index + 31 = ASCII code (line 1 = space = ASCII 32)
            let char_code = (index as u8).wrapping_add(32);

            glyphs.insert(char_code, GlyphData { path, left, right });
        }

        Ok(HersheyFont { glyphs })
    }

    /// Get glyph data for a character.
    pub fn get_glyph(&self, c: char) -> Option<&GlyphData> {
        if c.is_ascii() {
            self.glyphs.get(&(c as u8))
        } else {
            None
        }
    }

    /// Render text as SVG path elements.
    ///
    /// Returns a vector of SVG `<path>` element strings, one per character.
    /// The paths are positioned for the given starting point and font size.
    pub fn render_text(&self, text: &str, x: f64, y: f64, font_size: f64) -> Vec<String> {
        let scale = font_size / 21.0; // Hershey fonts are roughly 21 units tall
        let mut paths = Vec::new();
        let mut cursor_x = x;

        for c in text.chars() {
            if let Some(glyph) = self.get_glyph(c) {
                if !glyph.path.is_empty() {
                    // Transform the path to the correct position and scale
                    let transformed = transform_path(&glyph.path, cursor_x, y, scale);
                    paths.push(transformed);
                }
                // Advance cursor by glyph width
                let width = (glyph.right - glyph.left) as f64;
                cursor_x += width * scale;
            } else if c == ' ' {
                // Default space width
                cursor_x += 10.0 * scale;
            }
        }

        paths
    }

    /// Get the width of rendered text.
    pub fn text_width(&self, text: &str, font_size: f64) -> f64 {
        let scale = font_size / 21.0;
        let mut width = 0.0;

        for c in text.chars() {
            if let Some(glyph) = self.get_glyph(c) {
                width += (glyph.right - glyph.left) as f64 * scale;
            } else if c == ' ' {
                width += 10.0 * scale;
            }
        }

        width
    }

    /// Render text as a single combined SVG path.
    pub fn render_text_path(&self, text: &str, x: f64, y: f64, font_size: f64) -> String {
        let scale = font_size / 21.0;
        let mut combined = String::new();
        let mut cursor_x = x;

        for c in text.chars() {
            if let Some(glyph) = self.get_glyph(c) {
                if !glyph.path.is_empty() {
                    let transformed = transform_path_data(&glyph.path, cursor_x, y, scale);
                    if !combined.is_empty() {
                        combined.push(' ');
                    }
                    combined.push_str(&transformed);
                }
                let width = (glyph.right - glyph.left) as f64;
                cursor_x += width * scale;
            } else if c == ' ' {
                cursor_x += 10.0 * scale;
            }
        }

        combined
    }
}

/// Parse Hershey coordinate pairs into SVG path data.
fn parse_hershey_coords(coords: &str) -> String {
    let mut commands = Vec::new();
    let mut pen_up = true;
    let chars: Vec<char> = coords.chars().collect();
    let mut i = 0;

    while i + 1 < chars.len() {
        let c1 = chars[i];
        let c2 = chars[i + 1];

        // ' R' means pen up (move to next stroke)
        if c1 == ' ' && c2 == 'R' {
            pen_up = true;
            i += 2;
            continue;
        }

        // Convert from Hershey coordinates (R = origin, ASCII 82)
        let x = c1 as i32 - 'R' as i32;
        let y = c2 as i32 - 'R' as i32;

        if pen_up {
            commands.push(format!("M {} {}", x, y));
            pen_up = false;
        } else {
            commands.push(format!("L {} {}", x, y));
        }

        i += 2;
    }

    commands.join(" ")
}

/// Transform path data to a new position and scale.
fn transform_path(path: &str, x: f64, y: f64, scale: f64) -> String {
    let data = transform_path_data(path, x, y, scale);
    format!("<path d=\"{}\" />", data)
}

/// Transform just the path data (d attribute content).
fn transform_path_data(path: &str, x: f64, y: f64, scale: f64) -> String {
    let mut result = String::new();

    for part in path.split_whitespace() {
        if part.starts_with('M') || part.starts_with('L') {
            result.push_str(part);
            result.push(' ');
        } else if let Ok(val) = part.parse::<f64>() {
            // Check if this is an X or Y coordinate by counting previous numbers
            let num_count = result.matches(char::is_numeric).count()
                + result.matches('-').count()
                + result.matches('.').count();

            // Simple heuristic: odd position = X, even = Y
            // Actually, we need to track M/L commands properly
            result.push_str(&format!("{:.2} ", val));
        } else {
            result.push_str(part);
            result.push(' ');
        }
    }

    // Parse and transform properly
    let mut transformed = String::new();
    let tokens: Vec<&str> = path.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let cmd = tokens[i];
        if (cmd == "M" || cmd == "L") && i + 2 < tokens.len() {
            let px: f64 = tokens[i + 1].parse().unwrap_or(0.0);
            let py: f64 = tokens[i + 2].parse().unwrap_or(0.0);
            let tx = x + px * scale;
            let ty = y + py * scale;
            transformed.push_str(&format!("{} {:.2} {:.2} ", cmd, tx, ty));
            i += 3;
        } else {
            i += 1;
        }
    }

    transformed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coords() {
        // Simple test with known coordinates
        let coords = "RFRT"; // Two points: R,F and R,T
        let path = parse_hershey_coords(coords);
        assert!(path.contains("M "));
        assert!(path.contains("L "));
    }

    #[test]
    fn test_font_loading() {
        // This test will only work if the font file exists
        if let Ok(font) = HersheyFont::load_futural() {
            // Check that we got some glyphs
            assert!(font.get_glyph('A').is_some());
            assert!(font.get_glyph('a').is_some());
            assert!(font.get_glyph('0').is_some());
        }
    }
}
