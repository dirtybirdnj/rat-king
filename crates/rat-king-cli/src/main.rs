//! rat-king - TUI and CLI for pattern generation
//!
//! Usage:
//!   rat-king [svg_file]              Launch TUI (default: test_assets/essex.svg)
//!   rat-king fill <svg> -p <pattern> Generate pattern fill
//!   rat-king benchmark <svg>         Benchmark pattern generation
//!   rat-king patterns                List available patterns

use std::env;
use std::fs;
use std::io::{self, stdout, Read as IoRead};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
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

use rat_king::{
    extract_polygons_from_svg, Line, Pattern, Polygon,
    order_polygons, calculate_travel_distance, OrderingStrategy,
    patterns::{
        generate_lines_fill, generate_crosshatch_fill,
        generate_zigzag_fill, generate_wiggle_fill,
        generate_spiral_fill, generate_fermat_fill,
        generate_concentric_fill, generate_radial_fill,
        generate_honeycomb_fill, generate_scribble_fill,
        generate_crossspiral_fill, generate_hilbert_fill,
        generate_gyroid_fill, generate_guilloche_fill,
        generate_lissajous_fill, generate_rose_fill,
        generate_phyllotaxis_fill, generate_pentagon15_fill,
        generate_pentagon14_fill, generate_grid_fill,
        generate_brick_fill, generate_truchet_fill,
        generate_stipple_fill, generate_peano_fill,
        generate_sierpinski_fill, generate_diagonal_fill,
        generate_herringbone_fill, generate_stripe_fill,
    },
};

// Image rendering constants - wide aspect ratio for terminal display
const IMAGE_WIDTH: u32 = 4800;
const IMAGE_HEIGHT: u32 = 2700;
const STROKE_WIDTH: f64 = 2.0;

/// Render pattern lines and polygon outlines to an image using resvg
fn render_to_image(
    lines: &[Line],
    polygons: &[Polygon],
    bounds: (f64, f64, f64, f64),
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
) -> DynamicImage {
    let (min_x, min_y, max_x, max_y) = bounds;
    let svg_width = max_x - min_x;
    let svg_height = max_y - min_y;

    // Calculate base scale to fit image dimensions with padding
    let padding = 20.0;
    let scale_x = (IMAGE_WIDTH as f64 - padding * 2.0) / svg_width;
    let scale_y = (IMAGE_HEIGHT as f64 - padding * 2.0) / svg_height;
    let base_scale = scale_x.min(scale_y);

    // Apply zoom
    let scale = base_scale * zoom;

    // Calculate center offset, then apply pan
    let center_x = IMAGE_WIDTH as f64 / 2.0;
    let center_y = IMAGE_HEIGHT as f64 / 2.0;
    let content_center_x = (min_x + max_x) / 2.0;
    let content_center_y = (min_y + max_y) / 2.0;

    // Pan is in SVG units, scaled by zoom
    let translate_x = center_x - content_center_x * scale - pan_x * base_scale;
    let translate_y = center_y - content_center_y * scale - pan_y * base_scale;

    // Build SVG string
    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
<rect width="100%" height="100%" fill="white"/>
<g transform="translate({}, {}) scale({})">
"#,
        IMAGE_WIDTH, IMAGE_HEIGHT,
        IMAGE_WIDTH, IMAGE_HEIGHT,
        translate_x,
        translate_y,
        scale
    ));

    // Draw polygon outlines (gray, thinner)
    svg.push_str("<g stroke=\"#cccccc\" stroke-width=\"");
    svg.push_str(&format!("{}", STROKE_WIDTH / scale));
    svg.push_str("\" fill=\"none\">");
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

    // Draw pattern lines (black)
    svg.push_str(&format!(r#"<g stroke="black" stroke-width="{}" stroke-linecap="round" fill="none">"#, STROKE_WIDTH / scale));
    for line in lines {
        svg.push_str(&format!(
            "<line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\"/>\n",
            line.x1, line.y1, line.x2, line.y2
        ));
    }
    svg.push_str("</g>\n</g>\n</svg>");

    // Parse and render with resvg
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &options)
        .expect("Failed to parse generated SVG");

    let mut pixmap = Pixmap::new(IMAGE_WIDTH, IMAGE_HEIGHT)
        .expect("Failed to create pixmap");

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // Convert to image::DynamicImage
    let rgba = RgbaImage::from_raw(IMAGE_WIDTH, IMAGE_HEIGHT, pixmap.take())
        .expect("Failed to create image");

    DynamicImage::ImageRgba8(rgba)
}

