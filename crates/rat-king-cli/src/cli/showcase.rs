//! Generate pattern showcase pages with density progression.
//!
//! Creates a page for each pattern showing the same pattern at increasing
//! detail levels (varying spacing). Uses Hershey single-line fonts for
//! plotter-friendly text labels.

use std::fs;

use rat_king::{Pattern, Point, Polygon};

use super::hershey::HersheyFont;

/// Page size constants (8.5" x 11" letter)
const PAGE_WIDTH: f64 = 8.5 * 72.0;  // 612 pts
const PAGE_HEIGHT: f64 = 11.0 * 72.0; // 792 pts

/// Tile configuration for 3x4 grid (12 density levels)
const TILE_SIZE: f64 = 144.0; // 2 inches = 144 pts
const LABEL_HEIGHT: f64 = 18.0; // Space for text below tile
const GUTTER: f64 = 8.0; // Space between tiles
const MARGIN: f64 = 24.0; // Page margins

const COLUMNS: usize = 3;
const ROWS: usize = 4;

/// Density levels to show (spacing values)
/// Lower spacing = more detail/density
const DENSITY_LEVELS: &[f64] = &[
    12.0, 10.0, 8.0,    // Sparse row
    6.0, 5.0, 4.0,      // Medium row
    3.0, 2.5, 2.0,      // Dense row
    1.5, 1.0, 0.75,     // Very dense row
];

/// Pattern-specific notes for edge cases and special features.
fn get_pattern_notes(pattern: &Pattern) -> &'static str {
    match pattern {
        Pattern::Lines | Pattern::Crosshatch | Pattern::Diagonal =>
            "angle: rotation | simple, fast",
        Pattern::Zigzag =>
            "amplitude varies | sharp corners",
        Pattern::Wiggle =>
            "smooth sine waves | resolution varies",
        Pattern::Spiral =>
            "archimedean | center artifacts",
        Pattern::Fermat =>
            "parabolic spiral | bidirectional",
        Pattern::Concentric =>
            "follows shape | no angle param",
        Pattern::Radial =>
            "ray count fixed | center origin",
        Pattern::Honeycomb =>
            "hex cells | edge clipping",
        Pattern::Crossspiral =>
            "dual spiral arms | moire effects",
        Pattern::Hilbert =>
            "space-filling | recursive depth",
        Pattern::Guilloche =>
            "spirograph-like | high complexity",
        Pattern::Lissajous =>
            "frequency ratio | closed loops",
        Pattern::Rose =>
            "petal count | odd/even differ",
        Pattern::Phyllotaxis =>
            "golden angle | sunflower seeds",
        Pattern::Scribble =>
            "random chaos | non-deterministic",
        Pattern::Gyroid =>
            "3D projection | organic curves",
        Pattern::Pentagon15 =>
            "penrose P3 | aperiodic tiling",
        Pattern::Pentagon14 =>
            "cairo tiling | periodic",
        Pattern::Grid =>
            "square cells | axis-aligned",
        Pattern::Brick =>
            "running bond | offset rows",
        Pattern::Truchet =>
            "random tiles | connected arcs",
        Pattern::Stipple =>
            "dots only | no lines",
        Pattern::Peano =>
            "space-filling | recursive",
        Pattern::Sierpinski =>
            "arrowhead curve | fractal",
        Pattern::Herringbone =>
            "chevron pattern | 45 deg typical",
        Pattern::Stripe =>
            "grouped bands | thick/thin",
        Pattern::Tessellation =>
            "triangulate | adaptive fill",
        Pattern::Harmonograph =>
            "decaying pendulum | chaotic",
        Pattern::Flowfield =>
            "perlin noise | organic flow",
        Pattern::Voronoi =>
            "cell boundaries | random seeds",
        Pattern::Gosper =>
            "flowsnake curve | space-filling",
        Pattern::Wave =>
            "interference | multiple sources",
        Pattern::Sunburst =>
            "radial rays | center origin",
    }
}

