//! Generate pattern swatch sheets for documentation and reference.
//!
//! Creates a grid of swatches showing all available patterns
//! with labels below each swatch. Fits on 8.5"×11" letter paper.

use std::fs;

use rat_king::{Pattern, Point, Polygon};

/// Page size constants (8.5" × 11" letter)
const PAGE_WIDTH: f64 = 8.5 * 72.0;  // 612 pts
const PAGE_HEIGHT: f64 = 11.0 * 72.0; // 792 pts

/// Swatch configuration for 5×6 grid on letter paper
const SWATCH_SIZE: f64 = 108.0; // 1.5 inches = 108 pts
const LABEL_HEIGHT: f64 = 12.0; // Space for text below swatch
const GUTTER: f64 = 6.0; // Space between swatches
const MARGIN: f64 = 18.0; // Page margins (0.25")

const COLUMNS: usize = 5;
const ROWS: usize = 6;

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
            "-h" | "--help" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Use fixed 8.5×11" letter page
    let cell_width = SWATCH_SIZE + GUTTER;
    let cell_height = SWATCH_SIZE + LABEL_HEIGHT + GUTTER;

    eprintln!("Generating pattern swatches...");
    eprintln!("  Page: 8.5\" × 11\" (letter)");
    eprintln!("  Swatch size: {:.2}\" × {:.2}\"", SWATCH_SIZE / 72.0, SWATCH_SIZE / 72.0);
    eprintln!("  Grid: {}×{} ({} patterns)", COLUMNS, ROWS, COLUMNS * ROWS);
    eprintln!("  Spacing: {}, Angle: {}°", spacing, angle);

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

    // Generate each swatch
    for (idx, pattern) in patterns.iter().enumerate() {
        let col = idx % COLUMNS;
        let row = idx / COLUMNS;

        if row >= ROWS {
            break; // Don't exceed grid
        }

        let x = MARGIN + (col as f64 * cell_width);
        let y = MARGIN + (row as f64 * cell_height);

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

        eprint!(".");
    }

    svg_content.push_str("</svg>\n");
    eprintln!(" done!");

    // Write SVG
    fs::write(&output_path, &svg_content).expect("Failed to write SVG");
    eprintln!("Wrote: {}", output_path);

    // Generate PNG if requested
    if let Some(png_path) = png_output {
        generate_png(&svg_content, &png_path, png_scale, PAGE_WIDTH, PAGE_HEIGHT);
    }
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
    eprintln!("    -o, --output <file>    Output SVG file (default: pattern_swatches.svg)");
    eprintln!("    -c, --colorful         Use vibrant colors (one per pattern)");
    eprintln!("    --stroke <color>       Line color (default: black)");
    eprintln!("    --fill <color>         Swatch background (default: none/white)");
    eprintln!("    -w, --stroke-width <n> Line width (default: 1.0)");
    eprintln!("    -s, --spacing <n>      Pattern spacing (default: 4.0)");
    eprintln!("    -a, --angle <deg>      Pattern angle (default: 45)");
    eprintln!("    --png <file>           Also generate PNG output");
    eprintln!("    --png-scale <n>        PNG scale factor (default: 2.0)");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    # Generate B&W swatch sheet");
    eprintln!("    rat-king swatches -o swatches_bw.svg");
    eprintln!();
    eprintln!("    # Generate colorful swatch sheet for README");
    eprintln!("    rat-king swatches --colorful -o swatches_color.svg --png swatches.png");
    eprintln!();
    eprintln!("    # Generate SVG and PNG");
    eprintln!("    rat-king swatches -o swatches.svg --png swatches.png --png-scale 3");
    eprintln!();
    eprintln!("OUTPUT:");
    eprintln!("    Creates a {}×{} grid of {:.2}\"×{:.2}\" swatches with pattern labels.",
        COLUMNS, ROWS, SWATCH_SIZE / 72.0, SWATCH_SIZE / 72.0);
    eprintln!("    Page size: 8.5\" × 11\" (fits on letter paper)");
}
