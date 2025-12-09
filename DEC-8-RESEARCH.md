# December 8, 2024 - Research Notes

## Problem Statement

Some patterns are "passing" tests but not properly filling geometries. The core issue is likely:

1. **Curve flattening** - Bézier curves approximated with only endpoints (losing curve detail)
2. **Complex polygons** - Self-intersecting or non-simple polygons breaking clipping
3. **Holes handling** - TODO in `svg.rs:116` - multiple subpaths not handled
4. **Polygon ordering** - No optimization for plotter travel distance

---

## Priority 1: svg2polylines

**Repository:** https://github.com/dbrgn/svg2polylines
**Docs:** https://docs.rs/svg2polylines/latest/svg2polylines/
**License:** MIT / Apache-2.0

### What It Does

Converts SVG files to polylines (polygonal chains). Already uses:
- **usvg** for preprocessing (same as rat-king!)
- **Lyon** for curve flattening (the key difference)

### Key Insight

Current rat-king `svg.rs` does this for curves:

```rust
usvg::tiny_skia_path::PathSegment::CubicTo(_, _, p) => {
    // Approximate curve with endpoint  <-- PROBLEM!
    points.push(Point::new(p.x as f64, p.y as f64));
}
```

This loses ALL curve information - a complex Bézier becomes a straight line.

svg2polylines properly flattens curves with a tolerance parameter:

```rust
fn parse(svg: &str, tol: f64, preprocess: bool) -> Result<Vec<Polyline>, String>
```

### API

```rust
use svg2polylines::{parse, CoordinatePair, Polyline};

let svg_data = std::fs::read_to_string("input.svg")?;
let polylines: Vec<Polyline> = parse(&svg_data, 0.1, true)?;

for polyline in polylines {
    for point in polyline {
        println!("({}, {})", point.x, point.y);
    }
}
```

### Integration Options

**Option A: Use as dependency**
```toml
[dependencies]
svg2polylines = "0.8"
```

Replace `extract_polygons_from_svg()` with svg2polylines parsing, then convert polylines to our Polygon type.

**Option B: Port the curve flattening logic**

svg2polylines uses Lyon's `lyon_geom` for curve flattening. We could:
1. Add `lyon_geom` dependency
2. Use `CubicBezierSegment::for_each_flattened()` in our path parsing

### Tolerance Parameter

The `tol` parameter controls curve approximation quality:
- Lower = more points, smoother curves, slower
- Higher = fewer points, rougher curves, faster
- `0.1` is a good default for plotters (sub-pixel accuracy)

---

## Priority 2: Lyon Tessellation

**Repository:** https://github.com/nical/lyon
**Docs:** https://docs.rs/lyon_tessellation/latest/lyon_tessellation/
**License:** MIT / Apache-2.0

### What It Does

Path tessellation library - converts vector paths to triangle meshes. The key component for us:

**FillTessellator** - Handles:
- Complex non-convex polygons
- Self-intersecting paths
- Polygons with holes
- Proper winding rule (even-odd vs non-zero)

### Why This Matters

Current `clip.rs` uses ray casting point-in-polygon which can fail on:
- Self-intersecting polygons (figure-8 shapes)
- Very thin slivers
- Degenerate cases

Lyon's tessellator converts ANY path to simple triangles. We could:
1. Tessellate complex polygons into simple triangles
2. Generate hatching for each triangle (guaranteed convex!)
3. Or use tessellation to "fix" non-simple polygons before hatching

### Key Components

```rust
use lyon_tessellation::{
    FillTessellator,
    FillOptions,
    geometry_builder::simple_builder,
    path::Path,
};

// Build a path
let mut builder = Path::builder();
builder.begin(point(0.0, 0.0));
builder.line_to(point(100.0, 0.0));
builder.line_to(point(100.0, 100.0));
builder.line_to(point(0.0, 100.0));
builder.close();
let path = builder.build();

// Tessellate
let mut tessellator = FillTessellator::new();
let mut geometry = VertexBuffers::new();

tessellator.tessellate_path(
    &path,
    &FillOptions::default(),
    &mut simple_builder(&mut geometry),
)?;

// geometry.vertices = triangle vertices
// geometry.indices = triangle indices
```

