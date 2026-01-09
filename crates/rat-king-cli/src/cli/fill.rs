//! Fill command implementation.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use rat_king::{
    chain_lines, ChainConfig, ChainStats,
    extract_polygons_from_svg, Line, Pattern, Polygon,
    order_polygons, calculate_travel_distance, OrderingStrategy,
    SketchyConfig, sketchify_lines, polygon_to_lines,
};

use super::common::{OutputFormat, generate_pattern, lines_to_svg, chains_to_svg, grouped_chains_to_svg, StyledGroup};

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

// ============================================================================
// Fill Configuration (YAML)
// ============================================================================

/// Configuration for per-group pattern fills.
#[derive(Debug, Clone, Deserialize)]
pub struct FillConfig {
    /// Default settings applied to all groups
    #[serde(default)]
    pub defaults: GroupConfig,

    /// Per-group overrides keyed by SVG group ID
    #[serde(default)]
    pub groups: HashMap<String, GroupConfig>,
}

/// Configuration for a single group (or defaults).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GroupConfig {
    /// Pattern name (e.g., "lines", "crosshatch", "concentric")
    #[serde(default)]
    pub pattern: Option<String>,

    /// Line spacing
    #[serde(default)]
    pub spacing: Option<f64>,

    /// Pattern angle in degrees
    #[serde(default)]
    pub angle: Option<f64>,

    /// Stroke color (e.g., "#FF0000" or "red")
    #[serde(default)]
    pub color: Option<String>,
}

impl FillConfig {
    /// Load configuration from a YAML file.
    pub fn load(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse config YAML: {}", e))
    }

    /// Get the effective configuration for a group.
    /// Returns pattern, spacing, angle, color by merging group config with defaults.
    pub fn get_for_group(&self, group_id: Option<&str>) -> ResolvedConfig {
        let group_config = group_id.and_then(|id| self.groups.get(id));

        ResolvedConfig {
            pattern: group_config
                .and_then(|g| g.pattern.clone())
                .or_else(|| self.defaults.pattern.clone())
                .unwrap_or_else(|| "lines".to_string()),
            spacing: group_config
                .and_then(|g| g.spacing)
                .or(self.defaults.spacing)
                .unwrap_or(2.5),
            angle: group_config
                .and_then(|g| g.angle)
                .or(self.defaults.angle)
                .unwrap_or(45.0),
            color: group_config
                .and_then(|g| g.color.clone())
                .or_else(|| self.defaults.color.clone())
                .unwrap_or_else(|| "#000000".to_string()),
        }
    }
}

