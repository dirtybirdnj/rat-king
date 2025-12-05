//! rat-king CLI - pattern generation backend for svg-grouper
//!
//! Usage:
//!   rat-king-cli fill <svg_file> --pattern <pattern> [--output <svg_file>]
//!   rat-king-cli benchmark <svg_file> [--pattern <pattern>]
//!   rat-king-cli patterns

use std::env;
use std::fs;
use std::time::Instant;

use rat_king_core::{
    extract_polygons_from_svg, Line, Pattern, Polygon,
    patterns::{
        generate_lines_fill, generate_crosshatch_fill,
        generate_zigzag_fill, generate_wiggle_fill,
        generate_spiral_fill, generate_fermat_fill,
        generate_concentric_fill, generate_radial_fill,
        generate_honeycomb_fill, generate_scribble_fill,
        generate_crossspiral_fill, generate_hilbert_fill,
        generate_gyroid_fill, generate_guilloche_fill,
    },
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "fill" => cmd_fill(&args[2..]),
        "benchmark" => cmd_benchmark(&args[2..]),
        "patterns" => cmd_patterns(),
        "help" | "--help" | "-h" => print_usage(&args[0]),
        // Default: treat as SVG file for backward compatibility
        path if path.ends_with(".svg") => cmd_benchmark(&args[1..]),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(prog: &str) {
    eprintln!("rat-king-cli - fast pattern generation for SVG polygons");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {} fill <svg_file> -p <pattern> [-o <output.svg>] [-s <spacing>] [-a <angle>]", prog);
    eprintln!("  {} benchmark <svg_file> [-p <pattern>]", prog);
    eprintln!("  {} patterns", prog);
    eprintln!();
    eprintln!("Patterns:");
    eprintln!("  Implemented: lines, crosshatch, zigzag, wiggle, spiral, fermat, concentric, radial,");
    eprintln!("               honeycomb, crossspiral, hilbert, guilloche");
    eprintln!("  Stubs:       scribble, gyroid");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -p, --pattern <name>   Pattern to use (default: lines)");
    eprintln!("  -o, --output <file>    Output SVG file (default: stdout)");
    eprintln!("  -s, --spacing <num>    Line spacing in units (default: 2.5)");
    eprintln!("  -a, --angle <degrees>  Pattern angle (default: 45)");
}

fn cmd_patterns() {
    println!("Available patterns:");
    for pattern in Pattern::all() {
        if pattern.is_stub() {
            println!("  {} (stub)", pattern.name());
        } else {
            println!("  {}", pattern.name());
        }
    }
}

fn cmd_fill(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut pattern_name = "lines";
    let mut spacing = 2.5;
    let mut angle = 45.0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--pattern" => {
                i += 1;
                if i < args.len() {
                    pattern_name = &args[i];
                }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(&args[i]);
                }
            }
            "-s" | "--spacing" => {
                i += 1;
                if i < args.len() {
                    spacing = args[i].parse().unwrap_or(2.5);
                }
            }
            "-a" | "--angle" => {
                i += 1;
                if i < args.len() {
                    angle = args[i].parse().unwrap_or(45.0);
                }
            }
            path => {
                if svg_path.is_none() {
                    svg_path = Some(path);
                }
            }
        }
        i += 1;
    }

    let svg_path = svg_path.unwrap_or_else(|| {
        eprintln!("Error: SVG file required");
        std::process::exit(1);
    });

    let pattern = Pattern::from_name(pattern_name).unwrap_or_else(|| {
        eprintln!("Unknown pattern: {}. Use 'patterns' command to list available patterns.", pattern_name);
        std::process::exit(1);
    });

    // Load SVG
    eprintln!("Loading: {}", svg_path);
    let svg_content = fs::read_to_string(svg_path)
        .expect("Failed to read SVG file");

    let polygons = extract_polygons_from_svg(&svg_content)
        .expect("Failed to parse SVG");

    eprintln!("Loaded {} polygons", polygons.len());

    // Generate pattern
    let start = Instant::now();
    let mut all_lines: Vec<Line> = Vec::new();

    for polygon in &polygons {
        let lines = generate_pattern(pattern, polygon, spacing, angle);
        all_lines.extend(lines);
    }

    let elapsed = start.elapsed();
    eprintln!("Generated {} lines in {:?}", all_lines.len(), elapsed);

    // Output SVG
    let svg_output = lines_to_svg(&all_lines, &svg_content);

    if let Some(output) = output_path {
        fs::write(output, &svg_output).expect("Failed to write output SVG");
        eprintln!("Wrote: {}", output);
    } else {
        // Output to stdout for piping
        println!("{}", svg_output);
    }
}

