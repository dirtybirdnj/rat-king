//! rat-king - TUI and CLI for pattern generation
//!
//! Usage:
//!   rat-king [svg_file]              Launch TUI (default: test_assets/essex.svg)
//!   rat-king fill <svg> -p <pattern> Generate pattern fill
//!   rat-king benchmark <svg>         Benchmark pattern generation
//!   rat-king patterns                List available patterns

mod cli;

use std::env;
use std::fs;
use std::io::{self, stdout};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};


use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use image::{DynamicImage, RgbaImage};
use resvg::usvg;
use tiny_skia::Pixmap;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use ratatui_image::{picker::{Picker, ProtocolType}, protocol::StatefulProtocol, StatefulImage};

use rat_king::{extract_polygons_from_svg, Line, Pattern, Polygon};

// Import CLI commands
use cli::{
    cmd_fill, cmd_benchmark, generate_pattern,
    AnalysisResult, HarnessResult, VisualHarnessReport,
    analyze_pattern_vs_solid, generate_diff_image,
};

// Default stroke width (can be adjusted via UI)
const DEFAULT_STROKE_WIDTH: f64 = 1.5;
// Sixel typically renders at 10 pixels per terminal cell (varies by terminal)
const PIXELS_PER_CELL_X: u32 = 10;
const PIXELS_PER_CELL_Y: u32 = 20;

/// Style settings for SVG rendering
#[derive(Clone)]
struct RenderStyle {
    stroke_width: f64,
    stroke_color: String,
    show_strokes: bool,
}

impl Default for RenderStyle {
    fn default() -> Self {
        Self {
            stroke_width: DEFAULT_STROKE_WIDTH,
            stroke_color: "#000000".to_string(),
            show_strokes: true,
        }
    }
}

/// Build an SVG string from lines and polygons (without view transform).
/// The SVG is built at native coordinates for caching.
fn build_svg_content(
    lines: &[Line],
    polygons: &[Polygon],
    bounds: (f64, f64, f64, f64),
    style: &RenderStyle,
) -> String {
    let (min_x, min_y, max_x, max_y) = bounds;
    let width = max_x - min_x;
    let height = max_y - min_y;

    // Add padding around content
    let padding = width.max(height) * 0.02;
    let vb_x = min_x - padding;
    let vb_y = min_y - padding;
    let vb_w = width + padding * 2.0;
    let vb_h = height + padding * 2.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.2} {:.2} {:.2} {:.2}">
"#,
        vb_x, vb_y, vb_w, vb_h
    ));

    // Draw polygon outlines (gray) if strokes enabled
    if style.show_strokes {
        svg.push_str(&format!(
            "<g stroke=\"#cccccc\" stroke-width=\"{}\" fill=\"none\">",
            style.stroke_width
        ));
        for poly in polygons {
            let points = &poly.outer;
            if points.len() >= 2 {
                svg.push_str("<path d=\"M");
                for (i, pt) in points.iter().enumerate() {
                    if i == 0 {
                        svg.push_str(&format!("{:.2},{:.2}", pt.x, pt.y));
                    } else {
                        svg.push_str(&format!(" L{:.2},{:.2}", pt.x, pt.y));
                    }
                }
                svg.push_str(" Z\"/>\n");
            }
        }
        svg.push_str("</g>\n");
    }

    // Draw pattern lines
    svg.push_str(&format!(
        "<g stroke=\"{}\" stroke-width=\"{}\" stroke-linecap=\"round\" fill=\"none\">",
        style.stroke_color, style.stroke_width
    ));
    for line in lines {
        svg.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
            line.x1, line.y1, line.x2, line.y2
        ));
    }
    svg.push_str("</g>\n</svg>");

    svg
}

/// Build an SVG with solid filled polygons (no pattern lines).
/// Used as a reference for visual comparison - the pattern should fill exactly this area.
/// When `for_analysis` is true, omits strokes for cleaner pixel comparison.
fn build_solid_fill_svg(
    polygons: &[Polygon],
    bounds: (f64, f64, f64, f64),
) -> String {
    build_solid_fill_svg_internal(polygons, bounds, false)
}

fn build_solid_fill_svg_internal(
    polygons: &[Polygon],
    bounds: (f64, f64, f64, f64),
    for_analysis: bool,
) -> String {
    let (min_x, min_y, max_x, max_y) = bounds;
    let width = max_x - min_x;
    let height = max_y - min_y;

    let padding = width.max(height) * 0.02;
    let vb_x = min_x - padding;
    let vb_y = min_y - padding;
    let vb_w = width + padding * 2.0;
    let vb_h = height + padding * 2.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.2} {:.2} {:.2} {:.2}">
"#,
        vb_x, vb_y, vb_w, vb_h
    ));

    // For analysis, use pure black fill with no stroke for cleaner comparison
    // For visual display, use gray fill with stroke outline
    let (fill, stroke) = if for_analysis {
        ("#000000", "none")
    } else {
        ("#cccccc", "#888888")
    };
    let stroke_width = if for_analysis { "0" } else { "0.5" };

    svg.push_str(&format!("<g fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\">", fill, stroke, stroke_width));
    for poly in polygons {
        let points = &poly.outer;
        if points.len() >= 2 {
            svg.push_str("<path d=\"M");
            for (i, pt) in points.iter().enumerate() {
                if i == 0 {
                    svg.push_str(&format!("{:.2},{:.2}", pt.x, pt.y));
                } else {
                    svg.push_str(&format!(" L{:.2},{:.2}", pt.x, pt.y));
                }
            }
            svg.push_str(" Z\"/>\n");
        }
    }
    svg.push_str("</g>\n</svg>");

    svg
}

/// Build an SVG for pattern analysis (pattern lines only, no polygon outlines).
fn build_pattern_svg_for_analysis(
    lines: &[Line],
    bounds: (f64, f64, f64, f64),
    style: &RenderStyle,
) -> String {
    let (min_x, min_y, max_x, max_y) = bounds;
    let width = max_x - min_x;
    let height = max_y - min_y;

    let padding = width.max(height) * 0.02;
    let vb_x = min_x - padding;
    let vb_y = min_y - padding;
    let vb_w = width + padding * 2.0;
    let vb_h = height + padding * 2.0;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{:.2} {:.2} {:.2} {:.2}">
"#,
        vb_x, vb_y, vb_w, vb_h
    ));

    // Draw only pattern lines (no polygon outlines) for analysis
    svg.push_str(&format!(
        "<g stroke=\"#000000\" stroke-width=\"{}\" stroke-linecap=\"round\" fill=\"none\">",
        style.stroke_width
    ));
    for line in lines {
        svg.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
            line.x1, line.y1, line.x2, line.y2
        ));
    }
    svg.push_str("</g>\n</svg>");

    svg
}

