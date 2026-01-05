//! Pattern recipe system for declarative layer composition.
//!
//! Recipes are YAML files that define layered pattern compositions.
//! Each layer specifies a pattern type, parameters, and styling.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use rat_king::{Pattern, Point, Polygon, Line};

/// A complete recipe defining a layered pattern composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Recipe name/title
    pub name: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,

    /// Canvas configuration
    pub canvas: Canvas,

    /// Default style applied to all layers (can be overridden)
    #[serde(default)]
    pub defaults: LayerStyle,

    /// Ordered list of pattern layers (rendered bottom to top)
    pub layers: Vec<Layer>,
}

/// Canvas/output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    /// Width in millimeters
    pub width: f64,

    /// Height in millimeters
    pub height: f64,

    /// Background color (default: white)
    #[serde(default = "default_background")]
    pub background: String,
}

fn default_background() -> String {
    "white".to_string()
}

/// A single pattern layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Layer name (for identification)
    pub name: String,

    /// Pattern type (e.g., "lines", "crosshatch", "spiral")
    pub pattern: String,

    /// Pattern spacing parameter
    #[serde(default = "default_spacing")]
    pub spacing: f64,

    /// Pattern angle parameter (degrees)
    #[serde(default)]
    pub angle: f64,

    /// Layer style (merged with defaults)
    #[serde(default)]
    pub style: LayerStyle,

    /// Whether this layer is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_spacing() -> f64 {
    5.0
}

fn default_enabled() -> bool {
    true
}

/// Style properties for a layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerStyle {
    /// Stroke color
    #[serde(default)]
    pub color: Option<String>,

    /// Stroke width
    #[serde(default)]
    pub stroke_width: Option<f64>,

    /// Opacity (0.0 to 1.0)
    #[serde(default)]
    pub opacity: Option<f64>,
}

impl LayerStyle {
    /// Merge this style with defaults, preferring self's values.
    pub fn merge_with(&self, defaults: &LayerStyle) -> LayerStyle {
        LayerStyle {
            color: self.color.clone().or_else(|| defaults.color.clone()),
            stroke_width: self.stroke_width.or(defaults.stroke_width),
            opacity: self.opacity.or(defaults.opacity),
        }
    }

    /// Get color with fallback.
    pub fn color_or(&self, fallback: &str) -> String {
        self.color.clone().unwrap_or_else(|| fallback.to_string())
    }

    /// Get stroke width with fallback.
    pub fn stroke_width_or(&self, fallback: f64) -> f64 {
        self.stroke_width.unwrap_or(fallback)
    }

    /// Get opacity with fallback.
    pub fn opacity_or(&self, fallback: f64) -> f64 {
        self.opacity.unwrap_or(fallback)
    }
}

/// Result of rendering a recipe.
pub struct RenderedRecipe {
    /// All layers with their generated lines
    pub layers: Vec<RenderedLayer>,
    /// Canvas configuration
    pub canvas: Canvas,
    /// Recipe name
    pub name: String,
}

/// A rendered layer with lines and style.
pub struct RenderedLayer {
    pub name: String,
    pub lines: Vec<Line>,
    pub style: LayerStyle,
}

impl Recipe {
    /// Load a recipe from a YAML file.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read recipe file: {}", e))?;

        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse recipe YAML: {}", e))
    }

    /// Render the recipe to lines.
    pub fn render(&self) -> RenderedRecipe {
        // Convert mm to points (72 points per inch, 25.4 mm per inch)
        let scale = 72.0 / 25.4;
        let width_pts = self.canvas.width * scale;
        let height_pts = self.canvas.height * scale;

        // Create canvas polygon
        let canvas_poly = Polygon::new(vec![
            Point::new(0.0, 0.0),
            Point::new(width_pts, 0.0),
            Point::new(width_pts, height_pts),
            Point::new(0.0, height_pts),
        ]);

        let mut rendered_layers = Vec::new();

        for layer in &self.layers {
            if !layer.enabled {
                continue;
            }

            // Parse pattern type
            let pattern = match Pattern::from_name(&layer.pattern) {
                Some(p) => p,
                None => {
                    eprintln!("Warning: Unknown pattern '{}', skipping layer '{}'",
                        layer.pattern, layer.name);
                    continue;
                }
            };

            // Generate pattern lines
            let lines = pattern.generate(&canvas_poly, layer.spacing * scale, layer.angle);

            // Merge style with defaults
            let style = layer.style.merge_with(&self.defaults);

            rendered_layers.push(RenderedLayer {
                name: layer.name.clone(),
                lines,
                style,
            });
        }

        RenderedRecipe {
            layers: rendered_layers,
            canvas: self.canvas.clone(),
            name: self.name.clone(),
        }
    }
}