### Curve Flattening with Lyon

Lyon also handles Bézier curves:

```rust
use lyon_geom::{CubicBezierSegment, point};

let curve = CubicBezierSegment {
    from: point(0.0, 0.0),
    ctrl1: point(10.0, 50.0),
    ctrl2: point(90.0, 50.0),
    to: point(100.0, 0.0),
};

// Flatten to line segments with tolerance 0.1
curve.for_each_flattened(0.1, &mut |point| {
    println!("({}, {})", point.x, point.y);
});
```

### Integration Strategy

**Phase 1: Fix curve flattening (simpler)**
```toml
[dependencies]
lyon_geom = "1.0"
```

Update `path_to_polygon()` to use `for_each_flattened()` for Bézier curves.

**Phase 2: Handle complex polygons (if needed)**
```toml
[dependencies]
lyon_tessellation = "1.0"
```

Pre-tessellate non-simple polygons before hatching.

---

## Priority 3: Polygon Ordering (Travel Optimization)

### Problem

Current code processes polygons in SVG document order:

```rust
for polygon in &polygons {
    let lines = generate_pattern(pattern, polygon, spacing, angle);
    all_lines.extend(lines);
}
```

This can result in excessive plotter travel between distant polygons.

### Solution: Nearest Neighbor Ordering

```rust
fn order_polygons_nearest_neighbor(polygons: &[Polygon]) -> Vec<usize> {
    let mut order = Vec::with_capacity(polygons.len());
    let mut remaining: HashSet<usize> = (0..polygons.len()).collect();

    // Start from first polygon
    let mut current = 0;
    order.push(current);
    remaining.remove(&current);

    while !remaining.is_empty() {
        let current_center = polygons[current].centroid();

        // Find nearest remaining polygon
        let nearest = remaining.iter()
            .min_by(|&&a, &&b| {
                let dist_a = current_center.distance_to(&polygons[a].centroid());
                let dist_b = current_center.distance_to(&polygons[b].centroid());
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .copied()
            .unwrap();

        order.push(nearest);
        remaining.remove(&nearest);
        current = nearest;
    }

    order
}
```

### Advanced: TSP Solver

For better optimization, consider:
- 2-opt local search
- Christofides algorithm
- Or use a crate like `travelling_salesman`

---

## Priority 4: 15 Monohedral Pentagonal Tilings

### Background

In 2017, Michaël Rao proved there are exactly 15 types of convex pentagons that can tile the plane. This is a complete classification - no more exist.

### Discovery History

| Type | Year | Discoverer |
|------|------|------------|
| 1-5 | 1918 | Karl Reinhardt |
| 6-8 | 1968 | R. B. Kershner |
| 9 | 1975 | Richard James |
| 10-13 | 1976-77 | Marjorie Rice (amateur!) |
| 14 | 1985 | Rolf Stein |
| 15 | 2015 | Mann, McLoud-Mann, Von Derau |

### Mathematical Specifications

Pentagons have vertices A, B, C, D, E with opposite sides a, b, c, d, e.

**Type 1:** `A + B + C = 360°`
- Most general, infinite family

**Type 2:** `A + B + D = 360°`, `a = d`

**Type 3:** `A = C = D = 120°`, `a = b`, `d = c + e`

**Type 4:** `A = C = 90°`, `a = b`, `c = d`

**Type 5:** `A = 60°`, `D = 120°`, `a = b`, `c = d`

**Type 6:** `A + B + D = 360°`, `A = 2C`, `a = b = e`, `c = d`

**Type 7:** `2B + C = 360°`, `2D + A = 360°`, `a = b = c = d`

**Type 8:** `2A + B = 360°`, `2D + C = 360°`, `a = b = c = d`

**Type 9:** `2E + B = 360°`, `2D + C = 360°`, `a = b = c = d`

**Type 10:** `A = 90°`, `B + E = 180°`, `B + 2C = 360°`, `a = b = c + e`

**Type 11:** `A = 90°`, `C + E = 180°`, `2B + C = 360°`, `d = e = 2a + c`

