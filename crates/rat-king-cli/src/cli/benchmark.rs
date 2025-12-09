//! Benchmark command implementation.

use std::fs;
use std::time::Instant;

use rat_king::{extract_polygons_from_svg, Pattern};

use super::common::generate_pattern;

/// Execute the benchmark command.
pub fn cmd_benchmark(args: &[String]) {
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
            "-h" | "--help" => {
                print_usage();
                return;
            }
            path if !path.starts_with('-') => {
                if svg_path.is_none() {
                    svg_path = Some(path);
                }
            }
            _ => {}
        }
        i += 1;
    }

    let svg_path = svg_path.unwrap_or_else(|| {
        eprintln!("Error: SVG file required");
        print_usage();
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

    println!();
    println!("═══════════════════════════════════════════════");
    println!("  RUST BENCHMARK: {}", pattern.name().to_uppercase());
    println!("═══════════════════════════════════════════════");
    println!("  Polygons: {}", polygons.len());
    println!("  Lines generated: {}", total_lines);
    println!("  Time: {:?}", elapsed);
    println!("  Time (ms): {:.2}", elapsed.as_secs_f64() * 1000.0);
    println!("  Avg per polygon: {:.3}ms", elapsed.as_secs_f64() * 1000.0 / polygons.len() as f64);
    println!("═══════════════════════════════════════════════");
}

fn print_usage() {
    eprintln!("Usage: rat-king benchmark <input.svg> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -p, --pattern <name>    Pattern to benchmark (default: lines)");
    eprintln!();
    eprintln!("Benchmarks pattern generation performance.");
}