/// Parse SVG content into a usvg Tree for caching.
fn parse_svg_tree(svg_content: &str) -> usvg::Tree {
    let mut options = usvg::Options::default();
    // Increase node limit for complex patterns
    options.resources_dir = None;
    usvg::Tree::from_str(svg_content, &options)
        .expect("Failed to parse generated SVG")
}

/// Try to parse SVG content, returning None if it fails (e.g., too many nodes).
fn try_parse_svg_tree(svg_content: &str) -> Option<usvg::Tree> {
    let mut options = usvg::Options::default();
    options.resources_dir = None;
    usvg::Tree::from_str(svg_content, &options).ok()
}

/// Render a cached usvg Tree with zoom and pan applied via transform.
/// Dimensions are specified to allow dynamic sizing based on terminal area.
fn render_tree_to_image(
    tree: &usvg::Tree,
    bounds: (f64, f64, f64, f64),
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    width: u32,
    height: u32,
) -> DynamicImage {
    let (min_x, min_y, max_x, max_y) = bounds;
    let svg_width = max_x - min_x;
    let svg_height = max_y - min_y;

    // Calculate base scale to fit image dimensions with padding
    let padding = 20.0;
    let scale_x = (width as f64 - padding * 2.0) / svg_width;
    let scale_y = (height as f64 - padding * 2.0) / svg_height;
    let base_scale = scale_x.min(scale_y);

    // Apply zoom
    let scale = base_scale * zoom;

    // Calculate center offset, then apply pan
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let content_center_x = (min_x + max_x) / 2.0;
    let content_center_y = (min_y + max_y) / 2.0;

    // Pan is in SVG units, scaled by zoom
    let translate_x = center_x - content_center_x * scale - pan_x * base_scale;
    let translate_y = center_y - content_center_y * scale - pan_y * base_scale;

    // Build transform: first scale, then translate
    let transform = tiny_skia::Transform::from_scale(scale as f32, scale as f32)
        .post_translate(translate_x as f32, translate_y as f32);

    // Create pixmap with white background
    let mut pixmap = Pixmap::new(width, height)
        .expect("Failed to create pixmap");
    pixmap.fill(tiny_skia::Color::WHITE);

    // Render with transform
    resvg::render(tree, transform, &mut pixmap.as_mut());

    // Convert to image::DynamicImage
    let rgba = RgbaImage::from_raw(width, height, pixmap.take())
        .expect("Failed to create image");

    DynamicImage::ImageRgba8(rgba)
}

/// Result from background pattern generation
struct PatternResult {
    lines: Vec<Line>,
    gen_time_ms: f64,
    /// Cached SVG tree for fast re-rendering on zoom/pan
    svg_tree: usvg::Tree,
}

/// Available stroke colors
const STROKE_COLORS: &[(&str, &str)] = &[
    ("Black", "#000000"),
    ("Red", "#ff0000"),
    ("Green", "#00aa00"),
    ("Blue", "#0000ff"),
    ("Orange", "#ff8800"),
    ("Purple", "#8800ff"),
    ("Cyan", "#00aaaa"),
    ("Magenta", "#ff00ff"),
];

/// Application state for TUI
struct App {
    /// Loaded polygons from SVG
    polygons: Vec<Polygon>,
    /// Current pattern selection
    pattern_state: ListState,
    /// All available patterns
    patterns: Vec<Pattern>,
    /// Current spacing value
    spacing: f64,
    /// Current angle value
    angle: f64,
    /// Generated lines (cached)
    lines: Vec<Line>,
    /// Cached SVG tree for fast zoom/pan rendering
    cached_tree: Option<usvg::Tree>,
    /// Bounding box of all polygons
    bounds: (f64, f64, f64, f64),
    /// Last generation time
    gen_time_ms: f64,
    /// Should exit
    should_quit: bool,
    /// Which sidebar setting is focused
    setting_focus: usize,
    /// SVG file path
    svg_path: String,
    /// Is pattern generation in progress?
    is_loading: bool,
    /// Flag to regenerate after current generation completes
    needs_regenerate: bool,
    /// Channel to receive pattern results
    result_rx: Receiver<PatternResult>,
    /// Channel to send pattern generation requests
    result_tx: Sender<PatternResult>,
    /// Animation frame counter for spinner
    spinner_frame: usize,
    /// Image picker for terminal protocol detection
    picker: Picker,
    /// Current rendered image protocol state
    image_state: Option<Box<dyn StatefulProtocol>>,
    /// Flag to indicate image needs re-rendering
    needs_image_update: bool,
    /// Zoom level (1.0 = fit to view, higher = zoomed in)
    zoom: f64,
    /// Pan offset as fraction of view (0.0, 0.0 = centered)
    pan_x: f64,
    pan_y: f64,
    /// Last time a view change (zoom/pan) occurred - for debouncing
    last_view_change: Option<Instant>,
    /// Pattern list area for mouse hit testing
    pattern_list_area: Option<Rect>,
    /// Image view area for drag panning
    image_area: Option<Rect>,
    /// Mouse drag state: Some((start_x, start_y)) when dragging
    drag_start: Option<(u16, u16)>,
    /// Pan offset at drag start
    drag_start_pan: (f64, f64),
    /// Render style settings
    render_style: RenderStyle,
    /// Current color index in STROKE_COLORS
    color_index: usize,
    /// Time taken for last render (ms)
    render_time_ms: f64,
    /// Flag indicating style changed and SVG needs rebuild
    style_changed: bool,
    /// Current render dimensions (updated based on terminal size)
    render_width: u32,
    render_height: u32,
    /// Show solid fill reference (toggle with 'f')
    show_solid_fill: bool,
    /// Cached solid fill SVG tree
    solid_fill_tree: Option<usvg::Tree>,
    /// Status message (shown briefly after actions like screenshot save)
    status_message: Option<(String, Instant)>,
}