/// Execute the showcase command.
pub fn cmd_showcase(args: &[String]) {
    let mut output_dir = ".".to_string();
    let mut stroke_color = "black".to_string();
    let mut stroke_width = 0.75_f64;
    let mut angle = 45.0_f64;
    let mut png_output = false;
    let mut png_scale = 2.0_f64;
    let mut pattern_filter: Option<Pattern> = None;
    let mut use_hershey = true;
    let mut combined = false;

    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_dir = args[i].clone();
                }
            }
            "-p" | "--pattern" => {
                i += 1;
                if i < args.len() {
                    pattern_filter = Pattern::from_name(&args[i]);
                }
            }
            "--stroke" => {
                i += 1;
                if i < args.len() {
                    stroke_color = args[i].clone();
                }
            }
            "--stroke-width" | "-w" => {
                i += 1;
                if i < args.len() {
                    stroke_width = args[i].parse().unwrap_or(0.75);
                }
            }
            "-a" | "--angle" => {
                i += 1;
                if i < args.len() {
                    angle = args[i].parse().unwrap_or(45.0);
                }
            }
            "--png" => {
                png_output = true;
            }
            "--png-scale" => {
                i += 1;
                if i < args.len() {
                    png_scale = args[i].parse().unwrap_or(2.0);
                }
            }
            "--system-font" => {
                use_hershey = false;
            }
            "--combined" | "-c" => {
                combined = true;
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    // Load Hershey font if using single-line text
    let font = if use_hershey {
        match HersheyFont::load_futural() {
            Ok(f) => {
                eprintln!("Loaded Hershey font (single-line)");
                Some(f)
            }
            Err(e) => {
                eprintln!("Warning: Could not load Hershey font: {}", e);
                eprintln!("Falling back to system font");
                None
            }
        }
    } else {
        None
    };

    // Determine which patterns to generate
    let patterns: Vec<Pattern> = if let Some(p) = pattern_filter {
        vec![p]
    } else {
        Pattern::all().to_vec()
    };

    if combined {
        generate_combined_showcase(&patterns, &output_dir, &stroke_color, stroke_width, angle, font.as_ref(), png_output, png_scale);
    } else {
        for pattern in &patterns {
            generate_pattern_showcase(
                pattern,
                &output_dir,
                &stroke_color,
                stroke_width,
                angle,
                font.as_ref(),
                png_output,
                png_scale,
            );
        }
    }

    eprintln!("Done! Generated {} showcase page(s)", patterns.len());
}

/// Generate a showcase page for a single pattern.
fn generate_pattern_showcase(
    pattern: &Pattern,
    output_dir: &str,
    stroke_color: &str,
    stroke_width: f64,
    angle: f64,
    font: Option<&HersheyFont>,
    png_output: bool,
    png_scale: f64,
) {
    let pattern_name = pattern.name();
    let metadata = pattern.metadata();
    let notes = get_pattern_notes(pattern);

    eprintln!("Generating showcase for: {}", pattern_name);

    let cell_width = TILE_SIZE + GUTTER;
    let cell_height = TILE_SIZE + LABEL_HEIGHT + GUTTER;

    let mut svg_content = String::new();

    // SVG header
    svg_content.push_str(&format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{:.2}" height="{:.2}"
     viewBox="0 0 {:.2} {:.2}">
  <title>{} Pattern Showcase - rat-king</title>
  <desc>Density progression for {} pattern. {}</desc>

  <!-- Background -->
  <rect width="100%" height="100%" fill="white"/>

  <!-- Header -->
"##,
        PAGE_WIDTH, PAGE_HEIGHT, PAGE_WIDTH, PAGE_HEIGHT,
        pattern_name, pattern_name, metadata.description
    ));

    // Add header with pattern name and notes
    add_header(&mut svg_content, pattern_name, notes, font);

    svg_content.push_str("\n  <!-- Density Tiles -->\n");

    // Generate each density tile
    for (idx, &spacing) in DENSITY_LEVELS.iter().enumerate() {
        let col = idx % COLUMNS;
        let row = idx / COLUMNS;

        if row >= ROWS {
            break;
        }

        let x = MARGIN + (col as f64 * cell_width);
        let y = MARGIN + 48.0 + (row as f64 * cell_height); // 48pt for header

        // Create unique tile ID: pattern-spacing-uuid
        let tile_id = format!("{}-s{:.2}-{:02}", pattern_name, spacing, idx);
        let density_label = format!("s={:.2}", spacing);

        // Create square polygon for this tile
        let square = create_square(x, y, TILE_SIZE);

        // Generate pattern lines
        let lines = pattern.generate(&square, spacing, angle);

        // Add tile group with unique ID
        svg_content.push_str(&format!(
            r##"  <g id="{}" data-pattern="{}" data-spacing="{:.2}" data-lines="{}">
    <!-- Tile border -->
    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}"
          fill="none" stroke="#cccccc" stroke-width="0.5"/>

    <!-- Pattern lines -->
    <g stroke="{}" stroke-width="{}" fill="none" stroke-linecap="round">
"##,
            tile_id, pattern_name, spacing, lines.len(),
            x, y, TILE_SIZE, TILE_SIZE,
            stroke_color, stroke_width
        ));

        // Add pattern lines
        for line in &lines {
            svg_content.push_str(&format!(
                "      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                line.x1, line.y1, line.x2, line.y2
            ));
        }

        svg_content.push_str("    </g>\n");

        // Add label with density and line count
        let label_x = x + TILE_SIZE / 2.0;
        let label_y = y + TILE_SIZE + LABEL_HEIGHT - 4.0;
        let label_text = format!("{} ({})", density_label, lines.len());

        add_label(&mut svg_content, label_x, label_y, &label_text, font);

        svg_content.push_str("  </g>\n");

        eprint!(".");
    }

    svg_content.push_str("</svg>\n");
    eprintln!(" done!");

    // Write SVG
    let output_path = format!("{}/showcase_{}.svg", output_dir, pattern_name);
    fs::write(&output_path, &svg_content).expect("Failed to write SVG");
    eprintln!("Wrote: {}", output_path);

    // Generate PNG if requested
    if png_output {
        let png_path = format!("{}/showcase_{}.png", output_dir, pattern_name);
        generate_png(&svg_content, &png_path, png_scale, PAGE_WIDTH, PAGE_HEIGHT);
    }
}

