//! Fill command implementation.

use std::fs;
use std::io::{self, Read};
use std::time::Instant;

use serde::Serialize;

use rat_king::{
    chain_lines, ChainConfig, ChainStats,
    extract_polygons_from_svg, Line, Pattern, Polygon,
    order_polygons, calculate_travel_distance, OrderingStrategy,
    SketchyConfig, sketchify_lines, polygon_to_lines,
};

use super::common::{OutputFormat, generate_pattern, lines_to_svg, chains_to_svg};

/// A line in JSON output format.
#[derive(Serialize)]
struct JsonLine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

/// A point in JSON output format.
#[derive(Serialize)]
struct JsonPoint {
    x: f64,
    y: f64,
}

/// A chain (polyline) in JSON output format.
type JsonChain = Vec<JsonPoint>;

/// Chaining statistics for JSON output.
#[derive(Serialize)]
struct JsonChainStats {
    input_lines: usize,
    output_chains: usize,
    reduction_percent: f64,
    avg_chain_length: f64,
}

/// A shape with its lines in JSON output (per-polygon mode).
#[derive(Serialize)]
struct JsonShape {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    index: usize,
    lines: Vec<JsonLine>,
}

/// JSON output with all lines (flat mode) - includes both lines and chains.
#[derive(Serialize)]
struct JsonOutputFlat {
    lines: Vec<JsonLine>,
    chains: Vec<JsonChain>,
    chain_stats: JsonChainStats,
}

/// JSON output with per-shape grouping.
#[derive(Serialize)]
struct JsonOutputGrouped {
    shapes: Vec<JsonShape>,
}

