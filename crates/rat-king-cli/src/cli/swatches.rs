//! Generate pattern swatch sheets for documentation and reference.
//!
//! Creates a grid of swatches showing all available patterns
//! with labels below each swatch. Supports SVG and HTML output formats.

use std::fs;

use rat_king::{Pattern, Point, Polygon};

/// Output format for swatches
#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Svg,
    Html,
}

/// Page size constants (8.5" × 11" letter)
const PAGE_WIDTH: f64 = 8.5 * 72.0;  // 612 pts
const PAGE_HEIGHT: f64 = 11.0 * 72.0; // 792 pts

/// Swatch configuration for 6×6 grid on letter paper
const SWATCH_SIZE: f64 = 80.0; // Slightly smaller to fit headers
const LABEL_HEIGHT: f64 = 12.0; // Space for text below swatch
const GUTTER: f64 = 6.0; // Space between swatches
const MARGIN: f64 = 18.0; // Page margins (0.25")
const HEADER_HEIGHT: f64 = 20.0; // Height for rating headers
const DIVIDER_SPACING: f64 = 8.0; // Space around horizontal rules

const COLUMNS: usize = 6;
#[allow(dead_code)]
const ROWS: usize = 6; // Max rows per page (used for reference)

const DEFAULT_SPACING: f64 = 4.0;
const DEFAULT_ANGLE: f64 = 45.0;

/// Vibrant color palette for colorful mode (30 distinct colors)
const COLORS: &[&str] = &[
    "#E63946", // Red
    "#F4A261", // Sandy orange
    "#2A9D8F", // Teal
    "#264653", // Dark blue-gray
    "#E9C46A", // Yellow
    "#8338EC", // Purple
    "#FF006E", // Hot pink
    "#3A86FF", // Bright blue
    "#06D6A0", // Mint green
    "#FB5607", // Orange
    "#7209B7", // Deep purple
    "#00B4D8", // Cyan
    "#90BE6D", // Lime green
    "#F72585", // Magenta
    "#4361EE", // Royal blue
    "#4CC9F0", // Sky blue
    "#FFD166", // Gold
    "#EF476F", // Coral red
    "#118AB2", // Ocean blue
    "#073B4C", // Navy
    "#9B5DE5", // Lavender
    "#00F5D4", // Aqua
    "#FEE440", // Bright yellow
    "#F15BB5", // Pink
    "#00BBF9", // Light blue
    "#9EF01A", // Neon green
    "#FF595E", // Salmon
    "#1982C4", // Steel blue
    "#6A4C93", // Plum
    "#FFCA3A", // Sunflower
];