impl RenderedRecipe {
    /// Export to SVG string.
    pub fn to_svg(&self) -> String {
        let scale = 72.0 / 25.4;
        let width = self.canvas.width * scale;
        let height = self.canvas.height * scale;

        let mut svg = format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     width="{:.2}mm" height="{:.2}mm"
     viewBox="0 0 {:.2} {:.2}">
  <title>{}</title>
  <rect width="100%" height="100%" fill="{}"/>
"##,
            self.canvas.width, self.canvas.height,
            width, height,
            self.name,
            self.canvas.background
        );

        for layer in &self.layers {
            let color = layer.style.color_or("black");
            let stroke_width = layer.style.stroke_width_or(1.0);
            let opacity = layer.style.opacity_or(1.0);

            svg.push_str(&format!(
                r##"  <g id="{}" stroke="{}" stroke-width="{}" fill="none" opacity="{}" stroke-linecap="round">
"##,
                layer.name, color, stroke_width, opacity
            ));

            for line in &layer.lines {
                svg.push_str(&format!(
                    "    <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
                    line.x1, line.y1, line.x2, line.y2
                ));
            }

            svg.push_str("  </g>\n");
        }

        svg.push_str("</svg>\n");
        svg
    }
}

/// Execute the recipe command.
pub fn cmd_recipe(args: &[String]) {
    if args.is_empty() {
        print_usage();
        return;
    }

    let mut recipe_path: Option<String> = None;
    let mut output_path = "output.svg".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = args[i].clone();
                }
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            "--example" => {
                print_example();
                return;
            }
            arg if !arg.starts_with('-') => {
                recipe_path = Some(arg.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    let recipe_path = match recipe_path {
        Some(p) => p,
        None => {
            eprintln!("Error: No recipe file specified");
            print_usage();
            return;
        }
    };

    eprintln!("Loading recipe: {}", recipe_path);

    let recipe = match Recipe::load(&recipe_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    eprintln!("Recipe: {}", recipe.name);
    eprintln!("Canvas: {}mm × {}mm", recipe.canvas.width, recipe.canvas.height);
    eprintln!("Layers: {}", recipe.layers.len());

    let rendered = recipe.render();

    let total_lines: usize = rendered.layers.iter().map(|l| l.lines.len()).sum();
    eprintln!("Generated {} lines across {} layers", total_lines, rendered.layers.len());

    let svg = rendered.to_svg();
    fs::write(&output_path, &svg).expect("Failed to write SVG");
    eprintln!("Wrote: {}", output_path);
}

fn print_usage() {
    eprintln!("rat-king recipe - Render layered pattern compositions from YAML");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    rat-king recipe <recipe.yaml> [OPTIONS]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -o, --output <file>    Output SVG file (default: output.svg)");
    eprintln!("    --example              Print an example recipe YAML");
    eprintln!("    -h, --help             Show this help");
    eprintln!();
    eprintln!("EXAMPLE:");
    eprintln!("    rat-king recipe my_design.yaml -o my_design.svg");
}

fn print_example() {
    println!(r##"# Example rat-king recipe
name: "Layered Pattern Demo"
description: "A simple example showing layered patterns"

canvas:
  width: 100    # millimeters
  height: 100
  background: "white"

defaults:
  color: "#333333"
  stroke_width: 0.5
  opacity: 1.0

layers:
  - name: base_lines
    pattern: lines
    spacing: 4
    angle: 0
    style:
      color: "#666666"
      opacity: 0.5

  - name: crosshatch_overlay
    pattern: crosshatch
    spacing: 6
    angle: 45
    style:
      color: "#333333"
      stroke_width: 0.75

  - name: spiral_accent
    pattern: spiral
    spacing: 3
    angle: 0
    style:
      color: "#000000"
      stroke_width: 1.0
"##);
}
