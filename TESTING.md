# rat-king Testing Guide

## Quick Start

### Build from Source

```bash
git clone https://github.com/dirtybirdnj/rat-king.git
cd rat-king/crates
cargo build --release
```

Binary location: `./target/release/rat-king`

### Install via Cargo (once published)

```bash
cargo install rat-king-cli
```

---

## CLI Commands

```bash
# List all patterns (currently 30)
rat-king patterns

# Show help
rat-king help

# Launch TUI (interactive pattern preview)
rat-king [svg_file]

# Generate pattern fill
rat-king fill <input.svg> -p <pattern> [options] -o <output.svg>

# Benchmark pattern generation
rat-king benchmark <input.svg> -p <pattern>
```

---

## Testing Workflow

### 1. Test Curve Flattening (Dec 8, 2024)

The curve flattening fix ensures Bézier curves are properly converted to polylines.

```bash
# Create a test SVG with curves
cat > /tmp/circle_test.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40"/>
</svg>
EOF

# Fill it - should produce smooth circular boundary
rat-king fill /tmp/circle_test.svg -p lines -s 3 -o /tmp/circle_filled.svg

# Check the output - open in browser or Inkscape
open /tmp/circle_filled.svg
```

**Expected:** Lines should follow the circular boundary smoothly, not cut across as straight segments.

**Before fix:** Circle reduced to 4-5 points (just Bézier endpoints).
**After fix:** Circle has 50+ points from proper curve flattening.

### 2. Test Pentagon15 Pattern (Dec 8, 2024)

```bash
# Generate pentagon15 tiling
rat-king fill test_assets/simple.svg -p pentagon15 -s 5 -o /tmp/pentagon_test.svg

# View result
open /tmp/pentagon_test.svg

# Try different spacings
rat-king fill test_assets/essex.svg -p pentagon15 -s 10 -o /tmp/pentagon_large.svg
rat-king fill test_assets/essex.svg -p pentagon15 -s 3 -o /tmp/pentagon_small.svg
```

**Expected:** Tiling with irregular pentagons (fixed angles: 135°, 60°, 150°, 90°, 105°).

### 3. Test Polygon Ordering (Dec 8, 2024)

```bash
# Default: nearest-neighbor ordering (optimized)
rat-king fill test_assets/essex.svg -p lines -o /dev/null
# Output shows: "Travel optimization: X -> Y (Z% reduction)"

# Compare with document order
rat-king fill test_assets/essex.svg -p lines --order document -o /dev/null
rat-king fill test_assets/essex.svg -p lines --order nearest -o /dev/null
```

**Expected:** Nearest-neighbor should show 20-50% travel reduction on most multi-polygon files.

### 4. Test All Patterns

```bash
# Quick visual test of all patterns
for pattern in lines crosshatch zigzag wiggle spiral fermat concentric radial honeycomb crossspiral hilbert guilloche lissajous rose phyllotaxis scribble gyroid pentagon15 pentagon14 grid brick truchet stipple peano sierpinski diagonal herringbone stripe tessellation harmonograph; do
  echo "Testing: $pattern"
  rat-king fill test_assets/simple.svg -p $pattern -s 5 -o /tmp/test_$pattern.svg
done

# Open all results
open /tmp/test_*.svg
```

### 5. TUI Interactive Testing

```bash
# Launch TUI with test file
rat-king test_assets/essex.svg

# Controls:
#   ↑/↓ or j/k    - Select pattern
#   ←/→ or h/l    - Adjust spacing/angle (fine)
#   [ / ]         - Adjust spacing/angle (coarse)
#   Tab           - Switch between spacing/angle
#   +/-           - Zoom in/out
#   WASD          - Pan view
#   0 or r        - Reset view
#   q / Esc       - Quit
```

---

## Test Assets

| File | Description | Polygons |
|------|-------------|----------|
| `test_assets/simple.svg` | Basic shapes for quick testing | 3 |
| `test_assets/essex.svg` | USGS county boundaries | 314 |
| `test_assets/tiger.svg` | Complex vector artwork | ~500+ |

---

## Benchmarking

```bash
# Benchmark a specific pattern
rat-king benchmark test_assets/essex.svg -p lines
rat-king benchmark test_assets/essex.svg -p pentagon15
rat-king benchmark test_assets/essex.svg -p gyroid

# Expected output:
# ═══════════════════════════════════════════════
#   RUST BENCHMARK: LINES
# ═══════════════════════════════════════════════
#   Polygons: 314
#   Lines generated: 11521
#   Time: 159.23ms
#   Avg per polygon: 0.507ms
# ═══════════════════════════════════════════════
```

---

## Output Formats

### SVG (default)

```bash
rat-king fill input.svg -p lines -o output.svg
```

### JSON (flat)

```bash
rat-king fill input.svg -p lines -f json -o output.json
```

Output:
```json
{"lines":[{"x1":0.0,"y1":0.0,"x2":10.0,"y2":10.0},...]}
```

### JSON (grouped by polygon)

```bash
rat-king fill input.svg -p lines -f json --grouped -o output.json
```

Output:
```json
{"shapes":[{"id":"polygon1","index":0,"lines":[...]},...]}
```

### Stdin/Stdout

```bash
cat input.svg | rat-king fill - -p lines -o - > output.svg
```

---

## Common Issues

### 1. "No polygons found in SVG"

- SVG may contain only strokes, no filled paths
- Try converting strokes to paths in Inkscape

### 2. Pattern doesn't fill curved shapes properly

- **Fixed in Dec 8, 2024 release**
- If still seeing issues, ensure you have the latest build

### 3. Slow performance on large files

- Use release build: `cargo build --release`
- Reduce polygon count or simplify SVG
- Some patterns (gyroid, hilbert) are slower by design

### 4. TUI not displaying images

- Requires Sixel-compatible terminal (iTerm2, WezTerm, mlterm)
- Falls back to basic display on unsupported terminals

---

## Regression Tests

Run the full test suite:

```bash
cd crates
cargo test
```

Expected: 119+ unit tests + 8 integration tests passing (as of Dec 9, 2024)

Key test modules:
- `svg::tests` - SVG parsing and curve flattening
- `order::tests` - Polygon ordering algorithms
- `patterns::*::tests` - All 30 pattern generators
- `integration.rs` - CLI end-to-end tests

---

## Version History

### Dec 9, 2024
- Added `harmonograph` pattern (decaying pendulum curves)
- Added 8 CLI integration tests
- Refactored patterns to use `PatternContext` utilities
- Added `Pattern::generate()` method for centralized dispatch
- Total: 30 patterns, 119 unit tests + 8 integration tests

### Dec 8, 2024
- Added `lyon_geom` for proper Bézier curve flattening
- Added `pentagon15` pattern (15th pentagonal tiling)
- Added nearest-neighbor polygon ordering (`--order` flag)
- Added 12 new patterns: pentagon14, grid, brick, truchet, stipple, peano, sierpinski, diagonal, herringbone, stripe, tessellation
- Travel optimization shows % reduction in stderr

### Previous
- Initial 17 patterns implemented
- Sixel graphics in TUI
- JSON output modes
- Stdin support