/// Execute the swatches command.
pub fn cmd_swatches(args: &[String]) {
    let mut output_path = "pattern_swatches.svg".to_string();
    let mut stroke_color = "black".to_string();
    let mut fill_color = "none".to_string();
    let mut stroke_width = 1.0_f64;
    let mut spacing = DEFAULT_SPACING;
    let mut angle = DEFAULT_ANGLE;
    let mut png_output: Option<String> = None;
    let mut png_scale = 2.0_f64; // 2x for decent resolution
    let mut colorful = false;
    let mut format = OutputFormat::Svg;

    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "--stroke" => {
                i += 1;
                if i < args.len() {
                    stroke_color = args[i].clone();
                }
            }
            "--fill" => {
                i += 1;
                if i < args.len() {
                    fill_color = args[i].clone();
                }
            }
            "--stroke-width" | "-w" => {
                i += 1;
                if i < args.len() {
                    stroke_width = args[i].parse().unwrap_or(1.0);
                }
            }
            "-s" | "--spacing" => {
                i += 1;
                if i < args.len() {
                    spacing = args[i].parse().unwrap_or(DEFAULT_SPACING);
                }
            }
            "-a" | "--angle" => {
                i += 1;
                if i < args.len() {
                    angle = args[i].parse().unwrap_or(DEFAULT_ANGLE);
                }
            }
            "--png" => {
                i += 1;
                if i < args.len() {
                    png_output = Some(args[i].clone());
                }
            }
            "--png-scale" => {
                i += 1;
                if i < args.len() {
                    png_scale = args[i].parse().unwrap_or(2.0);
                }
            }
            "--colorful" | "-c" => {
                colorful = true;
            }
            "--html" => {
                format = OutputFormat::Html;
                if output_path == "pattern_swatches.svg" {
                    output_path = "pattern_swatches.html".to_string();
                }
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    eprintln!("Generating pattern swatches ({})...", if format == OutputFormat::Html { "HTML" } else { "SVG" });
    eprintln!("  Swatch size: {:.2}\" × {:.2}\"", SWATCH_SIZE / 72.0, SWATCH_SIZE / 72.0);
    eprintln!("  Spacing: {}, Angle: {}°", spacing, angle);

    let content = match format {
        OutputFormat::Html => generate_html_swatches(spacing, angle, stroke_width, &stroke_color, colorful),
        OutputFormat::Svg => generate_svg_swatches(spacing, angle, stroke_width, &stroke_color, &fill_color, colorful),
    };

    fs::write(&output_path, &content).expect("Failed to write output");
    eprintln!("Wrote: {}", output_path);

    // Generate PNG if requested (SVG only)
    if let Some(png_path) = png_output {
        if format == OutputFormat::Svg {
            generate_png(&content, &png_path, png_scale, PAGE_WIDTH, PAGE_HEIGHT);
        } else {
            eprintln!("Note: PNG output not supported for HTML format");
        }
    }
}

/// Generate HTML swatches with embedded SVG.
fn generate_html_swatches(
    spacing: f64,
    angle: f64,
    stroke_width: f64,
    stroke_color: &str,
    colorful: bool,
) -> String {
    let patterns = Pattern::all();
    let mut html = String::new();

    // HTML header with CSS
    html.push_str(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Pattern Swatches - rat-king</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: system-ui, -apple-system, sans-serif;
            background: white;
            padding: 0.5in;
            max-width: 8.5in;
            margin: 0 auto;
        }
        h1 {
            font-size: 24px;
            margin-bottom: 0.5em;
            color: #333;
        }
        .rating-group {
            margin-bottom: 1.5em;
            page-break-inside: avoid;
        }
        .rating-header {
            font-size: 16px;
            font-weight: bold;
            color: #666;
            margin-bottom: 0.5em;
            padding-bottom: 0.25em;
            border-bottom: 1px solid #ccc;
        }
        .swatches {
            display: grid;
            grid-template-columns: repeat(6, 1fr);
            gap: 8px;
        }
        .swatch {
            text-align: center;
        }
        .swatch svg {
            width: 100%;
            aspect-ratio: 1;
            border: 1px solid #ccc;
            background: white;
        }
        .swatch-label {
            font-size: 10px;
            color: #333;
            margin-top: 4px;
        }
        @media print {
            body { padding: 0.25in; }
            .rating-group { page-break-inside: avoid; }
        }
    </style>
</head>
<body>
    <h1>rat-king Pattern Swatches</h1>
"##);

    // Group patterns by rating
    let mut current_rating: Option<u8> = None;

    for (idx, pattern) in patterns.iter().enumerate() {
        let rating = pattern.rating();

        // Start new rating group if needed
        if current_rating != Some(rating) {
            // Close previous group
            if current_rating.is_some() {
                html.push_str("    </div>\n</div>\n\n");
            }

            // Open new group
            let stars = render_stars(rating);
            html.push_str(&format!(
                r##"<div class="rating-group">
    <div class="rating-header">{}</div>
    <div class="swatches">
"##,
                stars
            ));
            current_rating = Some(rating);
        }

        // Generate swatch SVG
        let swatch_svg = generate_swatch_svg(pattern, spacing, angle, stroke_width, stroke_color, colorful, idx);

        html.push_str(&format!(
            r##"        <div class="swatch">
            {}
            <div class="swatch-label">{}</div>
        </div>
"##,
            swatch_svg, pattern.name()
        ));

        eprint!(".");
    }

    // Close last group
    html.push_str("    </div>\n</div>\n\n");

    html.push_str("</body>\n</html>\n");
    eprintln!(" done!");

    html
}

/// Generate a single swatch as inline SVG.
fn generate_swatch_svg(
    pattern: &Pattern,
    spacing: f64,
    angle: f64,
    stroke_width: f64,
    stroke_color: &str,
    colorful: bool,
    idx: usize,
) -> String {
    let size = SWATCH_SIZE;
    let square = create_square(0.0, 0.0, size);
    let lines = pattern.generate(&square, spacing, angle);

    let color = if colorful {
        COLORS[idx % COLORS.len()]
    } else {
        stroke_color
    };

    let mut svg = format!(
        r##"<svg viewBox="0 0 {:.0} {:.0}" xmlns="http://www.w3.org/2000/svg">
            <g stroke="{}" stroke-width="{}" fill="none" stroke-linecap="round">"##,
        size, size, color, stroke_width
    );

    for line in &lines {
        svg.push_str(&format!(
            r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}"/>"##,
            line.x1, line.y1, line.x2, line.y2
        ));
    }

    svg.push_str("</g></svg>");
    svg
}

