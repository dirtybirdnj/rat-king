# rat-king Codebase Overview

## What is rat-king?

rat-king is a **blazing-fast fill pattern generator for pen plotters**, written in Rust. It takes SVG files with closed shapes and fills them with hatching patterns (lines, crosshatch, spirals, etc.) suitable for plotter output.

**Key value proposition**: 200x faster than Python/Shapely implementations (~159ms vs ~32,000ms for complex SVGs).

## Directory Structure

```
rat-king/
├── crates/                    # Rust workspace
│   ├── Cargo.toml             # Workspace config (version 0.1.0)
│   ├── rat-king/              # Core library crate (the dependency)
│   │   └── src/
│   │       ├── lib.rs         # Public API exports
│   │       ├── geometry.rs    # Point, Line, Polygon types
│   │       ├── clip.rs        # Point-in-polygon, line clipping
│   │       ├── hatch.rs       # Basic line generation
│   │       ├── svg.rs         # SVG parsing (extract polygons)
│   │       ├── sketchy.rs     # Hand-drawn effect (RoughJS-style)
│   │       ├── chain.rs       # Line chaining optimization
│   │       ├── order.rs       # Polygon ordering (nearest neighbor)
│   │       ├── rng.rs         # Deterministic RNG
│   │       └── patterns/      # 35 pattern implementations
│   │           ├── mod.rs     # Pattern enum, dispatch
│   │           ├── lines.rs   # Parallel lines (in hatch.rs)
│   │           ├── crosshatch.rs
│   │           ├── zigzag.rs
│   │           ├── wiggle.rs
│   │           ├── spiral.rs
│   │           ├── concentric.rs
│   │           ├── honeycomb.rs
│   │           ├── hilbert.rs
│   │           ├── truchet.rs
│   │           ├── stipple.rs
│   │           └── ... (30+ more)
│   └── rat-king-cli/          # CLI/TUI binary crate
│       └── src/
│           ├── main.rs        # TUI application, CLI dispatch
│           └── cli/           # CLI subcommands
│               ├── mod.rs     # Command exports
│               ├── fill.rs    # `rat-king fill` command
│               ├── benchmark.rs
│               ├── analyze/   # SVG analysis for AI agents
│               └── ...
├── docs/                      # Documentation and assets
│   └── all_patterns_color.png # Pattern reference image
├── test_assets/               # Test SVG files
│   └── essex.svg              # 314 polygons (USGS county data)
└── llm-context/               # This folder
```

## Available Patterns (35 total)

| Pattern | Description | Coverage |
|---------|-------------|----------|
| `lines` | Parallel line hatching | 99% |
| `crosshatch` | Two perpendicular line sets | 99% |
| `zigzag` | Connected angular waves | 99% |
| `wiggle` | Smooth sinusoidal waves | 99% |
| `spiral` | Archimedean spiral from center | 99% |
| `fermat` | Fermat (sqrt radius) spiral | 77% |
| `concentric` | Nested polygon outlines | N/A |
| `radial` | Lines radiating from center | 44% |
| `honeycomb` | Hexagonal grid | 90% |
| `hilbert` | Space-filling curve | 91% |
| `truchet` | Quarter-circle tiles | 85% |
| `stipple` | Poisson disk stippling | 98% |
| `grid` | Orthogonal crosshatch | 99% |
| `brick` | Offset rectangular tiling | 99% |
| `diagonal` | 45-degree parallel lines | 99% |
| `peano` | Peano space-filling curve | 99% |
| `guilloche` | Spirograph hypotrochoid | 15% |
| `lissajous` | Oscilloscope curves | 63% |
| `rose` | Flower petal curves | 33% |
| `phyllotaxis` | Golden angle sunflower | 65% |
| `scribble` | Organic random walk | N/A |
| `gyroid` | 3D minimal surface | N/A |
| `flowfield` | Perlin noise flow lines | varies |
| `voronoi` | Voronoi cell boundaries | varies |
| `harmonograph` | Pendulum simulation | 75% |
| `wave` | Wave interference | varies |
| `sunburst` | Radial rays | varies |
| ... | (plus more) | |

## CLI Usage

```bash
# TUI (interactive mode)
rat-king                      # Opens with default test file
rat-king myfile.svg           # Opens with your SVG

# Fill command
rat-king fill input.svg -p crosshatch -o output.svg
rat-king fill input.svg -p lines --sketchy -o output.svg
rat-king fill input.svg -p lines -s 2.5 -a 45 --strokes -o output.svg

# Other commands
rat-king benchmark input.svg -p gyroid
rat-king patterns             # List all patterns
rat-king analyze input.svg --json
rat-king swatches -o swatches.svg
```

## Library Usage (Rust)

```rust
use rat_king::{Pattern, extract_polygons_from_svg, Polygon, Line};

// Parse SVG
let svg = std::fs::read_to_string("input.svg")?;
let polygons = extract_polygons_from_svg(&svg)?;

// Generate pattern for each polygon
for poly in &polygons {
    let lines: Vec<Line> = Pattern::Lines.generate(poly, 2.5, 45.0);
    // lines contains clipped line segments
}
```

## Key Concepts

1. **Polygon**: Outer boundary + optional holes. SVG paths converted to vertex lists.
2. **Pattern**: Generates infinite lines, clipped to polygon boundary.
3. **Clipping**: Ray-casting for point-in-polygon, segment intersection for line clipping.
4. **Sketchy**: Optional post-process to add hand-drawn wobble to lines.