**Type 12:** `A = 90°`, `C + E = 180°`, `2B + C = 360°`, `a = b`, `d = 2c + e`

**Type 13:** `A = C = 90°`, `B = 150°`, `a = b`, `d = 2c`, `e = d + 2a`

**Type 14:** (Fixed shape - no free parameters)
- `A = 90°`, `B ≈ 145.34°`, `C ≈ 69.32°`, `D ≈ 124.66°`, `E ≈ 110.68°`
- `C = arccos((3√57 - 17)/16)`

**Type 15:** (Fixed shape)
- `A = 135°`, `B = 60°`, `C = 150°`, `D = 90°`, `E = 105°`
- Discovered by computer search in 2015

### Pattern Implementation Ideas

```rust
/// Generate a Type 1 pentagonal tiling
fn generate_pentagon_type1(polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
    // Type 1: A + B + C = 360°
    // This allows vertex A, B, C to meet at a point

    let bounds = polygon.bounding_box().unwrap();
    let (min_x, min_y, max_x, max_y) = bounds;

    // Pentagon dimensions based on spacing
    let side = spacing * 2.0;

    // Generate pentagon grid...
}
```

### Relevance to rat-king

These could be new fill patterns:
- `pentagon1` through `pentagon15`
- Each creates a unique tiling aesthetic
- Types 14 and 15 are fixed shapes (easier to implement)
- Others have parameters (more flexible but complex)

---

## CLI/TUI Improvements (from procs & dua analysis)

### Patterns from procs

1. **Automatic theme detection** - Detect terminal background for color scheme
2. **Configurable columns** - Let users customize output fields
3. **Watch mode** - `--watch` for live updates (useful for benchmarks)
4. **Tree view** - Show polygon hierarchy from SVG groups
5. **Search/filter** - `--filter "area>100"` for polygon selection
6. **TOML configuration** - Persist user preferences

### Patterns from dua

1. **Multi-stage deletion safety** - Confirm before destructive operations
2. **Interactive help** - `?` key shows shortcuts
3. **Progressive disclosure** - Simple default, advanced via flags
4. **Performance notes** - Show memory usage for large files

### Proposed rat-king TUI Enhancements

```
┌─ Patterns ──────┐ ┌─ Preview ─────────────────────────────┐
│ ► lines         │ │                                       │
│   crosshatch    │ │     [High-res Sixel preview]         │
│   zigzag        │ │                                       │
│   wiggle        │ │                                       │
│   spiral        │ │                                       │
│   pentagon1  NEW│ │                                       │
│   pentagon15 NEW│ │                                       │
└─────────────────┘ └───────────────────────────────────────┘
┌─ Stats ─────────────────────────────────────────────────────┐
│ Polygons: 314 | Lines: 45,231 | Time: 159ms | Travel: 2.3m │
└─────────────────────────────────────────────────────────────┘
┌─ Settings ──────────────────────────────────────────────────┐
│ Spacing: [====2.5====]  Angle: [===45°===]  Order: nearest │
└─────────────────────────────────────────────────────────────┘
```

New features:
- **Travel distance** - Show estimated plotter travel
- **Order mode** - Toggle between document/nearest/optimized
- **Polygon stats** - Count, total area, etc.

---

## Implementation Priority

1. **svg2polylines/lyon_geom** - Fix curve flattening (biggest impact)
2. **Polygon ordering** - Add nearest-neighbor optimization
3. **Pentagon patterns** - Start with Type 15 (fixed shape, easiest)
4. **TUI improvements** - Travel stats, ordering toggle

---

## References

- [svg2polylines](https://github.com/dbrgn/svg2polylines)
- [Lyon](https://github.com/nical/lyon) - [Intro blog post](https://nical.github.io/posts/lyon-intro.html)
- [Pentagon Tiling - Quanta Magazine](https://www.quantamagazine.org/pentagon-tiling-proof-solves-century-old-math-problem-20170711/)
- [Rao's Proof (2017)](https://perso.ens-lyon.fr/michael.rao/publi/penta.pdf)
- [procs](https://github.com/dalance/procs) - CLI design patterns
- [dua-cli](https://github.com/Byron/dua-cli) - TUI design patterns
