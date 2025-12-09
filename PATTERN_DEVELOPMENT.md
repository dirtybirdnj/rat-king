# Pattern Development Roadmap

Research and implementation notes for rat-king fill patterns.

---

## Current Status (December 2024)

### Implemented Patterns (29 total)

All patterns are fully implemented in Rust. Coverage ratings from test harness (spacing=2.5, angle=45°):

| Pattern | Description | Coverage | Status |
|---------|-------------|----------|--------|
| `lines` | Parallel hatch lines | Excellent (99%) | Done |
| `crosshatch` | Two perpendicular line sets | Excellent (99%) | Done |
| `zigzag` | Connected zigzag lines | Excellent (99%) | Done |
| `wiggle` | Sinusoidal wave lines | Excellent (99%) | Done |
| `spiral` | Archimedean spiral from center | Excellent (99%) | Done |
| `fermat` | Fermat spiral (sqrt radius) | Good (77%) | Done |
| `concentric` | Inward-shrinking polygon shells | N/A* | Done |
| `radial` | Lines radiating from center | Poor (44%) | Done |
| `honeycomb` | Hexagonal grid | Excellent (90%) | Done |
| `crossspiral` | Two opposing spirals | Excellent (95%) | Done |
| `hilbert` | Hilbert space-filling curve | Excellent (91%) | Done |
| `guilloche` | Spirograph hypotrochoid | Poor (15%) | Done |
| `lissajous` | Oscilloscope curves | Fair (63%) | Done |
| `rose` | Flower petal curves | Poor (33%) | Done |
| `phyllotaxis` | Golden angle sunflower | Fair (65%) | Done |
| `scribble` | Random organic walk | N/A* | Done |
| `gyroid` | 3D minimal surface projection | N/A* | Done |
| `pentagon15` | Type 15 pentagonal tiling | Poor (36%) | Done |
| `pentagon14` | Type 14 pentagonal tiling | Poor (42%) | Done |
| `grid` | Orthogonal crosshatch | Excellent (99%) | Done |
| `brick` | Offset rectangular tiling | Excellent (99%) | Done |
| `truchet` | Quarter-circle tiles | Good (85%) | Done |
| `stipple` | Poisson disk stippling | Excellent (98%) | Done |
| `peano` | Peano space-filling curve | Excellent (99%) | Done |
| `sierpinski` | Sierpinski triangle | Poor (24%) | Done |
| `diagonal` | 45° parallel lines | Excellent (99%) | Done |
| `herringbone` | V-shaped brick pattern | Fair (58%) | Done |
| `stripe` | Horizontal parallel lines | Excellent (98%) | Done |
| `tessellation` | Geometric tiled pattern | Fair (52%) | Done |

*N/A: Pattern generates too many elements for automated visual analysis at default spacing

### Additional Features

| Feature | Description | Status |
|---------|-------------|--------|
| `--sketchy` | Hand-drawn effect (RoughJS-style) | Done |
| `--strokes` | Include polygon outlines as geometry | Done |
| `--analyze` | Visual coverage/bounds analysis | Done |
| JSON output | Grouped or flat line export | Done |
| Sixel preview | High-res terminal graphics | Done |

---

## Future Patterns (Research)

### Mathematical Curves

#### Harmonograph
```
x(t) = A1*sin(f1*t + p1)*decay + A2*sin(f2*t + p2)*decay
y(t) = A3*sin(f3*t + p3)*decay + A4*sin(f4*t + p4)*decay

Creates spirograph-like patterns, single continuous line.
```

#### Superformula (Gielis curves)
```
r(θ) = (|cos(m*θ/4)/a|^n2 + |sin(m*θ/4)/b|^n3)^(-1/n1)

Generates: circles, ellipses, stars, organic leaf/petal shapes
```

---

### Tile-Based Systems

#### Wang Tiles
Edge-matching tiles for seamless aperiodic patterns.

```
TILE DEFINITION:
  Each edge has a "color" code
  Adjacent edges must match
  Creates never-repeating textures
```

#### Penrose Tilings
Aperiodic tilings with 5-fold symmetry.

```
TWO RHOMBS:
  - Thin: 36° and 144° angles
  - Fat: 72° and 108° angles

GENERATION: Substitution rules or de Bruijn pentagrids

ARC DECORATION:
  Draw arcs on rhombs that connect at matching edges
  Creates continuous curved paths through aperiodic structure
```

---

### Organic Patterns

#### Voronoi Diagrams
Partition space by nearest seed point.

```
ALGORITHM: Fortune's sweep line O(n log n)

SEED DISTRIBUTIONS:
  - Random uniform (clumpy)
  - Poisson disk (blue noise, more natural)
  - Lloyd relaxation (centroidal, very even)
  - Image-weighted (density follows brightness)

FILL VARIATIONS:
  - Cell outlines only (cracked look)
  - Stipple at centroids
  - Hatch each cell at different angle
```

#### Reaction-Diffusion (Turing Patterns)
Chemical simulation that produces spots, stripes, labyrinths.

```
GRAY-SCOTT MODEL:
  ∂A/∂t = Da∇²A - AB² + f(1-A)
  ∂B/∂t = Db∇²B + AB² - (k+f)B

PARAMETERS:
  f = 0.055, k = 0.062: mitosis (spots splitting)
  f = 0.030, k = 0.057: coral/maze
  f = 0.025, k = 0.055: spots
  f = 0.078, k = 0.061: stripes

VECTORIZATION:
  Simulate on grid, extract contours at threshold values.
```