/// Result from background pattern generation
struct PatternResult {
    lines: Vec<Line>,
    gen_time_ms: f64,
}

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
    /// Bounding box of all polygons
    bounds: (f64, f64, f64, f64),
    /// Last generation time
    gen_time_ms: f64,
    /// Should exit
    should_quit: bool,
    /// Which setting is focused (0=spacing, 1=angle)
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
        };

        app.regenerate_pattern();
        Ok(app)
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
        let pattern = self.selected_pattern();
        let spacing = self.spacing;
        let angle = self.angle;
        let polygons = self.polygons.clone();
        let tx = self.result_tx.clone();

        self.is_loading = true;

        thread::spawn(move || {
            let start = Instant::now();
            let mut lines = Vec::new();

            for polygon in &polygons {
                let poly_lines = generate_pattern(pattern, &polygon, spacing, angle);
                lines.extend(poly_lines);
            }

            let gen_time_ms = start.elapsed().as_secs_f64() * 1000.0;
            let _ = tx.send(PatternResult { lines, gen_time_ms });
        });
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
            self.is_loading = false;
            self.needs_image_update = true;

            // If user changed settings while we were generating, regenerate now
            if self.needs_regenerate {
                self.regenerate_pattern();
            }
        }
    }

    fn update_image(&mut self) {
        if self.needs_image_update && !self.is_loading {
            let img = render_to_image(&self.lines, &self.polygons, self.bounds, self.zoom, self.pan_x, self.pan_y);
            self.image_state = Some(self.picker.new_resize_protocol(img));
            self.needs_image_update = false;
        }
    }

    fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.25).min(10.0);
        self.needs_image_update = true;
    }

    fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.25).max(0.5);
        // Reset pan if zooming out to fit
        if self.zoom <= 1.0 {
            self.pan_x = 0.0;
            self.pan_y = 0.0;
        }
        self.needs_image_update = true;
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
        self.needs_image_update = true;
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

    fn adjust_setting(&mut self, delta: f64) {
        match self.setting_focus {
            0 => {
                self.spacing = (self.spacing + delta).max(0.5).min(50.0);
            }
            1 => {
                self.angle = (self.angle + delta * 5.0) % 360.0;
                if self.angle < 0.0 { self.angle += 360.0; }
            }
            _ => {}
        }
        self.regenerate_pattern();
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
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))
        .map_err(|e| e.to_string())?;

    // Create app
    let mut app = App::new(svg_path)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
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

        // Animate spinner while loading
        if app.is_loading {
            app.spinner_frame = (app.spinner_frame + 1) % 8;
        }

        terminal.draw(|frame| ui(frame, app)).map_err(|_| "Draw error".to_string())?;

        if event::poll(Duration::from_millis(50)).map_err(|e| e.to_string())? {
            if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
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
                            app.setting_focus = (app.setting_focus + 1) % 2;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            app.adjust_setting(-0.5);
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            app.adjust_setting(0.5);
                        }
                        KeyCode::Char('[') => {
                            app.adjust_setting(-5.0);
                        }
                        KeyCode::Char(']') => {
                            app.adjust_setting(5.0);
                        }
                        // Zoom controls
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            app.zoom_in();
                        }
                        KeyCode::Char('-') | KeyCode::Char('_') => {
                            app.zoom_out();
                        }
                        KeyCode::Char('0') | KeyCode::Char('r') => {
                            app.reset_view();
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
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(frame.area());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Min(40),
        ])
        .split(main_layout[0]);

    // Split left sidebar into patterns list and stats
    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(8),
        ])
        .split(top_layout[0]);

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

    frame.render_stateful_widget(list, sidebar_layout[0], &mut app.pattern_state.clone());

    // Stats panel
    let stats_text = format!(
        "Polygons: {}\nLines: {}\nGen: {:.1}ms\nZoom: {:.0}%",
        app.polygons.len(),
        app.lines.len(),
        app.gen_time_ms,
        app.zoom * 100.0
    );
    let stats = Paragraph::new(stats_text)
        .block(Block::default()
            .title(" Stats ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)))
        .style(Style::default().fg(Color::White));

    frame.render_widget(stats, sidebar_layout[1]);

    // Spinner animation frames
    let spinner_chars = ['|', '/', '-', '\\', '|', '/', '-', '\\'];
    let spinner = spinner_chars[app.spinner_frame % spinner_chars.len()];

    let image_title = if app.is_loading {
        format!(" [{}] Generating... ", spinner)
    } else {
        format!(" {} ", app.svg_path)
    };

    let border_color = if app.is_loading { Color::Yellow } else { Color::Green };

    // Create inner area for image (accounting for border)
    let image_block = Block::default()
        .title(image_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner_area = image_block.inner(top_layout[1]);
    frame.render_widget(image_block, top_layout[1]);

    // Render the image using ratatui-image
    if let Some(ref mut image_state) = app.image_state {
        let image_widget = StatefulImage::new(None);
        frame.render_stateful_widget(image_widget, inner_area, image_state);
    }

    // Settings panel
    let settings_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ])
        .split(main_layout[1]);

    let spacing_style = if app.setting_focus == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let angle_style = if app.setting_focus == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let spacing_block = Block::default()
        .title(" Spacing ")
        .borders(Borders::ALL)
        .border_style(spacing_style);

    let spacing_text = Paragraph::new(format!("{:.1}", app.spacing))
        .style(spacing_style)
        .alignment(Alignment::Center)
        .block(spacing_block);

    frame.render_widget(spacing_text, settings_layout[0]);

    let angle_block = Block::default()
        .title(" Angle ")
        .borders(Borders::ALL)
        .border_style(angle_style);

    let angle_text = Paragraph::new(format!("{:.0}°", app.angle))
        .style(angle_style)
        .alignment(Alignment::Center)
        .block(angle_block);

    frame.render_widget(angle_text, settings_layout[1]);

    // Help
    let help = Paragraph::new("↑↓ pattern  ←→ adjust\n+/- zoom  WASD pan\nTab switch  0 reset  q quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, settings_layout[2]);
}

// ============ CLI Commands ============

/// Output format for the fill command.
#[derive(Clone, Copy, PartialEq)]
enum OutputFormat {
    Svg,
    Json,
}

/// A line in JSON output format.
#[derive(Serialize)]
struct JsonLine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

/// A shape with its lines in JSON output (per-polygon mode).
#[derive(Serialize)]
struct JsonShape {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    index: usize,
    lines: Vec<JsonLine>,
}

/// JSON output with all lines (flat mode).
#[derive(Serialize)]
struct JsonOutputFlat {
    lines: Vec<JsonLine>,
}

/// JSON output with per-shape grouping.
#[derive(Serialize)]
struct JsonOutputGrouped {
    shapes: Vec<JsonShape>,
}

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
    eprintln!();
    eprintln!("Harness options:");
    eprintln!("  -p, --patterns   Comma-separated list of patterns (default: all)");
    eprintln!("  -s, --spacing    Spacing value (default: 2.5)");
    eprintln!("  -a, --angle      Angle value (default: 45)");
    eprintln!("  --json           Output results as JSON");
    eprintln!();
    eprintln!("Stdin support:");
    eprintln!("  Use '-' as input file to read SVG from stdin:");
    eprintln!("  echo '<svg>...</svg>' | {} fill - -p lines -o -", prog);
    eprintln!();
    eprintln!("TUI Controls:");
    eprintln!("  ↑/↓ or j/k    Select pattern");
    eprintln!("  ←/→ or h/l    Adjust setting (fine)");
    eprintln!("  [ / ]         Adjust setting (coarse)");
    eprintln!("  Tab           Switch between spacing/angle");
    eprintln!("  q / Esc       Quit");
}