/// Generate a combined showcase with all patterns on one page per density level.
fn generate_combined_showcase(
    patterns: &[Pattern],
    output_dir: &str,
    stroke_color: &str,
    stroke_width: f64,
    angle: f64,
    font: Option<&HersheyFont>,
    png_output: bool,
    png_scale: f64,
) {
    eprintln!("Generating combined showcase pages...");

    for (density_idx, &spacing) in DENSITY_LEVELS.iter().enumerate() {
        let mut svg_content = String::new();

        // SVG header
        svg_content.push_str(&format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{:.2}" height="{:.2}"
     viewBox="0 0 {:.2} {:.2}">
  <title>All Patterns - Spacing {:.2} - rat-king</title>
  <desc>All patterns at spacing {:.2}</desc>

  <!-- Background -->
  <rect width="100%" height="100%" fill="white"/>

"##,
            PAGE_WIDTH, PAGE_HEIGHT, PAGE_WIDTH, PAGE_HEIGHT,
            spacing, spacing
        ));

        // Add header
        let header_text = format!("Density: spacing={:.2}", spacing);
        if let Some(f) = font {
            let path_data = f.render_text_path(&header_text, MARGIN, 28.0, 14.0);
            if !path_data.is_empty() {
                svg_content.push_str(&format!(
                    "  <path d=\"{}\" stroke=\"{}\" stroke-width=\"0.75\" fill=\"none\"/>\n",
                    path_data, stroke_color
                ));
            }
        } else {
            svg_content.push_str(&format!(
                r##"  <text x="{:.2}" y="28" font-family="system-ui, sans-serif" font-size="14" fill="#333">{}</text>
"##,
                MARGIN, header_text
            ));
        }

        // Use same grid as swatches (5x7 = 35 patterns max)
        let cols = 5;
        let tile_size = 100.0;
        let cell_width = tile_size + 6.0;
        let cell_height = tile_size + 14.0 + 6.0;

        for (idx, pattern) in patterns.iter().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            let x = MARGIN + (col as f64 * cell_width);
            let y = MARGIN + 40.0 + (row as f64 * cell_height);

            let tile_id = format!("{}-s{:.2}-d{:02}", pattern.name(), spacing, density_idx);
            let square = create_square(x, y, tile_size);
            let lines = pattern.generate(&square, spacing, angle);

            svg_content.push_str(&format!(
                r##"  <g id="{}" data-pattern="{}" data-spacing="{:.2}" data-lines="{}">
    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}"
          fill="none" stroke="#ddd" stroke-width="0.5"/>
    <g stroke="{}" stroke-width="{}" fill="none" stroke-linecap="round">
"##,
                tile_id, pattern.name(), spacing, lines.len(),
                x, y, tile_size, tile_size,
                stroke_color, stroke_width
            ));

            for line in &lines {
                svg_content.push_str(&format!(
                    "      <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                    line.x1, line.y1, line.x2, line.y2
                ));
            }

            svg_content.push_str("    </g>\n");

            // Label
            let label_x = x + tile_size / 2.0;
            let label_y = y + tile_size + 10.0;
            add_label(&mut svg_content, label_x, label_y, pattern.name(), font);

            svg_content.push_str("  </g>\n");
        }

        svg_content.push_str("</svg>\n");

        let output_path = format!("{}/showcase_all_s{:.2}.svg", output_dir, spacing);
        fs::write(&output_path, &svg_content).expect("Failed to write SVG");
        eprintln!("Wrote: {}", output_path);

        if png_output {
            let png_path = format!("{}/showcase_all_s{:.2}.png", output_dir, spacing);
            generate_png(&svg_content, &png_path, png_scale, PAGE_WIDTH, PAGE_HEIGHT);
        }
    }
}