impl App {
    fn new(svg_path: &str) -> Result<Self, String> {
        let svg_content = fs::read_to_string(svg_path)
            .map_err(|e| format!("Failed to read {}: {}", svg_path, e))?;

        let polygons = extract_polygons_from_svg(&svg_content)
            .map_err(|e| format!("Failed to parse SVG: {}", e))?;

        if polygons.is_empty() {
            return Err("No polygons found in SVG".to_string());
        }

        // Calculate bounding box
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for poly in &polygons {
            if let Some((x1, y1, x2, y2)) = poly.bounding_box() {
                min_x = min_x.min(x1);
                min_y = min_y.min(y1);
                max_x = max_x.max(x2);
                max_y = max_y.max(y2);
            }
        }

        let patterns: Vec<Pattern> = Pattern::all().to_vec();
        let mut pattern_state = ListState::default();
        pattern_state.select(Some(0));

        let (result_tx, result_rx) = mpsc::channel();

        // Initialize image picker - force Sixel protocol
        let mut picker = Picker::from_termios()
            .unwrap_or_else(|_| Picker::new((8, 16)));
        picker.protocol_type = ProtocolType::Sixel;

        let mut app = App {
            polygons,
            pattern_state,
            patterns,
            spacing: 2.5,
            angle: 45.0,
            lines: Vec::new(),
            cached_tree: None,
            bounds: (min_x, min_y, max_x, max_y),
            gen_time_ms: 0.0,
            should_quit: false,
            setting_focus: 0,
            svg_path: svg_path.to_string(),
            is_loading: false,
            needs_regenerate: false,
            result_rx,
            result_tx,
            spinner_frame: 0,
            picker,
            image_state: None,
            needs_image_update: true,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            last_view_change: None,
            pattern_list_area: None,
            image_area: None,
            drag_start: None,
            drag_start_pan: (0.0, 0.0),
            render_style: RenderStyle::default(),
            color_index: 0,
            render_time_ms: 0.0,
            style_changed: false,
            render_width: 1920,  // Default, will be updated on first frame
            render_height: 1080,
            show_solid_fill: true,  // Default ON per user request
            solid_fill_tree: None,
            status_message: None,
        };

        // Build solid fill reference tree
        app.rebuild_solid_fill_tree();
        app.regenerate_pattern();
        Ok(app)
    }

    /// Build solid fill SVG tree for reference rendering
    fn rebuild_solid_fill_tree(&mut self) {
        let svg_content = build_solid_fill_svg(&self.polygons, self.bounds);
        self.solid_fill_tree = Some(parse_svg_tree(&svg_content));
    }

    fn selected_pattern(&self) -> Pattern {
        self.patterns[self.pattern_state.selected().unwrap_or(0)]
    }

    fn regenerate_pattern(&mut self) {
        // Skip if already loading - mark for regeneration after completion
        if self.is_loading {
            self.needs_regenerate = true;
            return;
        }

        self.needs_regenerate = false;
        self.style_changed = false;
        let pattern = self.selected_pattern();
        let spacing = self.spacing;
        let angle = self.angle;
        let polygons = self.polygons.clone();
        let bounds = self.bounds;
        let tx = self.result_tx.clone();
        let style = self.render_style.clone();

        self.is_loading = true;

        thread::spawn(move || {
            let start = Instant::now();
            let mut lines = Vec::new();

            for polygon in &polygons {
                let poly_lines = generate_pattern(pattern, &polygon, spacing, angle);
                lines.extend(poly_lines);
            }

            // Build SVG content and parse tree (cached for zoom/pan)
            let svg_content = build_svg_content(&lines, &polygons, bounds, &style);
            let svg_tree = parse_svg_tree(&svg_content);

            let gen_time_ms = start.elapsed().as_secs_f64() * 1000.0;
            let _ = tx.send(PatternResult { lines, gen_time_ms, svg_tree });
        });
    }

    /// Rebuild the SVG tree when style changes (without regenerating lines)
    fn rebuild_svg_tree(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        let svg_content = build_svg_content(&self.lines, &self.polygons, self.bounds, &self.render_style);
        self.cached_tree = Some(parse_svg_tree(&svg_content));
        self.needs_image_update = true;
        self.style_changed = false;
    }

    fn check_pattern_result(&mut self) {
        // Drain all pending results, keep only the latest
        let mut latest: Option<PatternResult> = None;
        while let Ok(result) = self.result_rx.try_recv() {
            latest = Some(result);
        }

        if let Some(result) = latest {
            self.lines = result.lines;
            self.gen_time_ms = result.gen_time_ms;
            self.cached_tree = Some(result.svg_tree);
            self.is_loading = false;
            self.needs_image_update = true;

            // If user changed settings while we were generating, regenerate now
            if self.needs_regenerate {
                self.regenerate_pattern();
            }
        }
    }

    /// Debounce delay for view changes
    const VIEW_DEBOUNCE_MS: u64 = 100;

    fn update_image(&mut self) {
        if self.is_loading {
            return;
        }

        // Rebuild SVG tree if style changed
        if self.style_changed {
            self.rebuild_svg_tree();
        }

        // Check if debounce period has passed for view changes
        if let Some(last_change) = self.last_view_change {
            if last_change.elapsed() >= Duration::from_millis(Self::VIEW_DEBOUNCE_MS) {
                self.needs_image_update = true;
                self.last_view_change = None;
            }
        }

        // Render if needed
        if self.needs_image_update {
            let start = Instant::now();

            // Choose which tree to render based on solid fill toggle
            let tree_to_render = if self.show_solid_fill {
                self.solid_fill_tree.as_ref()
            } else {
                self.cached_tree.as_ref()
            };

            if let Some(tree) = tree_to_render {
                let img = render_tree_to_image(
                    tree,
                    self.bounds,
                    self.zoom,
                    self.pan_x,
                    self.pan_y,
                    self.render_width,
                    self.render_height,
                );
                self.image_state = Some(self.picker.new_resize_protocol(img));
                self.render_time_ms = start.elapsed().as_secs_f64() * 1000.0;
            }
            self.needs_image_update = false;
        }
    }

    /// Update render dimensions based on terminal area (called from UI)
    fn update_render_size(&mut self, width_cells: u16, height_cells: u16) {
        let new_width = (width_cells as u32 * PIXELS_PER_CELL_X).max(400);
        let new_height = (height_cells as u32 * PIXELS_PER_CELL_Y).max(300);

        if new_width != self.render_width || new_height != self.render_height {
            self.render_width = new_width;
            self.render_height = new_height;
            self.needs_image_update = true;
        }
    }

    /// Toggle solid fill reference view
    fn toggle_solid_fill(&mut self) {
        self.show_solid_fill = !self.show_solid_fill;
        self.needs_image_update = true;
    }

    /// Save current render as PNG screenshot
    /// Returns the path where the screenshot was saved, or an error message
    fn save_screenshot(&self, output_dir: Option<&str>) -> Result<PathBuf, String> {
        // Determine which tree to render
        let tree = if self.show_solid_fill {
            self.solid_fill_tree.as_ref()
        } else {
            self.cached_tree.as_ref()
        };

        let tree = tree.ok_or("No image to save")?;

        // Render at current zoom/pan
        let img = render_tree_to_image(
            tree,
            self.bounds,
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.render_width,
            self.render_height,
        );

        // Determine output directory
        let dir = match output_dir {
            Some(d) => PathBuf::from(d),
            None => PathBuf::from("screenshots"),
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;

        // Generate filename: pattern_YYYYMMDD_HHMMSS.png
        let pattern_name = self.selected_pattern().name();
        let view_type = if self.show_solid_fill { "solid" } else { "pattern" };
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}_s{:.1}_a{:.0}_{}.png",
            pattern_name, view_type, self.spacing, self.angle, timestamp);