fn cmd_patterns() {
    println!("Available patterns:");
    for pattern in Pattern::all() {
        println!("  {}", pattern.name());
    }
}

fn cmd_fill(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut pattern_name = "lines";
    let mut spacing = 2.5;
    let mut angle = 45.0;
    let mut format = OutputFormat::Svg;
    let mut grouped = false;
    let mut order_strategy = OrderingStrategy::NearestNeighbor; // Default to optimized

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
            "--order" => {
                i += 1;
                if i < args.len() {
                    order_strategy = OrderingStrategy::from_name(&args[i]).unwrap_or_else(|| {
                        eprintln!("Unknown order strategy: {}. Use 'document' or 'nearest'.", args[i]);
                        std::process::exit(1);
                    });
                }
            }
            "--grouped" => {
                grouped = true;
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
        eprintln!("Error: SVG file required (use '-' for stdin)");
        std::process::exit(1);
    });

    let pattern = Pattern::from_name(pattern_name).unwrap_or_else(|| {
        eprintln!("Unknown pattern: {}. Use 'patterns' command to list available patterns.", pattern_name);
        std::process::exit(1);
    });

    // Read SVG content from file or stdin
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

    eprintln!("Loaded {} polygons", polygons.len());

    // Calculate and display travel optimization
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

    // Generate output based on format and grouping
    let output = match (format, grouped) {
        (OutputFormat::Json, true) => {
            // Per-polygon grouped JSON output (respects ordering)
            let shapes: Vec<JsonShape> = order
                .iter()
                .map(|&idx| {
                    let polygon = &polygons[idx];
                    let lines = generate_pattern(pattern, polygon, spacing, angle);
                    JsonShape {
                        id: polygon.id.clone(),
                        index: idx,
                        lines: lines.iter().map(|l| JsonLine {
                            x1: l.x1,
                            y1: l.y1,
                            x2: l.x2,
                            y2: l.y2,
                        }).collect(),
                    }
                })
                .collect();

            let elapsed = start.elapsed();
            let total_lines: usize = shapes.iter().map(|s| s.lines.len()).sum();
            eprintln!("Generated {} lines in {} shapes in {:?}", total_lines, shapes.len(), elapsed);

            let output = JsonOutputGrouped { shapes };
            serde_json::to_string(&output).expect("Failed to serialize JSON")
        }
        (OutputFormat::Json, false) => {
            // Flat JSON output (respects ordering)
            let mut all_lines: Vec<Line> = Vec::new();
            for &idx in &order {
                let lines = generate_pattern(pattern, &polygons[idx], spacing, angle);
                all_lines.extend(lines);
            }

            let elapsed = start.elapsed();
            eprintln!("Generated {} lines in {:?}", all_lines.len(), elapsed);

            let json_lines: Vec<JsonLine> = all_lines.iter().map(|l| JsonLine {
                x1: l.x1,
                y1: l.y1,
                x2: l.x2,
                y2: l.y2,
            }).collect();

            let output = JsonOutputFlat { lines: json_lines };
            serde_json::to_string(&output).expect("Failed to serialize JSON")
        }
        (OutputFormat::Svg, _) => {
            // SVG output (respects ordering)
            let mut all_lines: Vec<Line> = Vec::new();
            for &idx in &order {
                let lines = generate_pattern(pattern, &polygons[idx], spacing, angle);
                all_lines.extend(lines);
            }

            let elapsed = start.elapsed();
            eprintln!("Generated {} lines in {:?}", all_lines.len(), elapsed);

            lines_to_svg(&all_lines, &svg_content)
        }
    };

    // Write output
    match output_path {
        Some("-") | None => {
            // stdout
            println!("{}", output);
        }
        Some(path) => {
            fs::write(path, &output).expect("Failed to write output file");
            eprintln!("Wrote: {}", path);
        }
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

/// Result from running a single pattern in the harness
#[derive(Debug)]
struct HarnessResult {
    pattern: String,
    lines: usize,
    time_ms: f64,
    status: &'static str,
}

fn cmd_harness(args: &[String]) {
    let mut svg_path: Option<&str> = None;
    let mut pattern_filter: Option<Vec<&str>> = None;
    let mut spacing = 2.5;
    let mut angle = 45.0;
    let mut json_output = false;

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

    let mut results: Vec<HarnessResult> = Vec::new();
    let mut passed = 0;
    let mut failed = 0;

    for pattern in &patterns_to_run {
        if !json_output {
            eprint!("  {:12} ... ", pattern.name());
        }

        let start = Instant::now();
        let mut total_lines = 0;
        let mut error: Option<String> = None;

        // Use catch_unwind to handle panics
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut lines = 0;
            for polygon in &polygons {
                let poly_lines = generate_pattern(*pattern, polygon, spacing, angle);
                lines += poly_lines.len();
            }
            lines
        }));

        let elapsed = start.elapsed();
        let time_ms = elapsed.as_secs_f64() * 1000.0;

        let (status, lines) = match result {
            Ok(lines) => {
                total_lines = lines;
                passed += 1;
                ("OK", lines)
            }
            Err(e) => {
                failed += 1;
                error = Some(format!("{:?}", e));
                ("FAIL", 0)
            }
        };

        results.push(HarnessResult {
            pattern: pattern.name().to_string(),
            lines,
            time_ms,
            status,
        });

        if !json_output {
            if status == "OK" {
                eprintln!("{:>8} lines in {:>8.1}ms", total_lines, time_ms);
            } else {
                eprintln!("FAILED: {:?}", error);
            }
        }
    }

    // Output results
    if json_output {
        // JSON output for agents
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
    } else {
        // Summary table
        eprintln!("\n════════════════════════════════════════════════════════════");
        eprintln!("  HARNESS SUMMARY");
        eprintln!("════════════════════════════════════════════════════════════");
        eprintln!("  {:12}  {:>10}  {:>10}  {:>6}", "Pattern", "Lines", "Time(ms)", "Status");
        eprintln!("  {:12}  {:>10}  {:>10}  {:>6}", "-------", "-----", "--------", "------");

        let mut total_lines = 0;
        let mut total_time = 0.0;

        for r in &results {
            let status_str = if r.status == "OK" { "✓" } else { "✗" };
            eprintln!("  {:12}  {:>10}  {:>10.1}  {:>6}", r.pattern, r.lines, r.time_ms, status_str);
            total_lines += r.lines;
            total_time += r.time_ms;
        }

        eprintln!("  {:12}  {:>10}  {:>10}  {:>6}", "-------", "-----", "--------", "------");
        eprintln!("  {:12}  {:>10}  {:>10.1}", "TOTAL", total_lines, total_time);
        eprintln!("════════════════════════════════════════════════════════════");
        eprintln!("  Passed: {}  Failed: {}", passed, failed);
        eprintln!("════════════════════════════════════════════════════════════");

        if failed > 0 {
            std::process::exit(1);
        }
    }
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
        Pattern::Radial => generate_radial_fill(polygon, 10.0, angle),
        Pattern::Honeycomb => generate_honeycomb_fill(polygon, spacing * 4.0, angle),
        Pattern::Crossspiral => generate_crossspiral_fill(polygon, spacing, angle),
        Pattern::Hilbert => generate_hilbert_fill(polygon, spacing, angle),
        Pattern::Guilloche => generate_guilloche_fill(polygon, spacing, angle),
        Pattern::Lissajous => generate_lissajous_fill(polygon, spacing, angle),
        Pattern::Rose => generate_rose_fill(polygon, spacing, angle),
        Pattern::Phyllotaxis => generate_phyllotaxis_fill(polygon, spacing, angle),
        Pattern::Scribble => generate_scribble_fill(polygon, spacing, angle),
        Pattern::Gyroid => generate_gyroid_fill(polygon, spacing, angle),
        Pattern::Pentagon15 => generate_pentagon15_fill(polygon, spacing * 3.0, angle),
        Pattern::Pentagon14 => generate_pentagon14_fill(polygon, spacing * 3.0, angle),
        Pattern::Grid => generate_grid_fill(polygon, spacing, angle),
        Pattern::Brick => generate_brick_fill(polygon, spacing, angle),
        Pattern::Truchet => generate_truchet_fill(polygon, spacing * 2.0, angle),
        Pattern::Stipple => generate_stipple_fill(polygon, spacing, angle),
        Pattern::Peano => generate_peano_fill(polygon, spacing, angle),
        Pattern::Sierpinski => generate_sierpinski_fill(polygon, spacing, angle),
        Pattern::Diagonal => generate_diagonal_fill(polygon, spacing, angle),
        Pattern::Herringbone => generate_herringbone_fill(polygon, spacing * 2.0, angle),
        Pattern::Stripe => generate_stripe_fill(polygon, spacing * 2.0, angle),
    }
}

/// Convert lines to SVG output.
fn lines_to_svg(lines: &[Line], original_svg: &str) -> String {
    let viewbox = extract_viewbox(original_svg).unwrap_or("0 0 1000 1000".to_string());

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">
<g stroke="black" stroke-width="0.5" fill="none">
"#,
        viewbox
    ));

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
    if let Some(start) = svg.find("viewBox=\"") {
        let rest = &svg[start + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    if let Some(start) = svg.find("viewbox=\"") {
        let rest = &svg[start + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    None
}
