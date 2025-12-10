//! Generate randomized pattern banners for decorative use.
//!
//! Creates horizontal strips of random patterns, useful as visual dividers.

use std::fs;
use rand::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

use rat_king::{Pattern, Point, Polygon};

/// Vibrant color palette (same as swatches)
const COLORS: &[&str] = &[
    "#E63946", "#F4A261", "#2A9D8F", "#264653", "#E9C46A",
    "#8338EC", "#FF006E", "#3A86FF", "#06D6A0", "#FB5607",
    "#7209B7", "#00B4D8", "#90BE6D", "#F72585", "#4361EE",
    "#4CC9F0", "#FFD166", "#EF476F", "#118AB2", "#073B4C",
    "#9B5DE5", "#00F5D4", "#FEE440", "#F15BB5", "#00BBF9",
    "#9EF01A", "#FF595E", "#1982C4", "#6A4C93", "#FFCA3A",
];

/// Execute the banner command.
pub fn cmd_banner(args: &[String]) {
    let mut output_path = "pattern_banner.svg".to_string();
    let mut width_inches = 12.0_f64;
    let mut height_inches = 1.0_f64;
    let mut cell_count = 50_usize;
    let mut spacing = 3.0_f64;
    let mut png_output: Option<String> = None;
    let mut png_scale = 2.0_f64;
    let mut seed: Option<u64> = None;
    let mut whitelist: Option<Vec<String>> = None;
    let mut blacklist: Option<Vec<String>> = None;

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
            "-w" | "--width" => {
                i += 1;
                if i < args.len() {
                    width_inches = args[i].parse().unwrap_or(12.0);
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    height_inches = args[i].parse().unwrap_or(1.0);
                }
            }
            "-n" | "--cells" => {
                i += 1;
                if i < args.len() {
                    cell_count = args[i].parse().unwrap_or(50);
                }
            }
            "-s" | "--spacing" => {
                i += 1;
                if i < args.len() {
                    spacing = args[i].parse().unwrap_or(3.0);
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
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = args[i].parse().ok();
                }
            }
            "--whitelist" | "--only" => {
                i += 1;
                if i < args.len() {
                    whitelist = Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
                }
            }
            "--blacklist" | "--exclude" => {
                i += 1;
                if i < args.len() {
                    blacklist = Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
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

    // Set up RNG
    let mut rng: Box<dyn RngCore> = match seed {
        Some(s) => Box::new(StdRng::seed_from_u64(s)),
        None => Box::new(StdRng::from_os_rng()),
    };

    // Calculate dimensions
    let width_pts = width_inches * 72.0;
    let height_pts = height_inches * 72.0;
    let cell_width = width_pts / cell_count as f64;

    eprintln!("Generating pattern banner...");
    eprintln!("  Size: {}\" × {}\" ({} × {} pts)", width_inches, height_inches, width_pts as i32, height_pts as i32);
    eprintln!("  Cells: {} @ {:.1}pts each", cell_count, cell_width);

    // Get all patterns and filter by whitelist/blacklist
    let all_patterns = Pattern::all();
    let patterns: Vec<&Pattern> = all_patterns
        .iter()
        .filter(|p| {
            let name = p.name();
            // If whitelist is set, pattern must be in it
            if let Some(ref wl) = whitelist {
                if !wl.iter().any(|w| w.eq_ignore_ascii_case(name)) {
                    return false;
                }
            }
            // If blacklist is set, pattern must not be in it
            if let Some(ref bl) = blacklist {
                if bl.iter().any(|b| b.eq_ignore_ascii_case(name)) {
                    return false;
                }
            }
            true
        })
        .collect();

    if patterns.is_empty() {
        eprintln!("Error: No patterns available after filtering!");
        eprintln!("  Check your --whitelist/--blacklist arguments.");
        return;
    }

    eprintln!("  Patterns: {} available", patterns.len());
    let mut svg_content = String::new();

    // SVG header
    svg_content.push_str(&format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{:.2}" height="{:.2}"
     viewBox="0 0 {:.2} {:.2}">
  <title>Pattern Banner - rat-king</title>
  <desc>Randomized pattern strip with {} cells</desc>
  <rect width="100%" height="100%" fill="white"/>
"##,
        width_pts, height_pts, width_pts, height_pts, cell_count
    ));

    // Generate random cells
    for idx in 0..cell_count {
        let x = idx as f64 * cell_width;

        // Random pattern and color
        let pattern = patterns.choose(&mut *rng).unwrap();
        let color = COLORS[rng.random_range(0..COLORS.len())];
        let angle = rng.random_range(0..180i32) as f64;

        // Create cell polygon
        let cell = Polygon::new(vec![
            Point::new(x, 0.0),
            Point::new(x + cell_width, 0.0),
            Point::new(x + cell_width, height_pts),
            Point::new(x, height_pts),
        ]);

        // Generate pattern lines
        let lines = pattern.generate(&cell, spacing, angle);

        // Add cell
        svg_content.push_str(&format!(
            "  <g stroke=\"{}\" stroke-width=\"1\" fill=\"none\" stroke-linecap=\"round\">\n",
            color
        ));

        for line in &lines {
            svg_content.push_str(&format!(
                "    <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                line.x1, line.y1, line.x2, line.y2
            ));
        }

        svg_content.push_str("  </g>\n");
    }

    svg_content.push_str("</svg>\n");
    eprintln!("Done!");

    // Write SVG
    fs::write(&output_path, &svg_content).expect("Failed to write SVG");
    eprintln!("Wrote: {}", output_path);

    // Generate PNG if requested
    if let Some(png_path) = png_output {
        generate_png(&svg_content, &png_path, png_scale, width_pts, height_pts);
    }
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
    eprintln!("rat-king banner - Generate randomized pattern banner");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    rat-king banner [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -o, --output <file>    Output SVG file (default: pattern_banner.svg)");
    eprintln!("    -w, --width <inches>   Banner width (default: 12)");
    eprintln!("    --height <inches>      Banner height (default: 1)");
    eprintln!("    -n, --cells <n>        Number of pattern cells (default: 50)");
    eprintln!("    -s, --spacing <n>      Pattern line spacing (default: 3.0)");
    eprintln!("    --png <file>           Also generate PNG output");
    eprintln!("    --png-scale <n>        PNG scale factor (default: 2.0)");
    eprintln!("    --seed <n>             Random seed for reproducibility");
    eprintln!();
    eprintln!("PATTERN FILTERING:");
    eprintln!("    --whitelist <p1,p2>    Only use these patterns (comma-separated)");
    eprintln!("    --only <p1,p2>         Alias for --whitelist");
    eprintln!("    --blacklist <p1,p2>    Exclude these patterns (comma-separated)");
    eprintln!("    --exclude <p1,p2>      Alias for --blacklist");
    eprintln!();
    eprintln!("    Pattern names: lines, crosshatch, zigzag, wiggle, spiral, fermat,");
    eprintln!("    concentric, radial, honeycomb, crossspiral, hilbert, guilloche,");
    eprintln!("    lissajous, rose, phyllotaxis, scribble, gyroid, pentagon15,");
    eprintln!("    pentagon14, grid, brick, truchet, stipple, peano, sierpinski,");
    eprintln!("    diagonal, herringbone, stripe, tessellation, harmonograph");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    # Generate random banner");
    eprintln!("    rat-king banner -o hr1.svg --png hr1.png");
    eprintln!();
    eprintln!("    # Only use simple line patterns");
    eprintln!("    rat-king banner --only lines,crosshatch,zigzag,diagonal -o simple.svg");
    eprintln!();
    eprintln!("    # Exclude slow/complex patterns");
    eprintln!("    rat-king banner --exclude hilbert,peano,sierpinski -o fast.svg");
    eprintln!();
    eprintln!("    # Reproducible banner with seed");
    eprintln!("    rat-king banner --seed 42 -o hr_seed42.svg");
}