/// Resolved configuration with all values filled in.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub pattern: String,
    pub spacing: f64,
    pub angle: f64,
    pub color: String,
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
    let mut quiet = false;  // --quiet: suppress info messages
    let mut config_path: Option<&str> = None;  // --config: per-group pattern config
    let mut color_override: Option<&str> = None;  // --color: simple color override

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
            "-q" | "--quiet" => {
                quiet = true;
            }
            "--config" => {
                i += 1;
                if i < args.len() {
                    config_path = Some(&args[i]);
                }
            }
            "--color" => {
                i += 1;
                if i < args.len() {
                    color_override = Some(&args[i]);
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
        if !quiet { eprintln!("Reading SVG from stdin..."); }
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else {
        if !quiet { eprintln!("Loading: {}", svg_path); }
        fs::read_to_string(svg_path)
            .expect("Failed to read SVG file")
    };

    let polygons = extract_polygons_from_svg(&svg_content)
        .expect("Failed to parse SVG");

    let with_holes: Vec<_> = polygons.iter().filter(|p| !p.holes.is_empty()).collect();
    if !quiet {
        eprintln!("Loaded {} polygons ({} with holes, {} total holes)",
            polygons.len(),
            with_holes.len(),
            with_holes.iter().map(|p| p.holes.len()).sum::<usize>());
    }

    // Calculate travel optimization
    let order = order_polygons(&polygons, order_strategy);

    if polygons.len() > 1 && !quiet {
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
            if !quiet {
                eprintln!("Generated {} lines in {} shapes in {:?}", total_lines, shapes.len(), elapsed);
                if sketchy_config.is_some() {
                    eprintln!("Applied sketchy effect");
                }
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
            if !quiet {
                eprintln!("Generated {} lines -> {} chains ({:.0}% reduction) in {:?}",
                    all_lines.len(), chains.len(), stats.reduction_ratio * 100.0, elapsed);
                if sketchy_config.is_some() {
                    eprintln!("Applied sketchy effect");
                }
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
            // Load config if provided
            let fill_config = config_path.map(|path| {
                FillConfig::load(path).unwrap_or_else(|e| {
                    eprintln!("Error loading config: {}", e);
                    std::process::exit(1);
                })
            });

            // Check if any polygon has data_pattern attribute
            let has_data_patterns = polygons.iter().any(|p| p.data_pattern.is_some());

            if has_data_patterns || fill_config.is_some() {
                // Per-polygon or per-group mode
                // Group polygons by their group_id (for output structure)
                let mut groups_map: HashMap<String, Vec<usize>> = HashMap::new();

                for &idx in &order {
                    let polygon = &polygons[idx];
                    let group_id = polygon.group_id.clone().unwrap_or_else(|| "_default".to_string());
                    groups_map.entry(group_id).or_default().push(idx);
                }

                // Generate lines for each group
                let mut styled_groups: Vec<StyledGroup> = Vec::new();
                let mut total_lines = 0;

                for (group_id, polygon_indices) in &groups_map {
                    let mut group_lines: Vec<Line> = Vec::new();
                    let mut group_color = "#000000".to_string();

                    for &idx in polygon_indices {
                        let polygon = &polygons[idx];

                        // Priority: data_pattern > config group > command-line pattern
                        let (poly_pattern, poly_spacing, poly_angle, poly_color) = if let Some(ref pat_name) = polygon.data_pattern {
                            // Use pattern from data-pattern attribute
                            let pat = Pattern::from_name(pat_name).unwrap_or(pattern);
                            // Get spacing/angle/color from config or defaults
                            if let Some(ref config) = fill_config {
                                let resolved = config.get_for_group(polygon.group_id.as_deref());
                                (pat, resolved.spacing, resolved.angle, resolved.color)
                            } else {
                                (pat, spacing, angle, "#000000".to_string())
                            }
                        } else if let Some(ref config) = fill_config {
                            // Use config-based pattern
                            let resolved = config.get_for_group(polygon.group_id.as_deref());
                            let pat = Pattern::from_name(&resolved.pattern).unwrap_or(pattern);
                            (pat, resolved.spacing, resolved.angle, resolved.color)
                        } else {
                            // Use command-line pattern
                            (pattern, spacing, angle, "#000000".to_string())
                        };

                        group_color = poly_color;

                        let lines = generate_pattern(poly_pattern, polygon, poly_spacing, poly_angle);
                        let lines = post_process(lines, polygon);
                        group_lines.extend(lines);
                    }

                    let group_lines = apply_sketchy(group_lines);
                    total_lines += group_lines.len();

                    // Chain lines within this group
                    let chain_config_inner = ChainConfig::with_tolerance(chain_tolerance);
                    let chains = chain_lines(&group_lines, &chain_config_inner);

                    styled_groups.push(StyledGroup {
                        group_id: group_id.clone(),
                        chains,
                        color: color_override.map(|s| s.to_string()).unwrap_or(group_color),
                    });
                }

                let elapsed = start.elapsed();
                if !quiet {
                    eprintln!("Generated {} lines in {} groups in {:?}", total_lines, styled_groups.len(), elapsed);
                    if has_data_patterns {
                        eprintln!("Using per-polygon data-pattern attributes");
                    }
                    if sketchy_config.is_some() {
                        eprintln!("Applied sketchy effect");
                    }
                }

                grouped_chains_to_svg(&styled_groups, &svg_content)
            } else {
                // Simple mode: single pattern for all polygons
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
                    if !quiet {
                        eprintln!("Generated {} lines in {:?}", all_lines.len(), elapsed);
                        if sketchy_config.is_some() {
                            eprintln!("Applied sketchy effect");
                        }
                    }
                    lines_to_svg(&all_lines, &svg_content)
                } else {
                    // Default: chain lines into polylines for smaller output
                    let chain_config_inner = ChainConfig::with_tolerance(chain_tolerance);
                    let chains = chain_lines(&all_lines, &chain_config_inner);
                    let stats = ChainStats::from_chains(all_lines.len(), &chains);

                    if !quiet {
                        eprintln!("Generated {} lines -> {} chains ({:.0}% reduction) in {:?}",
                            all_lines.len(), chains.len(), stats.reduction_ratio * 100.0, elapsed);
                        if sketchy_config.is_some() {
                            eprintln!("Applied sketchy effect");
                        }
                    }

                    chains_to_svg(&chains, &svg_content)
                }
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
            if !quiet { eprintln!("Wrote: {}", path); }
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
    eprintln!("  --config <file>         Per-group pattern config (YAML)");
    eprintln!("  --color <hex>           Override stroke color for all patterns");
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
    eprintln!("  -q, --quiet             Suppress info messages (for piping)");
    eprintln!();
    eprintln!("Config file format (YAML):");
    eprintln!("  defaults:");
    eprintln!("    pattern: lines");
    eprintln!("    spacing: 2.0");
    eprintln!("    angle: 45");
    eprintln!("    color: \"#666666\"");
    eprintln!("  groups:");
    eprintln!("    towns_vt:");
    eprintln!("      pattern: concentric");
    eprintln!("      color: \"#2E7D32\"");
    eprintln!("    water:");
    eprintln!("      pattern: wiggle");
    eprintln!("      color: \"#0288D1\"");
    eprintln!();
    eprintln!("Use '-' as input to read from stdin");
}