/// Add header with pattern name and notes.
fn add_header(svg: &mut String, pattern_name: &str, notes: &str, font: Option<&HersheyFont>) {
    if let Some(f) = font {
        // Render pattern name in Hershey font
        let name_path = f.render_text_path(pattern_name, MARGIN, 28.0, 18.0);
        if !name_path.is_empty() {
            svg.push_str(&format!(
                "  <path d=\"{}\" stroke=\"black\" stroke-width=\"1\" fill=\"none\"/>\n",
                name_path
            ));
        }

        // Render notes in smaller Hershey font
        let notes_path = f.render_text_path(notes, MARGIN, 42.0, 10.0);
        if !notes_path.is_empty() {
            svg.push_str(&format!(
                "  <path d=\"{}\" stroke=\"#666\" stroke-width=\"0.5\" fill=\"none\"/>\n",
                notes_path
            ));
        }
    } else {
        // Fallback to system fonts
        svg.push_str(&format!(
            r##"  <text x="{:.2}" y="28"
        font-family="system-ui, -apple-system, sans-serif"
        font-size="18" font-weight="bold"
        fill="#333333">{}</text>
  <text x="{:.2}" y="42"
        font-family="system-ui, -apple-system, sans-serif"
        font-size="10"
        fill="#666666">{}</text>
"##,
            MARGIN, pattern_name,
            MARGIN, notes
        ));
    }
}

/// Add a centered label below a tile.
fn add_label(svg: &mut String, x: f64, y: f64, text: &str, font: Option<&HersheyFont>) {
    if let Some(f) = font {
        // Calculate width to center the text
        let text_width = f.text_width(text, 9.0);
        let start_x = x - text_width / 2.0;
        let path = f.render_text_path(text, start_x, y, 9.0);
        if !path.is_empty() {
            svg.push_str(&format!(
                "    <path d=\"{}\" stroke=\"#333\" stroke-width=\"0.5\" fill=\"none\"/>\n",
                path
            ));
        }
    } else {
        svg.push_str(&format!(
            r##"    <text x="{:.2}" y="{:.2}"
          font-family="system-ui, -apple-system, sans-serif"
          font-size="9"
          text-anchor="middle"
          fill="#333333">{}</text>
"##,
            x, y, text
        ));
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
    eprintln!("rat-king showcase - Generate pattern detail progression pages");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    rat-king showcase [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -o, --output <dir>     Output directory (default: current)");
    eprintln!("    -p, --pattern <name>   Generate only this pattern");
    eprintln!("    -c, --combined         Generate combined pages (all patterns per density)");
    eprintln!("    --stroke <color>       Line color (default: black)");
    eprintln!("    -w, --stroke-width <n> Line width (default: 0.75)");
    eprintln!("    -a, --angle <deg>      Pattern angle (default: 45)");
    eprintln!("    --png                  Also generate PNG output");
    eprintln!("    --png-scale <n>        PNG scale factor (default: 2.0)");
    eprintln!("    --system-font          Use system fonts instead of Hershey");
    eprintln!();
    eprintln!("OUTPUT:");
    eprintln!("    For each pattern, creates a page showing 12 density levels");
    eprintln!("    (spacing from 12.0 down to 0.75) in a 3x4 grid.");
    eprintln!();
    eprintln!("    Each tile has a unique ID: <pattern>-s<spacing>-<index>");
    eprintln!("    Example: zigzag-s4.00-05");
    eprintln!();
    eprintln!("    Uses Hershey single-line fonts for plotter-friendly labels.");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    # Generate all pattern showcases");
    eprintln!("    rat-king showcase -o ./showcases/");
    eprintln!();
    eprintln!("    # Generate just the zigzag showcase with PNG");
    eprintln!("    rat-king showcase -p zigzag --png");
    eprintln!();
    eprintln!("    # Generate combined pages (all patterns at each density)");
    eprintln!("    rat-king showcase --combined -o ./density_comparison/");
}
