# rat-king API Reference

## Core Types (geometry.rs)

```rust
/// A 2D point with x,y coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self;
    pub fn distance(&self, other: Point) -> f64;
}

/// A line segment defined by two endpoints
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Line {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self;
    pub fn start(&self) -> Point;
    pub fn end(&self) -> Point;
    pub fn midpoint(&self) -> Point;
    pub fn length(&self) -> f64;
}

/// A polygon with outer boundary and optional holes
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub outer: Vec<Point>,           // Counter-clockwise vertices
    pub holes: Vec<Vec<Point>>,      // Clockwise holes
    pub id: Option<String>,          // SVG element ID
}

impl Polygon {
    pub fn new(outer: Vec<Point>) -> Self;
    pub fn with_holes(outer: Vec<Point>, holes: Vec<Vec<Point>>) -> Self;
    pub fn with_id(outer: Vec<Point>, id: Option<String>) -> Self;
    pub fn bounding_box(&self) -> Option<(f64, f64, f64, f64)>;  // (min_x, min_y, max_x, max_y)
    pub fn center(&self) -> Option<Point>;
    pub fn diagonal(&self) -> Option<f64>;
    pub fn signed_area(&self) -> f64;    // Positive=CCW, Negative=CW
    pub fn is_clockwise(&self) -> bool;
    pub fn point_in_body<F>(&self, x: f64, y: f64, pip_fn: F) -> bool;
}
```

## Pattern Generation (patterns/mod.rs)

```rust
/// Available pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    Lines, Crosshatch, Zigzag, Wiggle, Spiral, Fermat, Concentric,
    Radial, Honeycomb, Crossspiral, Hilbert, Guilloche, Lissajous,
    Rose, Phyllotaxis, Scribble, Gyroid, Pentagon15, Pentagon14,
    Grid, Brick, Truchet, Stipple, Peano, Sierpinski, Diagonal,
    Herringbone, Stripe, Tessellation, Harmonograph, Flowfield,
    Voronoi, Gosper, Wave, Sunburst,
}

impl Pattern {
    /// Get all available patterns
    pub fn all() -> &'static [Pattern];

    /// Get pattern name as string (e.g., "crosshatch")
    pub fn name(&self) -> &'static str;

    /// Parse pattern from string (case-insensitive, supports aliases)
    /// "crosshatch", "zigzag", "sine"->Wiggle, "dots"->Stipple, etc.
    pub fn from_name(name: &str) -> Option<Pattern>;

    /// Get UI metadata (spacing_label, angle_label, description)
    pub fn metadata(&self) -> PatternMetadata;

    /// Generate pattern fill lines for a polygon
    /// - spacing: Controls density (pattern-specific meaning)
    /// - angle: Rotation in degrees
    /// Returns Vec<Line> clipped to polygon boundary
    pub fn generate(&self, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line>;
}
```

## Individual Pattern Functions (patterns/*.rs)

All pattern functions follow this signature:
```rust
pub fn generate_<pattern>_fill(
    polygon: &Polygon,
    spacing: f64,      // Density/scale parameter
    angle_degrees: f64 // Rotation parameter
) -> Vec<Line>;
```

Examples:
```rust
// crates/rat-king/src/hatch.rs
pub fn generate_lines_fill(polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line>;
pub fn generate_crosshatch_fill(polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line>;

// crates/rat-king/src/patterns/zigzag.rs
pub fn generate_zigzag_fill(polygon: &Polygon, spacing: f64, angle: f64, amplitude: f64) -> Vec<Line>;

// crates/rat-king/src/patterns/spiral.rs
pub fn generate_spiral_fill(polygon: &Polygon, spacing: f64, start_angle: f64) -> Vec<Line>;
pub fn generate_fermat_fill(polygon: &Polygon, spacing: f64, rotation: f64) -> Vec<Line>;

// crates/rat-king/src/patterns/hilbert.rs
pub fn generate_hilbert_fill(polygon: &Polygon, detail: f64, rotation: f64) -> Vec<Line>;

// crates/rat-king/src/patterns/stipple.rs
pub fn generate_stipple_fill(polygon: &Polygon, spacing: f64, randomness: f64) -> Vec<Line>;
```

## Clipping (clip.rs)

