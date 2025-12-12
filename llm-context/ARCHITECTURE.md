# rat-king Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Interface                               │
├─────────────────┬───────────────────────────────────────────────────┤
│   CLI Commands  │                    TUI                            │
│  (fill, bench)  │           (interactive preview)                   │
└────────┬────────┴───────────────────────────────────────────────────┘
         │                           │
         ▼                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      rat-king-cli crate                             │
│  • main.rs - TUI app with ratatui                                   │
│  • cli/fill.rs - fill command implementation                        │
│  • cli/benchmark.rs - performance testing                           │
│  • cli/analyze/ - SVG analysis for AI agents                        │
└────────┬────────────────────────────────────────────────────────────┘
         │ uses
         ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      rat-king crate (library)                       │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │   svg.rs    │  │ geometry.rs │  │  clip.rs    │                  │
│  │ Parse SVG   │  │ Point,Line  │  │ Clip lines  │                  │
│  │ to polygons │  │ Polygon     │  │ to polygon  │                  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                  │
│         │                │                │                         │
│         ▼                ▼                ▼                         │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    patterns/mod.rs                              ││
│  │  Pattern enum + dispatch to 35 pattern generators               ││
│  │                                                                 ││
│  │  patterns/lines.rs     patterns/spiral.rs   patterns/hilbert.rs ││
│  │  patterns/zigzag.rs    patterns/truchet.rs  patterns/stipple.rs ││
│  │  patterns/honeycomb.rs patterns/voronoi.rs  ... (30+ more)      ││
│  └─────────────────────────────────────────────────────────────────┘│
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │ sketchy.rs  │  │  order.rs   │  │  chain.rs   │                  │
│  │ Hand-drawn  │  │ Polygon     │  │ Connect     │                  │
│  │ effect      │  │ ordering    │  │ line segs   │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow

### Pattern Fill Pipeline

```
Input SVG
    │
    ▼
┌───────────────────┐
│ extract_polygons_ │    svg.rs
│ from_svg()        │    Parse <path>, <polygon>, <rect>, etc.
└─────────┬─────────┘    Apply transforms, handle groups
          │
          ▼
    Vec<Polygon>         Each polygon has outer boundary + holes
          │
          ▼
┌───────────────────┐
│ order_polygons()  │    order.rs (optional)
│                   │    Nearest-neighbor TSP to reduce travel
└─────────┬─────────┘
          │
          ▼
    For each polygon:
          │
          ▼
┌───────────────────┐
│ Pattern::generate │    patterns/mod.rs -> patterns/*.rs
│ (polygon,spacing, │    Generate infinite pattern lines
│  angle)           │    Clip to polygon boundary
└─────────┬─────────┘
          │
          ▼
     Vec<Line>           Line segments inside polygon
          │
          ▼
┌───────────────────┐
│ sketchify_lines() │    sketchy.rs (optional)
│                   │    Add hand-drawn wobble
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ chain_lines()     │    chain.rs (optional)
│                   │    Connect segments into polylines
└─────────┬─────────┘
          │
          ▼
    Output SVG/JSON      Lines ready for plotter
```

## Module Responsibilities

### geometry.rs
- Defines `Point`, `Line`, `Polygon` structs
- Basic geometric operations (distance, bounding box, area)
- Foundation for all other modules

### svg.rs
- Parses SVG content using `usvg` library
- Converts SVG shapes to `Polygon` vertices
- Handles transforms, nested groups, various shape types
- Entry point: `extract_polygons_from_svg(svg_string)`

### clip.rs (HOT PATH)
- **Performance critical** - runs millions of times
- `point_in_polygon()` - ray casting algorithm
- `clip_line_to_polygon()` - finds line-polygon intersections
- `clip_lines_to_polygon()` - batch operation for pattern lines

### patterns/mod.rs
- `Pattern` enum with all 35 pattern variants
- `Pattern::generate()` dispatches to specific implementations
- `Pattern::from_name()` parses strings like "crosshatch"
- `PatternMetadata` for UI labels and descriptions

### patterns/*.rs (one file per pattern)
Each pattern file exports a function like:
```rust
pub fn generate_<name>_fill(polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line>
```

Pattern implementation steps:
1. Get bounding box from polygon
2. Generate pattern lines covering bounding box
3. Clip lines to polygon using `clip_lines_to_polygon()`
4. Return clipped lines

### sketchy.rs
- `SketchyConfig` - roughness, bowing, double-stroke settings
- `sketchify_lines()` - adds randomized wobble to line endpoints
- Creates hand-drawn aesthetic (inspired by RoughJS)

### order.rs
- `OrderingStrategy::NearestNeighbor` - TSP-style optimization
- Reduces plotter pen travel distance
- `order_polygons()` returns indices for optimal ordering

### chain.rs
- Connects disconnected line segments into continuous polylines
- Reduces pen lifts on plotter
- `chain_lines()` returns `Vec<Chain>` (chains of connected points)

## Adding a New Pattern

1. **Create pattern file**: `crates/rat-king/src/patterns/mypattern.rs`

```rust
use crate::geometry::{Line, Polygon};
use crate::clip::clip_lines_to_polygon;

pub fn generate_mypattern_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line> {
    let Some((min_x, min_y, max_x, max_y)) = polygon.bounding_box() else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    // Generate pattern lines covering bounding box...
    // ...

    // Clip to polygon
    clip_lines_to_polygon(&lines, polygon)
}
```

2. **Register in mod.rs**: `crates/rat-king/src/patterns/mod.rs`

```rust
mod mypattern;
pub use mypattern::generate_mypattern_fill;

// Add to Pattern enum:
pub enum Pattern {
    // ...existing...
    Mypattern,
}

// Add to Pattern::all(), name(), from_name(), metadata(), generate()
```

3. **Test**: `cargo test` and `cargo run --release` (TUI)

## Performance Considerations

1. **Clipping is the bottleneck**: `clip.rs` functions are called millions of times
2. **Bounding box rejection**: Skip polygons that can't possibly intersect
3. **Pre-allocate vectors**: Use `Vec::with_capacity()` when size is known
4. **Avoid allocations in loops**: Reuse buffers where possible
5. **Parallel processing**: Each polygon can be processed independently

## Crate Dependencies

**rat-king (library)**:
- No heavy dependencies, pure Rust geometry

**rat-king-cli (binary)**:
- `ratatui` - TUI framework
- `crossterm` - Terminal handling
- `resvg`/`usvg` - SVG parsing and rendering
- `tiny_skia` - 2D rendering
- `image` - Image encoding
- `serde`/`serde_json` - JSON output