#### Differential Growth
Curve that grows and wrinkles, filling space organically.

```
ALGORITHM:
  1. Start with simple closed curve
  2. Points repel nearby points
  3. Points attract immediate neighbors (spring force)
  4. Apply growth force (outward)
  5. Subdivide long segments, merge close points
  6. Repeat

RESULT: Coral-like, organic wrinkled patterns
```

#### Phyllotaxis (Sunflower Spirals)
```
GOLDEN ANGLE: 137.507764°

for i in 0..N:
  angle = i * GOLDEN_ANGLE
  radius = sqrt(i) * scale
  place_element(x, y)

Creates visible Fibonacci spiral arms (parastichies).
```

---

### Flow & Field Patterns

#### Vector Field Flow Lines
```
CONCEPT:
  Define vector field F(x,y) = (u, v)
  Trace streamlines following the field
  Evenly-spaced streamlines fill region

FIELD SOURCES:
  - Mathematical: sin(x)*cos(y)
  - Noise-based: Perlin noise derivatives
  - Image-derived: gradient of brightness

EFFECTS: Wood grain, fingerprints, topographic contours
```

#### Contour Lines (Marching Squares)
```
FIELDS TO CONTOUR:
  - Distance from point(s)
  - Perlin noise
  - Mathematical functions
  - Image brightness

Evenly spaced values = evenly spaced lines.
```

---

### Optical Effects

#### Moiré Patterns
```
Two regular patterns overlapped with slight differences.

FORMULA (line grids at angle θ, spacing d):
  Moiré spacing = d / (2 * sin(θ/2))

TYPES:
  - Line grids at angle offset
  - Concentric circles with offset centers
  - Radial lines with rotation offset
```

#### Op-Art Distortion
```
BRIDGET RILEY STYLE:
  Parallel lines with controlled wave distortion
  Width variation creates 3D illusion

VASARELY STYLE:
  Grid of shapes with systematic variation
  Size/position shifts create bulge/warp illusion

IMPLEMENTATION:
  Apply distortion function to base pattern coordinates.
```

#### Chladni Figures
Standing wave patterns on vibrating plates.

```
SQUARE PLATE:
  f(x,y) = cos(m*π*x/L)*cos(n*π*y/L) - cos(n*π*x/L)*cos(m*π*y/L)

Contour the zero-set for nodal lines.
Integers m, n determine pattern complexity.
```

---

### Advanced Concepts

#### Weaving
Interleave two pattern layers with over/under crossings.

```
ALGORITHM:
  1. Find all intersections between Layer A and Layer B
  2. Assign over/under in checkerboard pattern
  3. Split lines at intersections
  4. Reorder segments by z-order
```

#### TSP Art (Traveling Salesman)
```
1. Place stipple points based on image darkness
2. Solve TSP to connect all points with shortest path
3. Single continuous line recreates image
```

#### Fourier Drawing
```
Any closed curve = sum of rotating circles (epicycles)
DFT of path points gives circle radii and speeds
Recreate drawing as single continuous motion
```

---

## Implementation Notes

### Rust Pattern Interface
```rust
/// All patterns should implement this signature:
pub fn generate_<pattern>_fill(
    polygon: &Polygon,
    spacing: f64,
    angle_degrees: f64,
) -> Vec<Line>
```

### Path Optimization
Many patterns produce disconnected segments. Consider:
- Nearest-neighbor ordering to minimize pen-up travel
- 2-opt improvement for better paths
- Eulerian path finding for connected patterns (Truchet)

### Clipping
All patterns must be clipped to polygon boundaries:
1. Generate pattern over bounding box with padding
2. Clip each line/segment to polygon using ray casting
3. Handle holes by subtracting from clipped segments

---

## References

### Libraries & Tools
- [vpype](https://github.com/abey79/vpype) - Swiss-army knife for plotter graphics
- [Paper.js](http://paperjs.org/) - Boolean operations on bezier paths
- [Clipper](https://github.com/angusj/Clipper2) - Fast polygon clipping

### Academic
- Fortune, S. (1987) "A sweepline algorithm for Voronoi diagrams"
- Turing, A. (1952) "The Chemical Basis of Morphogenesis"
- Prusinkiewicz, P. & Lindenmayer, A. (1990) "The Algorithmic Beauty of Plants"
- Grünbaum, B. & Shephard, G.C. (1987) "Tilings and Patterns"
- Amidror, I. (2009) "The Theory of the Moiré Phenomenon"

### Online Resources
- [DIY SVG Hatching](https://observablehq.com/@plmrry/diy-svg-hatching)
- [Optimal Path Planning for Pen Plotters](https://engineerdog.com/2021/08/18/optimal-path-planning-and-hatch-filling-for-pen-plotters/)

---

## Wild Ideas

From the svg-grouper research - experimental directions:

- **Woven Paper**: Cut patterns into strips, weave together
- **Tunnel Books**: Multiple cut frames with depth
- **Data Visualization**: GPS tracks, music waveforms as patterns
- **Anamorphic Patterns**: Distorted images that correct in cylindrical mirror
- **Generative Typography**: Fill regions with tiny text
- **Destruction Art**: Patterns with designed tear lines

---

*Compiled from svg-grouper research - December 2024*