fn cmd_benchmark(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut pattern_name = "lines";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--pattern" => {
                i += 1;
                if i < args.len() {
                    pattern_name = &args[i];
                }
            }
            path => {
                if svg_path.is_none() {
                    svg_path = Some(path);
                }
            }
        }
        i += 1;
    }

    let svg_path = svg_path.unwrap_or_else(|| {
        eprintln!("Error: SVG file required");
        std::process::exit(1);
    });

    let pattern = Pattern::from_name(pattern_name).unwrap_or_else(|| {
        eprintln!("Unknown pattern: {}", pattern_name);
        std::process::exit(1);
    });

    println!("Loading: {}", svg_path);
    let start_load = Instant::now();

    let svg_content = fs::read_to_string(svg_path)
        .expect("Failed to read SVG file");

    let polygons = extract_polygons_from_svg(&svg_content)
        .expect("Failed to parse SVG");

    let load_time = start_load.elapsed();
    println!("Loaded {} polygons in {:?}", polygons.len(), load_time);

    // Parameters
    let spacing = 2.5;
    let angle = 45.0;

    println!("\nRunning '{}' pattern...", pattern.name());
    let start = Instant::now();

    let mut total_lines = 0;
    for polygon in &polygons {
        let lines = generate_pattern(pattern, polygon, spacing, angle);
        total_lines += lines.len();
    }

    let elapsed = start.elapsed();

    println!("\n═══════════════════════════════════════════════");
    println!("  RUST BENCHMARK: {}", pattern.name().to_uppercase());
    println!("═══════════════════════════════════════════════");
    println!("  Polygons: {}", polygons.len());
    println!("  Lines generated: {}", total_lines);
    println!("  Time: {:?}", elapsed);
    println!("  Time (ms): {:.2}", elapsed.as_secs_f64() * 1000.0);
    println!("  Avg per polygon: {:.3}ms", elapsed.as_secs_f64() * 1000.0 / polygons.len() as f64);
    println!("═══════════════════════════════════════════════");
}

/// Generate pattern lines for a polygon.
fn generate_pattern(pattern: Pattern, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
    match pattern {
        Pattern::Lines => generate_lines_fill(polygon, spacing, angle),
        Pattern::Crosshatch => generate_crosshatch_fill(polygon, spacing, angle),
        Pattern::Zigzag => generate_zigzag_fill(polygon, spacing, angle, spacing),
        Pattern::Wiggle => generate_wiggle_fill(polygon, spacing, angle, spacing, 0.1),
        Pattern::Spiral => generate_spiral_fill(polygon, spacing, angle),
        Pattern::Fermat => generate_fermat_fill(polygon, spacing, angle),
        Pattern::Concentric => generate_concentric_fill(polygon, spacing, true),
        Pattern::Radial => generate_radial_fill(polygon, 10.0, angle), // 10 degrees between rays
        Pattern::Honeycomb => generate_honeycomb_fill(polygon, spacing * 4.0, angle),
        Pattern::Crossspiral => generate_crossspiral_fill(polygon, spacing, angle),
        Pattern::Hilbert => generate_hilbert_fill(polygon, spacing, angle),
        Pattern::Guilloche => generate_guilloche_fill(polygon, spacing, angle),
        // Stub patterns - these output warnings and fall back to simpler patterns
        Pattern::Scribble => generate_scribble_fill(polygon, spacing, angle),
        Pattern::Gyroid => generate_gyroid_fill(polygon, spacing, angle),
    }
}

/// Convert lines to SVG output.
fn lines_to_svg(lines: &[Line], original_svg: &str) -> String {
    // Extract viewBox from original SVG
    let viewbox = extract_viewbox(original_svg).unwrap_or("0 0 1000 1000".to_string());

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">
<g stroke="black" stroke-width="0.5" fill="none">
"#,
        viewbox
    ));

    // Group lines into paths for efficiency
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
fn extract_viewbox(svg: &str) -> Option<String> {
    // Simple regex-free extraction
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