/// Generate SVG swatches (original format).
fn generate_svg_swatches(
    spacing: f64,
    angle: f64,
    stroke_width: f64,
    stroke_color: &str,
    fill_color: &str,
    colorful: bool,
) -> String {
    // Use fixed 8.5×11" letter page
    let cell_width = SWATCH_SIZE + GUTTER;
    let cell_height = SWATCH_SIZE + LABEL_HEIGHT + GUTTER;

    let patterns = Pattern::all();
    let mut svg_content = String::new();

    // SVG header
    svg_content.push_str(&format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{:.2}" height="{:.2}"
     viewBox="0 0 {:.2} {:.2}">
  <title>Pattern Swatches - rat-king</title>
  <desc>{:.2}"×{:.2}" swatches of all {} patterns (8.5×11" letter)</desc>

  <!-- Background -->
  <rect width="100%" height="100%" fill="white"/>

  <!-- Swatches -->
"##,
        PAGE_WIDTH, PAGE_HEIGHT, PAGE_WIDTH, PAGE_HEIGHT,
        SWATCH_SIZE / 72.0, SWATCH_SIZE / 72.0, patterns.len()
    ));

    // Track position and rating groups
    let mut current_y = MARGIN;
    let mut current_rating: Option<u8> = None;
    let mut col = 0;

    // Generate each swatch with rating headers
    for (idx, pattern) in patterns.iter().enumerate() {
        let rating = pattern.rating();

        // Check if we need a new rating header
        if current_rating != Some(rating) {
            // Add divider line if not the first group
            if current_rating.is_some() {
                // Move to next row if we're mid-row
                if col > 0 {
                    current_y += cell_height;
                    col = 0;
                }
                current_y += DIVIDER_SPACING;

                // Add horizontal rule
                svg_content.push_str(&format!(
                    r##"  <line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}"
        stroke="#cccccc" stroke-width="1"/>
"##,
                    MARGIN, current_y,
                    PAGE_WIDTH - MARGIN, current_y
                ));
                current_y += DIVIDER_SPACING;
            }

            // Add rating header
            let stars = render_stars(rating);
            svg_content.push_str(&format!(
                r##"  <text x="{:.2}" y="{:.2}"
        font-family="system-ui, -apple-system, sans-serif"
        font-size="14"
        font-weight="bold"
        fill="#666666">{}</text>
"##,
                MARGIN, current_y + HEADER_HEIGHT - 6.0,
                stars
            ));
            current_y += HEADER_HEIGHT;
            current_rating = Some(rating);
            col = 0;
        }

        let x = MARGIN + (col as f64 * cell_width);
        let y = current_y;

        // Create square polygon for this swatch
        let square = create_square(x, y, SWATCH_SIZE);

        // Generate pattern lines
        let lines = pattern.generate(&square, spacing, angle);

        // Select color for this swatch
        let swatch_stroke = if colorful {
            COLORS[idx % COLORS.len()]
        } else {
            &stroke_color
        };

        // Add swatch group
        svg_content.push_str(&format!(
            r##"  <g id="swatch-{}" transform="translate(0,0)">
    <!-- Swatch border -->
    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}"
          fill="{}" stroke="#cccccc" stroke-width="0.5"/>

    <!-- Pattern lines -->
    <g stroke="{}" stroke-width="{}" fill="none" stroke-linecap="round">
"##,
            pattern.name(),
            x, y, SWATCH_SIZE, SWATCH_SIZE,
            fill_color,
            swatch_stroke, stroke_width
        ));

        // Add pattern lines
        for line in &lines {
            svg_content.push_str(&format!(
                "      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                line.x1, line.y1, line.x2, line.y2
            ));
        }

        svg_content.push_str("    </g>\n");

        // Add label
        let label_x = x + SWATCH_SIZE / 2.0;
        let label_y = y + SWATCH_SIZE + LABEL_HEIGHT - 4.0;
        svg_content.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}"
          font-family="system-ui, -apple-system, sans-serif"
          font-size="10"
          text-anchor="middle"
          fill="#333333">{}</text>
  </g>
