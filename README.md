# rat-king 🐀👑

**Blazing-fast fill pattern generation for pen plotters.** Written in Rust for maximum performance.

## Performance

rat-king processes complex SVGs in milliseconds:

```
Essex.svg (314 polygons)
├── Python/Shapely:  ~32,000ms (32 seconds)
└── Rust rat-king:      ~159ms (0.16 seconds)

Speedup: 200x faster
```

## Installation

### Binary (Recommended)

```bash
# The Rust CLI is pre-built in the crates directory
./crates/target/release/rat-king-cli --help

# Or build from source
cd crates && cargo build --release
```

### Python Wrapper (for vpype integration)

```bash
pip install -e .

# With vpype plugin
pip install -e ".[vpype]"
```

## CLI Usage

```bash
# Fill shapes with a pattern
rat-king-cli fill input.svg --pattern lines --spacing 2 -o output.svg

# Benchmark a pattern
rat-king-cli benchmark input.svg -p crosshatch

# List all available patterns
rat-king-cli patterns
```

## Available Patterns (17 total)

All patterns fully implemented - no stubs!

| Pattern | Description | Visual Style |
|---------|-------------|--------------|
| `lines` | Parallel line hatching | Classic crosshatch base |
| `crosshatch` | Two perpendicular line sets | X pattern |
| `zigzag` | Connected angular waves | Lightning bolt |
| `wiggle` | Smooth sinusoidal waves | Wavy lines |
| `spiral` | Archimedean spiral from center | Single arm spiral |
| `fermat` | Fermat (sqrt radius) spiral | Dense center spiral |
| `concentric` | Nested polygon outlines | Shrinking shells |
| `radial` | Lines radiating from center | Sunburst |
| `honeycomb` | Hexagonal grid | Beehive cells |
| `crossspiral` | Two opposing spirals | CW + CCW arms |
| `hilbert` | Space-filling curve | Recursive maze |
| `guilloche` | Spirograph hypotrochoid | Currency-style |
| `lissajous` | Oscilloscope curves | Figure-8s |
| `rose` | Flower petal curves | Rhodonea petals |
| `phyllotaxis` | Golden angle sunflower | Fibonacci spirals |
| `scribble` | Organic random walk | Hand-drawn look |
| `gyroid` | 3D minimal surface | Flowing contours |

## Architecture

```
rat-king/
├── crates/                    # Rust workspace
│   ├── rat-king-core/         # Pattern generation library
│   │   └── src/
│   │       ├── patterns/      # 17 pattern implementations
│   │       ├── geometry.rs    # Point, Line, Polygon types
│   │       ├── clip.rs        # Point-in-polygon testing
│   │       └── hatch.rs       # Line generation utilities
│   └── rat-king-cli/          # CLI binary
│       └── src/main.rs        # fill, benchmark, patterns commands
├── rat_king/                  # Python wrapper (calls Rust binary)
│   ├── cli.py                 # Python CLI facade
│   └── vpype_plugin.py        # vpype integration
└── test_assets/               # Test SVGs
```

## Integration with svg-grouper

[svg-grouper](https://github.com/dirtybirdnj/svg-grouper) calls rat-king's Rust CLI directly:

```python
# svg-grouper invokes the binary
subprocess.run([
    "rat-king-cli", "fill", input_svg,
    "-p", pattern_name,
    "-s", str(spacing),
    "-a", str(angle),
    "-o", output_svg
])
```

## Development

### Building the Rust CLI

```bash
cd crates
cargo build --release
cargo test
```

### Running Benchmarks

```bash
./crates/target/release/rat-king-cli benchmark test_assets/essex.svg -p lines
./crates/target/release/rat-king-cli benchmark test_assets/essex.svg -p gyroid
```

### Adding a New Pattern

1. Create `crates/rat-king-core/src/patterns/mypattern.rs`
2. Implement `generate_mypattern_fill(polygon, spacing, angle) -> Vec<Line>`
3. Add module to `patterns/mod.rs`
4. Add variant to `Pattern` enum
5. Wire up in CLI `generate_pattern()` match
6. Add tests

All patterns follow the same signature:

```rust
pub fn generate_mypattern_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line>
```

## Why Rust?

The original Python implementation using Shapely was clean but slow:
- Complex polygons took 30+ seconds
- Interactive tools were unusable

Rust gives us:
- 200x speedup (milliseconds instead of seconds)
- Zero-copy geometry operations
- Parallel-ready (future work)
- Single binary distribution

## Related Projects

- [vpype](https://github.com/abey79/vpype) - Swiss-army knife for plotter workflows
- [svg-grouper](https://github.com/dirtybirdnj/svg-grouper) - GUI for plotter SVG prep

## License

MIT