        let path = dir.join(&filename);

        // Save image
        img.save(&path).map_err(|e| format!("Failed to save: {}", e))?;

        Ok(path)
    }

    /// Save both solid fill and pattern renders for visual comparison
    /// Returns paths to both screenshots
    #[allow(dead_code)]
    fn save_comparison_screenshots(&mut self, output_dir: &str) -> Result<(PathBuf, PathBuf), String> {
        let dir = PathBuf::from(output_dir);
        fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;

        let pattern_name = self.selected_pattern().name();
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");

        // Save solid fill
        let solid_tree = self.solid_fill_tree.as_ref()
            .ok_or("No solid fill tree")?;
        let solid_img = render_tree_to_image(
            solid_tree,
            self.bounds,
            1.0, 0.0, 0.0,  // Reset view for consistent comparison
            self.render_width,
            self.render_height,
        );
        let solid_path = dir.join(format!("{}_solid_{}.png", pattern_name, timestamp));
        solid_img.save(&solid_path).map_err(|e| format!("Failed to save solid: {}", e))?;

        // Save pattern
        let pattern_tree = self.cached_tree.as_ref()
            .ok_or("No pattern tree")?;
        let pattern_img = render_tree_to_image(
            pattern_tree,
            self.bounds,
            1.0, 0.0, 0.0,  // Reset view for consistent comparison
            self.render_width,
            self.render_height,
        );
        let pattern_path = dir.join(format!("{}_pattern_{}.png", pattern_name, timestamp));
        pattern_img.save(&pattern_path).map_err(|e| format!("Failed to save pattern: {}", e))?;

        Ok((solid_path, pattern_path))
    }

    /// Set a status message that will be displayed temporarily
    fn set_status(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    /// Clear status message if it's been shown long enough (3 seconds)
    fn clear_old_status(&mut self) {
        if let Some((_, created)) = &self.status_message {
            if created.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
            }
        }
    }

    fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.25).min(10.0);
        self.last_view_change = Some(Instant::now());
    }

    fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.25).max(0.5);
        // Reset pan if zooming out to fit
        if self.zoom <= 1.0 {
            self.pan_x = 0.0;
            self.pan_y = 0.0;
        }
        self.last_view_change = Some(Instant::now());
    }

    fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.needs_image_update = true;
    }

    fn pan(&mut self, dx: f64, dy: f64) {
        // Pan speed scales with zoom level
        let pan_speed = 50.0 / self.zoom;
        self.pan_x += dx * pan_speed;
        self.pan_y += dy * pan_speed;
        self.last_view_change = Some(Instant::now());
    }

    /// Cycle to next stroke color
    fn next_color(&mut self) {
        self.color_index = (self.color_index + 1) % STROKE_COLORS.len();
        self.render_style.stroke_color = STROKE_COLORS[self.color_index].1.to_string();
        self.style_changed = true;
    }

    /// Cycle to previous stroke color
    fn prev_color(&mut self) {
        self.color_index = if self.color_index == 0 {
            STROKE_COLORS.len() - 1
        } else {
            self.color_index - 1
        };
        self.render_style.stroke_color = STROKE_COLORS[self.color_index].1.to_string();
        self.style_changed = true;
    }

    /// Increase stroke width
    fn increase_stroke_width(&mut self) {
        self.render_style.stroke_width = (self.render_style.stroke_width + 0.5).min(10.0);
        self.style_changed = true;
    }

    /// Decrease stroke width
    fn decrease_stroke_width(&mut self) {
        self.render_style.stroke_width = (self.render_style.stroke_width - 0.5).max(0.5);
        self.style_changed = true;
    }

    /// Toggle polygon stroke visibility
    fn toggle_strokes(&mut self) {
        self.render_style.show_strokes = !self.render_style.show_strokes;
        self.style_changed = true;
    }

    /// Select a pattern by index (for mouse clicks)
    fn select_pattern(&mut self, index: usize) {
        if index < self.patterns.len() {
            self.pattern_state.select(Some(index));
            self.regenerate_pattern();
        }
    }

    /// Handle mouse click at terminal coordinates
    fn handle_click(&mut self, x: u16, y: u16) {
        // Check pattern list first
        if let Some(area) = self.pattern_list_area {
            let inner_x = area.x + 1;
            let inner_y = area.y + 1;
            let inner_width = area.width.saturating_sub(2);
            let inner_height = area.height.saturating_sub(2);

            if x >= inner_x && x < inner_x + inner_width
                && y >= inner_y && y < inner_y + inner_height
            {
                let clicked_index = (y - inner_y) as usize;
                let offset = self.pattern_state.offset();
                let actual_index = offset + clicked_index;
                self.select_pattern(actual_index);
                return;
            }
        }

        // Check if click is in image area - start drag
        if let Some(area) = self.image_area {
            if x >= area.x && x < area.x + area.width
                && y >= area.y && y < area.y + area.height
            {
                self.drag_start = Some((x, y));
                self.drag_start_pan = (self.pan_x, self.pan_y);
            }
        }
    }

    /// Handle mouse drag
    fn handle_drag(&mut self, x: u16, y: u16) {
        if let Some((start_x, start_y)) = self.drag_start {
            // Calculate delta in terminal cells
            let dx = x as i32 - start_x as i32;
            let dy = y as i32 - start_y as i32;

            // Convert terminal cells to pan units
            // Each cell is roughly 8x16 pixels, scale by zoom
            let pan_scale = 8.0 / self.zoom;

            self.pan_x = self.drag_start_pan.0 - dx as f64 * pan_scale;
            self.pan_y = self.drag_start_pan.1 - dy as f64 * pan_scale;

            self.last_view_change = Some(Instant::now());
        }
    }

    /// Handle mouse release
    fn handle_release(&mut self) {
        self.drag_start = None;
    }

    fn next_pattern(&mut self) {
        let i = match self.pattern_state.selected() {
            Some(i) => (i + 1) % self.patterns.len(),
            None => 0,
        };
        self.pattern_state.select(Some(i));
        self.regenerate_pattern();
    }

    fn prev_pattern(&mut self) {
        let i = match self.pattern_state.selected() {
            Some(i) => {
                if i == 0 { self.patterns.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.pattern_state.select(Some(i));
        self.regenerate_pattern();
    }

    /// Adjust the currently focused setting
    /// delta is a direction/magnitude indicator: positive = increase, negative = decrease
    fn adjust_current_setting(&mut self, delta: i32) {
        match self.setting_focus {
            0 => {
                // Stroke width
                if delta > 0 {
                    self.increase_stroke_width();
                } else {
                    self.decrease_stroke_width();
                }
            }
            1 => {
                // Color
                if delta > 0 {
                    self.next_color();
                } else {
                    self.prev_color();
                }
            }
            2 => {
                // Strokes toggle (any direction toggles)
                self.toggle_strokes();
            }
            3 => {
                // Pattern spacing
                let step = if delta.abs() > 1 { 5.0 } else { 0.5 };
                self.spacing = (self.spacing + step * delta.signum() as f64).max(0.5).min(50.0);
                self.regenerate_pattern();
            }
            4 => {
                // Pattern angle
                let step = if delta.abs() > 1 { 15.0 } else { 5.0 };
                self.angle = (self.angle + step * delta.signum() as f64) % 360.0;
                if self.angle < 0.0 { self.angle += 360.0; }
                self.regenerate_pattern();
            }
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for CLI subcommands
    if args.len() >= 2 {
        match args[1].as_str() {
            "fill" => {
                cmd_fill(&args[2..]);
                return;
            }
            "benchmark" => {
                cmd_benchmark(&args[2..]);
                return;
            }
            "patterns" => {
                cmd_patterns();
                return;
            }
            "harness" => {
                cmd_harness(&args[2..]);
                return;
            }
            "help" | "--help" | "-h" => {
                print_usage(&args[0]);
                return;
            }
            _ => {}
        }
    }

    // Launch TUI
    let svg_path = if args.len() >= 2 && args[1].ends_with(".svg") {
        args[1].clone()
    } else {
        // Try to find a default SVG file
        let candidates = [
            "test_assets/essex.svg",
            "../test_assets/essex.svg",
            "essex.svg",
        ];

        let mut found: Option<String> = None;
        for candidate in &candidates {
            if std::path::Path::new(candidate).exists() {
                found = Some(candidate.to_string());
                break;
            }
        }

        match found {
            Some(path) => path,
            None => {
                eprintln!("Usage: rat-king <svg_file>");
                eprintln!();
                eprintln!("No SVG file specified and no default found.");
                eprintln!("Please provide an SVG file path.");
                eprintln!();
                eprintln!("Examples:");
                eprintln!("  rat-king mydesign.svg");
                eprintln!("  rat-king ~/Downloads/artwork.svg");
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = run_tui(&svg_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_tui(svg_path: &str) -> Result<(), String> {
    // Initialize terminal
    enable_raw_mode().map_err(|e| e.to_string())?;
    stdout().execute(EnterAlternateScreen).map_err(|e| e.to_string())?;
    stdout().execute(EnableMouseCapture).map_err(|e| e.to_string())?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .map_err(|e| e.to_string())?;

    // Create app
    let mut app = App::new(svg_path)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    stdout().execute(DisableMouseCapture).map_err(|e| e.to_string())?;
    disable_raw_mode().map_err(|e| e.to_string())?;
    stdout().execute(LeaveAlternateScreen).map_err(|e| e.to_string())?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<(), String> {
    loop {
        // Check for completed pattern generation (non-blocking)
        app.check_pattern_result();

        // Update the rendered image if needed
        app.update_image();

        // Clear old status messages
        app.clear_old_status();

        // Animate spinner while loading
        if app.is_loading {
            app.spinner_frame = (app.spinner_frame + 1) % 8;
        }

        terminal.draw(|frame| ui(frame, app)).map_err(|_| "Draw error".to_string())?;

        if event::poll(Duration::from_millis(50)).map_err(|e| e.to_string())? {
            match event::read().map_err(|e| e.to_string())? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.prev_pattern();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.next_pattern();
                            }
                            KeyCode::Tab => {
                                // Cycle through: width(0), color(1), strokes(2), spacing(3), angle(4)
                                app.setting_focus = (app.setting_focus + 1) % 5;
                            }
                            KeyCode::BackTab => {
                                // Reverse cycle
                                app.setting_focus = if app.setting_focus == 0 { 4 } else { app.setting_focus - 1 };
                            }
                            KeyCode::Left | KeyCode::Char('h') => {
                                app.adjust_current_setting(-1);
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                app.adjust_current_setting(1);
                            }
                            KeyCode::Char('[') => {
                                app.adjust_current_setting(-10);
                            }
                            KeyCode::Char(']') => {
                                app.adjust_current_setting(10);
                            }
                            // Zoom controls
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                app.zoom_in();
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                app.zoom_out();
                            }
                            KeyCode::Char('0') => {
                                app.reset_view();
                            }
                            // Toggle solid fill reference view
                            KeyCode::Char('f') => {
                                app.toggle_solid_fill();
                            }
                            // Pan controls (WASD)
                            KeyCode::Char('w') => {
                                app.pan(0.0, -1.0);
                            }
                            KeyCode::Char('s') => {
                                app.pan(0.0, 1.0);
                            }
                            KeyCode::Char('a') => {
                                app.pan(-1.0, 0.0);
                            }
                            KeyCode::Char('d') => {
                                app.pan(1.0, 0.0);
                            }
                            // Screenshot (Shift+S)
                            KeyCode::Char('S') => {
                                match app.save_screenshot(None) {
                                    Ok(path) => {
                                        app.set_status(format!("Saved: {}", path.display()));
                                    }
                                    Err(e) => {
                                        app.set_status(format!("Error: {}", e));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::Down(_) => {
                            app.handle_click(mouse.column, mouse.row);
                        }
                        MouseEventKind::Up(_) => {
                            app.handle_release();
                        }
                        MouseEventKind::Drag(_) => {
                            app.handle_drag(mouse.column, mouse.row);
                        }
                        MouseEventKind::ScrollUp => {
                            app.zoom_in();
                        }
                        MouseEventKind::ScrollDown => {
                            app.zoom_out();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Get pattern-specific labels and descriptions for the settings panel.
/// Returns (spacing_label, angle_label, description)
fn get_pattern_settings_info(pattern: Pattern) -> (&'static str, &'static str, &'static str) {
    let meta = pattern.metadata();
    (meta.spacing_label, meta.angle_label, meta.description)
}

fn ui(frame: &mut Frame, app: &mut App) {
    // Main horizontal layout: sidebar | image
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26),
            Constraint::Min(40),
        ])
        .split(frame.area());

    // Split left sidebar into: patterns list | stats | style settings | pattern settings
    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),      // Pattern list (flexible)
            Constraint::Length(7),    // Stats (increased for render size info)
            Constraint::Length(7),    // Style settings (line width, color, strokes)
            Constraint::Length(7),    // Pattern-specific settings
        ])
        .split(main_layout[0]);

    // Pattern list
    let items: Vec<ListItem> = app.patterns
        .iter()
        .map(|p| ListItem::new(p.name()))
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title(" Patterns ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)))
        .highlight_style(Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");

    // Store the pattern list area for mouse hit testing
    app.pattern_list_area = Some(sidebar_layout[0]);

    frame.render_stateful_widget(list, sidebar_layout[0], &mut app.pattern_state.clone());

    // Stats panel
    let view_mode = if app.show_solid_fill { "SOLID [f]" } else { "PATTERN [f]" };
    let stats_text = format!(
        "Polys: {}  Lines: {}\nGen: {:.0}ms Rnd: {:.0}ms\nZoom: {:.0}%  {}\n{}x{}px",
        app.polygons.len(),
        app.lines.len(),
        app.gen_time_ms,
        app.render_time_ms,
        app.zoom * 100.0,
        view_mode,
        app.render_width,
        app.render_height
    );
    let stats = Paragraph::new(stats_text)
        .block(Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White));

    frame.render_widget(stats, sidebar_layout[1]);

    // Style settings panel
    let color_name = STROKE_COLORS[app.color_index].0;
    let strokes_str = if app.render_style.show_strokes { "ON" } else { "OFF" };

    // Determine which setting is highlighted
    let width_style = if app.setting_focus == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let color_style = if app.setting_focus == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let stroke_style = if app.setting_focus == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let style_text = ratatui::text::Line::from(vec![
        Span::styled(format!("Width: {:.1}", app.render_style.stroke_width), width_style),
    ]);
    let color_line = ratatui::text::Line::from(vec![
        Span::styled(format!("Color: {}", color_name), color_style),
    ]);
    let stroke_line = ratatui::text::Line::from(vec![
        Span::styled(format!("Strokes: {}", strokes_str), stroke_style),
    ]);

    let style_para = Paragraph::new(vec![style_text, color_line, stroke_line])
        .block(Block::default()
            .title(" Style [Tab:switch ←→:adj] ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));

    frame.render_widget(style_para, sidebar_layout[2]);

    // Pattern-specific settings panel
    let (spacing_label, angle_label, pattern_desc) = get_pattern_settings_info(app.selected_pattern());

    let spacing_style = if app.setting_focus == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let angle_style = if app.setting_focus == 4 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let pattern_settings = Paragraph::new(vec![
        ratatui::text::Line::from(vec![
            Span::styled(format!("{}: {:.1}", spacing_label, app.spacing), spacing_style),
        ]),
        ratatui::text::Line::from(vec![
            Span::styled(format!("{}: {:.0}°", angle_label, app.angle), angle_style),
        ]),
        ratatui::text::Line::from(Span::styled(pattern_desc, Style::default().fg(Color::DarkGray))),
    ])
        .block(Block::default()
            .title(" Pattern Settings ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)));

    frame.render_widget(pattern_settings, sidebar_layout[3]);

    // Image panel (full right side)
    let spinner_chars = ['|', '/', '-', '\\', '|', '/', '-', '\\'];
    let spinner = spinner_chars[app.spinner_frame % spinner_chars.len()];

    let image_title = if app.is_loading {
        format!(" [{}] Generating... ", spinner)
    } else {
        format!(" {} ", app.svg_path)
    };

    let border_color = if app.is_loading { Color::Yellow } else { Color::Green };

    let image_block = Block::default()
        .title(image_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = image_block.inner(main_layout[1]);

    // Store image area for drag detection
    app.image_area = Some(main_layout[1]);

    // Update render size based on available area
    app.update_render_size(inner_area.width, inner_area.height);

    frame.render_widget(image_block, main_layout[1]);

    // Render the image using ratatui-image
    if let Some(ref mut image_state) = app.image_state {
        let image_widget = StatefulImage::new(None);
        frame.render_stateful_widget(image_widget, inner_area, image_state);
    }

    // Display status message if present (overlay at bottom of image)
    if let Some((message, _)) = &app.status_message {
        let status_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(1),
            width: inner_area.width,
            height: 1,
        };
        let status_widget = Paragraph::new(message.as_str())
            .style(Style::default().fg(Color::Green).bg(Color::Black));
        frame.render_widget(status_widget, status_area);
    }
}

// ============ CLI Commands ============

fn print_usage(prog: &str) {
    eprintln!("rat-king - fast pattern generation for SVG polygons");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  {} [svg_file]                      Launch TUI", prog);
    eprintln!("  {} fill <svg> -p <pattern> [options]", prog);
    eprintln!("  {} benchmark <svg> [-p <pattern>]", prog);
    eprintln!("  {} harness [svg] [-p pattern1,pattern2,...]", prog);
    eprintln!("  {} patterns", prog);
    eprintln!();
    eprintln!("Fill options:");
    eprintln!("  -p, --pattern <name>   Pattern to use (required)");
    eprintln!("  -o, --output <file>    Output file (- for stdout, default: stdout)");
    eprintln!("  -s, --spacing <n>      Line spacing (default: 2.5)");
    eprintln!("  -a, --angle <deg>      Pattern angle (default: 45)");
    eprintln!("  -f, --format <fmt>     Output format: svg, json (default: svg)");
    eprintln!("  --order <strategy>     Polygon ordering: document, nearest (default: nearest)");
    eprintln!("  --grouped              JSON only: group lines by input shape");
    eprintln!("  --strokes              Include polygon outlines as line geometry");
    eprintln!("  --sketchy              Apply hand-drawn/sketchy effect to all lines");
    eprintln!("  --roughness <n>        Sketchy: endpoint randomization (default: 1.0)");
    eprintln!("  --bowing <n>           Sketchy: line curvature amount (default: 1.0)");
    eprintln!("  --no-double-stroke     Sketchy: disable double-stroke effect");
    eprintln!("  --seed <n>             Sketchy: random seed for reproducibility");
    eprintln!();
    eprintln!("Harness options:");
    eprintln!("  -p, --patterns   Comma-separated list of patterns (default: all)");
    eprintln!("  -s, --spacing    Spacing value (default: 2.5)");
    eprintln!("  -a, --angle      Angle value (default: 45)");
    eprintln!("  --json           Output results as JSON");
    eprintln!("  --visual         Visual mode: save PNG screenshots for each pattern");
    eprintln!("  --analyze        Analyze patterns for bounds/coverage (implies --visual)");
    eprintln!("  -o, --output     Output directory for screenshots (default: harness_output)");
    eprintln!("  --width          Screenshot width in pixels (default: 800)");
    eprintln!("  --height         Screenshot height in pixels (default: 600)");
    eprintln!();
    eprintln!("Stdin support:");
    eprintln!("  Use '-' as input file to read SVG from stdin:");
    eprintln!("  echo '<svg>...</svg>' | {} fill - -p lines -o -", prog);
    eprintln!();
    eprintln!("TUI Controls:");
    eprintln!("  ↑/↓ or j/k    Select pattern");
    eprintln!("  ←/→ or h/l    Adjust setting (fine)");
    eprintln!("  [ / ]         Adjust setting (coarse)");
    eprintln!("  Tab           Switch between settings");
    eprintln!("  +/-           Zoom in/out");
    eprintln!("  WASD          Pan view");
    eprintln!("  0             Reset view");
    eprintln!("  f             Toggle solid fill / pattern view");
    eprintln!("  Shift+S       Save screenshot (PNG)");
    eprintln!("  q / Esc       Quit");
}

fn cmd_patterns() {
    println!("Available patterns:");
    for pattern in Pattern::all() {
        println!("  {}", pattern.name());
    }
}

// cmd_fill, cmd_benchmark, and harness utilities are now in cli/ module

fn cmd_harness(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut pattern_filter: Option<Vec<&str>> = None;
    let mut spacing = 2.5;
    let mut angle = 45.0;
    let mut json_output = false;
    let mut visual_mode = false;
    let mut analyze_mode = false;
    let mut output_dir = "harness_output".to_string();
    let mut render_width: u32 = 800;
    let mut render_height: u32 = 600;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--patterns" => {
                i += 1;
                if i < args.len() {
                    pattern_filter = Some(args[i].split(',').collect());
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
            "--json" => {
                json_output = true;
            }
            "--visual" => {
                visual_mode = true;
                json_output = true;  // Visual mode always outputs JSON
            }
            "--analyze" => {
                analyze_mode = true;
                visual_mode = true;  // Analysis requires visual mode
                json_output = true;
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_dir = args[i].clone();
                }
            }
            "--width" => {
                i += 1;
                if i < args.len() {
                    render_width = args[i].parse().unwrap_or(800);
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    render_height = args[i].parse().unwrap_or(600);
                }
            }
            path => {
                if svg_path.is_none() && !path.starts_with('-') {
                    svg_path = Some(path);
                }
            }
        }
        i += 1;
    }

    // Default to essex.svg if no file provided
    let svg_path = svg_path.unwrap_or_else(|| {
        let candidates = [
            "test_assets/essex.svg",
            "../test_assets/essex.svg",
            "essex.svg",
        ];
        for candidate in &candidates {
            if std::path::Path::new(candidate).exists() {
                return *candidate;
            }
        }
        eprintln!("Error: No SVG file specified and default essex.svg not found");
        eprintln!("Usage: rat-king harness [svg_file] [-p pattern1,pattern2]");
        std::process::exit(1);
    });

    // Load SVG
    let svg_content = match fs::read_to_string(svg_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading {}: {}", svg_path, e);
            std::process::exit(1);
        }
    };

    let polygons = match extract_polygons_from_svg(&svg_content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing SVG: {}", e);
            std::process::exit(1);
        }
    };

    // Determine which patterns to run
    let patterns_to_run: Vec<Pattern> = if let Some(filter) = &pattern_filter {
        filter.iter()
            .filter_map(|name| Pattern::from_name(name))
            .collect()
    } else {
        Pattern::all().to_vec()
    };

    if patterns_to_run.is_empty() {
        eprintln!("Error: No valid patterns specified");
        std::process::exit(1);
    }

    if !json_output {
        eprintln!("rat-king harness");
        eprintln!("================");
        eprintln!("SVG: {} ({} polygons)", svg_path, polygons.len());
        eprintln!("Spacing: {}, Angle: {}°", spacing, angle);
        eprintln!("Patterns: {}\n", patterns_to_run.len());
    }

    // Calculate bounding box for rendering
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for poly in &polygons {
        if let Some((x1, y1, x2, y2)) = poly.bounding_box() {
            min_x = min_x.min(x1);
            min_y = min_y.min(y1);
            max_x = max_x.max(x2);
            max_y = max_y.max(y2);
        }
    }
    let bounds = (min_x, min_y, max_x, max_y);

    // Create output directory for visual mode
    // We keep two solid images: one for visual display, one for analysis (no strokes)
    let (_solid_img_for_display, solid_img_for_analysis): (Option<DynamicImage>, Option<DynamicImage>) = if visual_mode {
        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Error creating output directory: {}", e);
            std::process::exit(1);
        }
        eprintln!("Visual harness mode: saving screenshots to {}/", output_dir);
        eprintln!("Render size: {}x{}", render_width, render_height);
        if analyze_mode {
            eprintln!("Analysis mode: checking bounds and coverage");
        }

        // Build and save solid fill reference for visual display
        let solid_svg = build_solid_fill_svg(&polygons, bounds);
        let solid_tree = parse_svg_tree(&solid_svg);
        let solid_img = render_tree_to_image(&solid_tree, bounds, 1.0, 0.0, 0.0, render_width, render_height);
        let solid_ref_path = PathBuf::from(&output_dir).join("solid_reference.png");
        if let Err(e) = solid_img.save(&solid_ref_path) {
            eprintln!("Warning: Failed to save solid reference: {}", e);
        } else {
            eprintln!("Saved solid reference: {}", solid_ref_path.display());
        }

        // For analysis, create a version with no strokes for accurate pixel comparison
        let analysis_img = if analyze_mode {
            let analysis_svg = build_solid_fill_svg_internal(&polygons, bounds, true);
            let analysis_tree = parse_svg_tree(&analysis_svg);
            Some(render_tree_to_image(&analysis_tree, bounds, 1.0, 0.0, 0.0, render_width, render_height))
        } else {
            None
        };

        (Some(solid_img), analysis_img)
    } else {
        (None, None)
    };

    let mut results: Vec<HarnessResult> = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    let mut bounds_failures = 0;

    let style = RenderStyle::default();

    for pattern in &patterns_to_run {
        if !json_output {
            eprint!("  {:12} ... ", pattern.name());
        }

        let start = Instant::now();
        let mut total_lines = 0;
        let mut error_msg: Option<String> = None;
        let mut solid_path: Option<String> = None;
        let mut pattern_path: Option<String> = None;
        let mut diff_path: Option<String> = None;
        let mut analysis: Option<AnalysisResult> = None;

        // Use catch_unwind to handle panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut lines: Vec<Line> = Vec::new();
            for polygon in &polygons {
                let poly_lines = generate_pattern(*pattern, polygon, spacing, angle);
                lines.extend(poly_lines);
            }
            lines
        }));

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;

        let (status, lines_count) = match result {
            Ok(lines) => {
                total_lines = lines.len();

                // Visual mode: render and save screenshots
                if visual_mode {
                    // Try to save pattern screenshot (with polygon outlines for visual display)
                    let pattern_svg = build_svg_content(&lines, &polygons, bounds, &style);
                    if let Some(pattern_tree) = try_parse_svg_tree(&pattern_svg) {
                        let pattern_img = render_tree_to_image(&pattern_tree, bounds, 1.0, 0.0, 0.0, render_width, render_height);
                        let pattern_file = format!("{}_pattern.png", pattern.name());
                        let full_path = PathBuf::from(&output_dir).join(&pattern_file);
                        if let Err(e) = pattern_img.save(&full_path) {
                            eprintln!("Warning: Failed to save pattern screenshot: {}", e);
                        } else {
                            pattern_path = Some(pattern_file);
                        }

                        // Run analysis if enabled
                        // For analysis, we render pattern lines only (no outlines) against
                        // the solid fill mask (no strokes) for accurate comparison
                        if analyze_mode {
                            if let Some(ref solid_img) = solid_img_for_analysis {
                                // Render pattern-only version for analysis
                                let analysis_pattern_svg = build_pattern_svg_for_analysis(&lines, bounds, &style);
                                if let Some(analysis_pattern_tree) = try_parse_svg_tree(&analysis_pattern_svg) {
                                    let analysis_pattern_img = render_tree_to_image(&analysis_pattern_tree, bounds, 1.0, 0.0, 0.0, render_width, render_height);

                                    let result = analyze_pattern_vs_solid(solid_img, &analysis_pattern_img);
                                    if !result.bounds_ok {
                                        bounds_failures += 1;
                                    }
                                    analysis = Some(result);

                                    // Generate and save diff image
                                    let diff_img = generate_diff_image(solid_img, &analysis_pattern_img);
                                    let diff_file = format!("{}_diff.png", pattern.name());
                                    let diff_full_path = PathBuf::from(&output_dir).join(&diff_file);
                                    if let Err(e) = diff_img.save(&diff_full_path) {
                                        eprintln!("Warning: Failed to save diff image: {}", e);
                                    } else {
                                        diff_path = Some(diff_file);
                                    }
                                } else {
                                    error_msg = Some("SVG too complex for analysis".to_string());
                                }
                            }
                        }
                    } else {
                        error_msg = Some("SVG too complex to render".to_string());
                    }

                    solid_path = Some("solid_reference.png".to_string());
                }

                passed += 1;
                ("OK".to_string(), total_lines)
            }
            Err(e) => {
                failed += 1;
                error_msg = Some(format!("{:?}", e));
                ("FAIL".to_string(), 0)
            }
        };

        results.push(HarnessResult {
            pattern: pattern.name().to_string(),
            lines: lines_count,
            time_ms,
            status,
            solid_screenshot: solid_path,
            pattern_screenshot: pattern_path,
            diff_image: diff_path,
            error: error_msg.clone(),
            analysis: analysis.clone(),
        });

        if !json_output {
            if error_msg.is_none() {
                if let Some(ref a) = analysis {
                    let bounds_status = if a.bounds_ok { "✓" } else { "✗" };
                    eprintln!("{:>8} lines {:>5.1}% coverage {} bounds {:>8.1}ms",
                        total_lines, a.coverage_percent, bounds_status, time_ms);
                } else {
                    eprintln!("{:>8} lines in {:>8.1}ms", total_lines, time_ms);
                }
            } else {
                eprintln!("FAILED: {:?}", error_msg);
            }
        }
    }

    // Output results
    if json_output {
        // Use proper JSON serialization for structured output
        if visual_mode {
            // Full visual harness report with screenshot paths
            let report = VisualHarnessReport {
                svg: svg_path.to_string(),
                polygons: polygons.len(),
                spacing,
                angle,
                output_dir: output_dir.clone(),
                render_width,
                render_height,
                passed,
                failed,
                results,
            };
            println!("{}", serde_json::to_string_pretty(&report).unwrap());

            // Also write report to file in output directory
            let report_path = PathBuf::from(&output_dir).join("report.json");
            if let Err(e) = fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap()) {
                eprintln!("Warning: Failed to write report.json: {}", e);
            } else {
                eprintln!("Wrote report: {}", report_path.display());
            }
        } else {
            // Simple JSON output (backward compatible)
            println!("{{");
            println!("  \"svg\": \"{}\",", svg_path);
            println!("  \"polygons\": {},", polygons.len());
            println!("  \"spacing\": {},", spacing);
            println!("  \"angle\": {},", angle);
            println!("  \"passed\": {},", passed);
            println!("  \"failed\": {},", failed);
            println!("  \"results\": [");
            for (i, r) in results.iter().enumerate() {
                let comma = if i < results.len() - 1 { "," } else { "" };
                println!("    {{\"pattern\": \"{}\", \"lines\": {}, \"time_ms\": {:.2}, \"status\": \"{}\"}}{}",
                    r.pattern, r.lines, r.time_ms, r.status, comma);
            }
            println!("  ]");
            println!("}}");
        }
    } else {
        // Summary table
        eprintln!("\n════════════════════════════════════════════════════════════════════════════════");
        eprintln!("  HARNESS SUMMARY");
        eprintln!("════════════════════════════════════════════════════════════════════════════════");
        if analyze_mode {
            eprintln!("  {:12}  {:>8}  {:>8}  {:>8}  {:>6}  {:>6}", "Pattern", "Lines", "Time(ms)", "Coverage", "Bounds", "Status");
            eprintln!("  {:12}  {:>8}  {:>8}  {:>8}  {:>6}  {:>6}", "-------", "-----", "--------", "--------", "------", "------");
        } else {
            eprintln!("  {:12}  {:>10}  {:>10}  {:>6}", "Pattern", "Lines", "Time(ms)", "Status");
            eprintln!("  {:12}  {:>10}  {:>10}  {:>6}", "-------", "-----", "--------", "------");
        }

        let mut total_lines = 0;
        let mut total_time = 0.0;

        for r in &results {
            let status_str = if r.status == "OK" { "✓" } else { "✗" };
            if let Some(ref a) = r.analysis {
                let bounds_str = if a.bounds_ok { "✓" } else { "✗" };
                eprintln!("  {:12}  {:>8}  {:>8.1}  {:>7.1}%  {:>6}  {:>6}",
                    r.pattern, r.lines, r.time_ms, a.coverage_percent, bounds_str, status_str);
            } else {
                eprintln!("  {:12}  {:>10}  {:>10.1}  {:>6}", r.pattern, r.lines, r.time_ms, status_str);
            }
            total_lines += r.lines;
            total_time += r.time_ms;
        }

        eprintln!("════════════════════════════════════════════════════════════════════════════════");
        eprintln!("  TOTAL: {} lines in {:.1}ms", total_lines, total_time);
        eprintln!("  Passed: {}  Failed: {}", passed, failed);
        if analyze_mode {
            eprintln!("  Bounds failures: {}", bounds_failures);
        }
        eprintln!("════════════════════════════════════════════════════════════════════════════════");

        if failed > 0 || bounds_failures > 0 {
            std::process::exit(1);
        }
    }
}