"##,
            label_x, label_y, pattern.name()
        ));

        // Advance column, wrap to next row if needed
        col += 1;
        if col >= COLUMNS {
            col = 0;
            current_y += cell_height;
        }

        eprint!(".");
    }

    svg_content.push_str("</svg>\n");
    eprintln!(" done!");

    svg_content
}

/// Create a square polygon at the given position.
fn create_square(x: f64, y: f64, size: f64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + size, y),
        Point::new(x + size, y + size),
        Point::new(x, y + size),
    ])
}

/// Render star rating as text (e.g., "★★★★★" for 5 stars).
fn render_stars(rating: u8) -> String {
    let filled = "★".repeat(rating as usize);
    let empty = "☆".repeat(5 - rating as usize);
    format!("{}{}", filled, empty)
}

/// Generate PNG from SVG content using resvg.
fn generate_png(svg_content: &str, png_path: &str, scale: f64, width: f64, height: f64) {
    use resvg::usvg;
    use tiny_skia::Pixmap;

    eprint!("Generating PNG at {}x scale...", scale);

    let options = usvg::Options::default();
    let tree = match usvg::Tree::from_str(svg_content, &options) {
        Ok(t) => t,
        Err(e) => {
            eprintln!(" failed: {}", e);
            return;
        }
    };

    let pixmap_width = (width * scale) as u32;
    let pixmap_height = (height * scale) as u32;

    let mut pixmap = match Pixmap::new(pixmap_width, pixmap_height) {
        Some(p) => p,
        None => {
            eprintln!(" failed: could not create pixmap");
            return;
        }
    };

    // Fill with white background
    pixmap.fill(tiny_skia::Color::WHITE);

    let transform = tiny_skia::Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    match pixmap.save_png(png_path) {
        Ok(_) => eprintln!(" done!\nWrote: {} ({}x{})", png_path, pixmap_width, pixmap_height),
        Err(e) => eprintln!(" failed: {}", e),
    }
}

/// Print usage information.
pub fn print_usage() {
    eprintln!("rat-king swatches - Generate pattern swatch sheet");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    rat-king swatches [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -o, --output <file>    Output file (default: pattern_swatches.svg)");
    eprintln!("    --html                 Output as HTML (easier for PDF conversion)");
    eprintln!("    -c, --colorful         Use vibrant colors (one per pattern)");
    eprintln!("    --stroke <color>       Line color (default: black)");
    eprintln!("    --fill <color>         Swatch background (default: none/white)");
    eprintln!("    -w, --stroke-width <n> Line width (default: 1.0)");
    eprintln!("    -s, --spacing <n>      Pattern spacing (default: 4.0)");
    eprintln!("    -a, --angle <deg>      Pattern angle (default: 45)");
    eprintln!("    --png <file>           Also generate PNG output (SVG only)");
    eprintln!("    --png-scale <n>        PNG scale factor (default: 2.0)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    # Generate B&W SVG swatch sheet");
    eprintln!("    rat-king swatches -o swatches.svg");
    eprintln!();
    eprintln!("    # Generate HTML swatch sheet (for easy PDF conversion)");
    eprintln!("    rat-king swatches --html -o swatches.html");
    eprintln!();
    eprintln!("    # Generate colorful swatch sheet");
    eprintln!("    rat-king swatches --colorful --html -o swatches_color.html");
    eprintln!();
    eprintln!("OUTPUT:");
    eprintln!("    Patterns grouped by quality rating (5 stars to 2 stars).");
    eprintln!("    HTML output can be converted to PDF via browser print.");
}