```rust
/// Test if point is inside polygon (ray casting algorithm)
pub fn point_in_polygon(px: f64, py: f64, polygon: &[Point]) -> bool;

/// Clip a line segment to a polygon boundary
/// Returns segments that lie inside the polygon
pub fn clip_line_to_polygon(line: Line, polygon: &Polygon) -> Vec<Line>;

/// Clip line to polygon, excluding holes
pub fn clip_line_to_polygon_with_holes(line: Line, polygon: &Polygon) -> Vec<Line>;

/// Batch clip multiple lines to a polygon
pub fn clip_lines_to_polygon(lines: &[Line], polygon: &Polygon) -> Vec<Line>;

/// Line-line intersection result
pub enum Intersection {
    None,
    Point { x: f64, y: f64, t: f64 },  // t = parameter along first line (0..1)
}

pub fn line_segment_intersection(
    x1: f64, y1: f64, x2: f64, y2: f64,  // First line
    x3: f64, y3: f64, x4: f64, y4: f64,  // Second line
) -> Intersection;
```

## SVG Parsing (svg.rs)

```rust
/// Extract polygons from SVG content
/// Handles: <path>, <polygon>, <rect>, <circle>, <ellipse>
/// Applies transforms, handles nested groups
pub fn extract_polygons_from_svg(svg_content: &str) -> Result<Vec<Polygon>, SvgError>;

#[derive(Debug)]
pub enum SvgError {
    ParseError(String),
    InvalidPath(String),
}
```

## Sketchy Effect (sketchy.rs)

```rust
/// Configuration for hand-drawn effect
#[derive(Clone, Debug)]
pub struct SketchyConfig {
    pub roughness: f64,      // Endpoint randomization (0.0=smooth, 1.0+=rough)
    pub bowing: f64,         // Line curvature amount
    pub double_stroke: bool, // Draw each line twice with offset
    pub seed: Option<u64>,   // Reproducible randomness
}

impl Default for SketchyConfig {
    fn default() -> Self;  // roughness=1.0, bowing=1.0, double_stroke=true
}

impl SketchyConfig {
    pub fn with_roughness(self, roughness: f64) -> Self;
    pub fn with_bowing(self, bowing: f64) -> Self;
    pub fn with_double_stroke(self, double_stroke: bool) -> Self;
    pub fn with_seed(self, seed: u64) -> Self;
}

/// Apply sketchy effect to lines
pub fn sketchify_lines(lines: &[Line], config: &SketchyConfig) -> Vec<Line>;

/// Convert polygon edges to line segments
pub fn polygon_to_lines(polygon: &Polygon) -> Vec<Line>;
```

## Line Ordering (order.rs)

```rust
pub enum OrderingStrategy {
    Document,        // Original SVG order
    NearestNeighbor, // TSP-style optimization
}

/// Order polygons to minimize travel distance
pub fn order_polygons(polygons: &[Polygon], strategy: OrderingStrategy) -> Vec<usize>;

/// Order points using nearest neighbor algorithm
pub fn order_nearest_neighbor(points: &[Point]) -> Vec<usize>;

/// Calculate total travel distance for an ordering
pub fn calculate_travel_distance(polygons: &[Polygon], order: &[usize]) -> f64;
```

## Line Chaining (chain.rs)

```rust
/// Configuration for line chaining
pub struct ChainConfig {
    pub tolerance: f64,  // Max gap to bridge
}

/// Statistics about chaining results
pub struct ChainStats {
    pub input_lines: usize,
    pub output_chains: usize,
    pub total_length: f64,
}

/// A chain of connected line segments
pub struct Chain {
    pub points: Vec<Point>,
}

/// Chain disconnected line segments into continuous paths
pub fn chain_lines(lines: &[Line], config: &ChainConfig) -> (Vec<Chain>, ChainStats);
```

## Re-exports from lib.rs

The library re-exports common types at the crate root:
```rust
pub use geometry::{Line, Point, Polygon};
pub use patterns::Pattern;
pub use clip::{clip_line_to_polygon, clip_lines_to_polygon, point_in_polygon};
pub use svg::{extract_polygons_from_svg, SvgError};
pub use sketchy::{SketchyConfig, sketchify_lines, polygon_to_lines};
pub use order::{order_polygons, OrderingStrategy};
pub use chain::{chain_lines, Chain, ChainConfig, ChainStats};
pub use hatch::{generate_lines_fill, generate_crosshatch_fill};
```