/// Execute the fill command.
pub fn cmd_fill(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut pattern_name = "lines";
    let mut spacing = 2.5;
    let mut angle = 45.0;
    let mut format = OutputFormat::Svg;
    let mut grouped = false;
    let mut order_strategy = OrderingStrategy::NearestNeighbor;
    let mut include_strokes = false;
    let mut sketchy_config: Option<SketchyConfig> = None;
    let mut raw_output = false;  // --raw: output individual lines instead of chained polylines
    let mut chain_tolerance = 0.1;  // --chain-tolerance: max distance to consider endpoints connected

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
            "-f" | "--format" => {
                i += 1;
                if i < args.len() {
                    format = match args[i].to_lowercase().as_str() {
                        "json" => OutputFormat::Json,
                        "svg" => OutputFormat::Svg,
                        other => {
                            eprintln!("Unknown format: {}. Use 'svg' or 'json'.", other);
                            std::process::exit(1);
                        }
                    };
                }
            }
            "--json" => {
                format = OutputFormat::Json;
            }
            "--order" => {
                i += 1;
                if i < args.len() {
                    order_strategy = OrderingStrategy::from_name(&args[i]).unwrap_or_else(|| {
                        eprintln!("Unknown order strategy: {}. Use 'document' or 'nearest'.", args[i]);
                        std::process::exit(1);
                    });
                }
            }
            "--no-optimize" => {
                order_strategy = OrderingStrategy::Document;
            }
            "--grouped" => {
                grouped = true;
            }
            "--strokes" => {
                include_strokes = true;
            }
            "--sketchy" => {
                sketchy_config = Some(SketchyConfig::default());
            }
            "--roughness" => {
                i += 1;
                if i < args.len() {
                    let roughness: f64 = args[i].parse().unwrap_or(1.0);
                    if let Some(ref mut config) = sketchy_config {
                        config.roughness = roughness;
                    } else {
                        let mut config = SketchyConfig::default();
                        config.roughness = roughness;
                        sketchy_config = Some(config);
                    }
                }
            }
            "--bowing" => {
                i += 1;
                if i < args.len() {
                    let bowing: f64 = args[i].parse().unwrap_or(1.0);
                    if let Some(ref mut config) = sketchy_config {
                        config.bowing = bowing;
                    } else {
                        let mut config = SketchyConfig::default();
                        config.bowing = bowing;
                        sketchy_config = Some(config);
                    }
                }
            }
            "--no-double-stroke" => {
                if let Some(ref mut config) = sketchy_config {
                    config.double_stroke = false;
                } else {
                    let mut config = SketchyConfig::default();
                    config.double_stroke = false;
                    sketchy_config = Some(config);
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    let seed: u64 = args[i].parse().unwrap_or(42);
                    if let Some(ref mut config) = sketchy_config {
                        config.seed = Some(seed);
                    } else {
                        let mut config = SketchyConfig::default();
                        config.seed = Some(seed);
                        sketchy_config = Some(config);
                    }
                }
            }
            "--raw" => {
                raw_output = true;
            }
            "--chain-tolerance" => {
                i += 1;
                if i < args.len() {
                    chain_tolerance = args[i].parse().unwrap_or(0.1);
                }
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "-" => {
                if svg_path.is_none() {
                    svg_path = Some("-");
                }
            }
            path if !path.starts_with('-') => {
                if svg_path.is_none() {
                    svg_path = Some(path);
                }
            }
            unknown => {
                eprintln!("Unknown option: {}", unknown);
            }
        }
        i += 1;
    }

    let svg_path = svg_path.unwrap_or_else(|| {
        eprintln!("Error: SVG file required (use '-' for stdin)");
        print_usage();
        std::process::exit(1);
    });

    let pattern = Pattern::from_name(pattern_name).unwrap_or_else(|| {
        eprintln!("Unknown pattern: {}. Use 'rat-king patterns' to list available.", pattern_name);
        std::process::exit(1);
    });

    // Read SVG content
    let svg_content = if svg_path == "-" {
        eprintln!("Reading SVG from stdin...");
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else {
        eprintln!("Loading: {}", svg_path);
        fs::read_to_string(svg_path)
            .expect("Failed to read SVG file")
    };

    let polygons = extract_polygons_from_svg(&svg_content)
        .expect("Failed to parse SVG");

    let with_holes: Vec<_> = polygons.iter().filter(|p| !p.holes.is_empty()).collect();
    eprintln!("Loaded {} polygons ({} with holes, {} total holes)",
        polygons.len(),
        with_holes.len(),
        with_holes.iter().map(|p| p.holes.len()).sum::<usize>());

    // Calculate travel optimization
    let order = order_polygons(&polygons, order_strategy);

    if polygons.len() > 1 {
        let doc_order: Vec<usize> = (0..polygons.len()).collect();
        let doc_travel = calculate_travel_distance(&polygons, &doc_order);
        let opt_travel = calculate_travel_distance(&polygons, &order);

        if order_strategy == OrderingStrategy::NearestNeighbor {
            let savings = ((doc_travel - opt_travel) / doc_travel * 100.0).max(0.0);
            eprintln!("Travel optimization: {:.1} -> {:.1} ({:.0}% reduction)",
                doc_travel, opt_travel, savings);
        } else {
            eprintln!("Using document order (travel: {:.1})", doc_travel);
        }
    }

    let start = Instant::now();

    // Helper closures
    let post_process = |mut lines: Vec<Line>, polygon: &Polygon| -> Vec<Line> {
        if include_strokes {
            lines.extend(polygon_to_lines(polygon));
        }
        lines
    };

    let apply_sketchy = |lines: Vec<Line>| -> Vec<Line> {
        if let Some(ref config) = sketchy_config {
            sketchify_lines(&lines, config)
        } else {
            lines
        }
    };

    // Generate output
    let output = match (format, grouped) {
        (OutputFormat::Json, true) => {
            let shapes: Vec<JsonShape> = order
                .iter()
                .map(|&idx| {
                    let polygon = &polygons[idx];
                    let lines = generate_pattern(pattern, polygon, spacing, angle);
                    let lines = post_process(lines, polygon);
                    let lines = apply_sketchy(lines);
                    JsonShape {
                        id: polygon.id.clone(),
                        index: idx,
                        lines: lines.iter().map(|l| JsonLine {
                            x1: l.x1, y1: l.y1, x2: l.x2, y2: l.y2,
                        }).collect(),
                    }
                })
                .collect();

            let elapsed = start.elapsed();
            let total_lines: usize = shapes.iter().map(|s| s.lines.len()).sum();
            eprintln!("Generated {} lines in {} shapes in {:?}", total_lines, shapes.len(), elapsed);
            if sketchy_config.is_some() {
                eprintln!("Applied sketchy effect");
            }

            serde_json::to_string(&JsonOutputGrouped { shapes }).expect("Failed to serialize JSON")
        }
        (OutputFormat::Json, false) => {
            let mut all_lines: Vec<Line> = Vec::new();
            for &idx in &order {
                let polygon = &polygons[idx];
                let lines = generate_pattern(pattern, polygon, spacing, angle);
                let lines = post_process(lines, polygon);
                all_lines.extend(lines);
            }
            let all_lines = apply_sketchy(all_lines);

            // Chain lines for JSON output (includes both raw lines and chains)
            let chain_config = ChainConfig::with_tolerance(chain_tolerance);
            let chains = chain_lines(&all_lines, &chain_config);
            let stats = ChainStats::from_chains(all_lines.len(), &chains);

            let elapsed = start.elapsed();
            eprintln!("Generated {} lines -> {} chains ({:.0}% reduction) in {:?}",
                all_lines.len(), chains.len(), stats.reduction_ratio * 100.0, elapsed);
            if sketchy_config.is_some() {
                eprintln!("Applied sketchy effect");
            }

            let json_lines: Vec<JsonLine> = all_lines.iter().map(|l| JsonLine {
                x1: l.x1, y1: l.y1, x2: l.x2, y2: l.y2,
            }).collect();

            let json_chains: Vec<JsonChain> = chains.iter().map(|chain| {
                chain.iter().map(|p| JsonPoint { x: p.x, y: p.y }).collect()
            }).collect();

            let json_stats = JsonChainStats {
                input_lines: stats.input_lines,
                output_chains: stats.output_chains,
                reduction_percent: stats.reduction_ratio * 100.0,
                avg_chain_length: stats.avg_chain_length,
            };

            serde_json::to_string(&JsonOutputFlat {
                lines: json_lines,
                chains: json_chains,
                chain_stats: json_stats,
            }).expect("Failed to serialize JSON")
        }
        (OutputFormat::Svg, _) => {
            let mut all_lines: Vec<Line> = Vec::new();
            for &idx in &order {
                let polygon = &polygons[idx];
                let lines = generate_pattern(pattern, polygon, spacing, angle);
                let lines = post_process(lines, polygon);
                all_lines.extend(lines);
            }
            let all_lines = apply_sketchy(all_lines);

            let elapsed = start.elapsed();

            if raw_output {
                // --raw: output individual <line> elements
                eprintln!("Generated {} lines in {:?}", all_lines.len(), elapsed);
                if sketchy_config.is_some() {
                    eprintln!("Applied sketchy effect");
                }
                lines_to_svg(&all_lines, &svg_content)
            } else {
                // Default: chain lines into polylines for smaller output
                let chain_config = ChainConfig::with_tolerance(chain_tolerance);
                let chains = chain_lines(&all_lines, &chain_config);
                let stats = ChainStats::from_chains(all_lines.len(), &chains);

                eprintln!("Generated {} lines -> {} chains ({:.0}% reduction) in {:?}",
                    all_lines.len(), chains.len(), stats.reduction_ratio * 100.0, elapsed);
                if sketchy_config.is_some() {
                    eprintln!("Applied sketchy effect");
                }

                chains_to_svg(&chains, &svg_content)
            }
        }
    };

    // Write output
    match output_path {
        Some("-") | None => {
            println!("{}", output);
        }
        Some(path) => {
            fs::write(path, &output).expect("Failed to write output file");
            eprintln!("Wrote: {}", path);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: rat-king fill <input.svg> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -o, --output <file>     Output file (default: stdout)");
    eprintln!("  -p, --pattern <name>    Pattern name (default: lines)");
    eprintln!("  -s, --spacing <n>       Line spacing (default: 2.5)");
    eprintln!("  -a, --angle <deg>       Pattern angle (default: 45)");
    eprintln!("  --json                  Output as JSON instead of SVG");
    eprintln!("  --grouped               Group lines by polygon (JSON only)");
    eprintln!("  --no-optimize           Disable travel path optimization");
    eprintln!("  --strokes               Include polygon outlines");
    eprintln!("  --raw                   Output individual <line> elements (default: chained <polyline>)");
    eprintln!("  --chain-tolerance <n>   Max distance to chain endpoints (default: 0.1)");
    eprintln!("  --sketchy               Enable hand-drawn effect");
    eprintln!("  --roughness <n>         Sketchy roughness (default: 1.0)");
    eprintln!("  --bowing <n>            Sketchy bowing (default: 1.0)");
    eprintln!("  --no-double-stroke      Disable double-stroke in sketchy mode");
    eprintln!("  --seed <n>              Random seed for sketchy effect");
    eprintln!();
    eprintln!("Use '-' as input to read from stdin");
}
