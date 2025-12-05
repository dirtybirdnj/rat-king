# rat-king Pattern Reference

Complete specification of all fill patterns supported by rat-king CLI and vpype plugin.

## Quick Reference

| Pattern | Description | Key Parameters |
|---------|-------------|----------------|
| `lines` | Parallel straight lines | spacing, angle, inset |
| `crosshatch` | Two sets of crossed lines | spacing, angle, inset, cross_angle |
| `concentric` | Nested polygon outlines | spacing, connect_loops |
| `honeycomb` | Hexagonal grid | spacing, angle, inset |
| `zigzag` | Parallel zigzag lines | spacing, angle, amplitude, inset |
| `wiggle` | Sinusoidal wavy lines | spacing, angle, amplitude, frequency, inset |
| `wave` | Wave interference pattern | spacing, angle, amplitude, frequency, inset |
| `spiral` | Archimedean spiral | spacing, angle, inset, over_diameter |
| `fermat` | Fermat (golden) spiral | spacing, angle, inset, over_diameter |
| `crossspiral` | Two crossed spirals | spacing, angle, inset, over_diameter |
| `radial` | Lines radiating from center | spacing, start_angle, inset |
| `hilbert` | Hilbert space-filling curve | spacing, inset |
| `gyroid` | TPMS-inspired pattern | spacing, angle, inset |
| `scribble` | Random walk fill | spacing, inset, density, seed |
| `custom` | User-defined tile shape | spacing, angle, inset, tile_shape, ... |

---

## Global Parameters

These parameters apply to all or most patterns:

### `spacing` (required)
- **Type**: float
- **Units**: SVG units (typically mm or px depending on document)
- **Description**: Distance between fill lines/elements
- **Default**: 2.0
- **Range**: > 0 (typically 0.5 - 20)

### `inset` (optional)
- **Type**: float
- **Default**: 0
- **Description**: Shrink polygon boundary before filling. Prevents lines from touching edges.
- **Range**: >= 0

### `angle` (optional)
- **Type**: float (degrees)
- **Default**: 0 or 45 (pattern-dependent)
- **Description**: Rotation angle for the pattern
- **Range**: -180 to 180 (or 0 to 360)

---

## Pattern Specifications

### `lines`
Parallel straight lines at a given angle.

```
Parameters:
  spacing: float      # Distance between lines
  angle: float        # Line angle in degrees (default: 45)
  inset: float        # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern lines --spacing 3 --angle 45

vpype Example:
  vpype read input.svg ratking fill -p lines -s 3 -a 45 write output.svg
```

**Notes:**
- Simplest and fastest pattern
- Good for large areas
- Combine with crosshatch for denser fill

---

### `crosshatch`
Two sets of parallel lines crossed at an angle.

```
Parameters:
  spacing: float      # Distance between lines
  angle: float        # Primary angle in degrees (default: 45)
  inset: float        # Edge inset (default: 0)
  cross_angle: float  # Angle between line sets (default: 90)

CLI Example:
  rat-king fill input.svg --pattern crosshatch --spacing 3 --angle 45 --cross-angle 90

vpype Example:
  vpype read input.svg ratking fill -p crosshatch -s 3 -a 45 --cross-angle 90 write output.svg
```

**Notes:**
- Creates denser coverage than single lines
- cross_angle=90 creates perpendicular grid
- cross_angle=60 creates triangular pattern

---

### `concentric`
Nested polygon outlines, shrinking inward.

```
Parameters:
  spacing: float       # Distance between loops
  connect_loops: bool  # Connect loops for continuous path (default: true)

CLI Example:
  rat-king fill input.svg --pattern concentric --spacing 2 --connect-loops

vpype Example:
  vpype read input.svg ratking fill -p concentric -s 2 --connect write output.svg
```

**Notes:**
- Excellent for pen plotters (minimal pen lifts when connected)
- Follows shape contour naturally
- May produce artifacts on complex shapes

---

### `honeycomb`
Hexagonal grid pattern.

```
Parameters:
  spacing: float  # Hexagon size (default: 2.0)
  angle: float    # Rotation angle (default: 0)
  inset: float    # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern honeycomb --spacing 5 --angle 30

vpype Example:
  vpype read input.svg ratking fill -p honeycomb -s 5 -a 30 write output.svg
```

**Notes:**
- Creates natural-looking organic fill
- Good structural pattern
- Each hexagon is ~1.5x the spacing value

---

### `zigzag`
Parallel zigzag lines.

```
Parameters:
  spacing: float    # Distance between zigzag rows
  angle: float      # Direction angle (default: 0)
  amplitude: float  # Height of zigzag peaks (default: 3)
  inset: float      # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern zigzag --spacing 4 --amplitude 5 --angle 0

vpype Example:
  vpype read input.svg ratking fill -p zigzag -s 4 --amplitude 5 write output.svg
```

**Notes:**
- Sharp peaks (vs smooth wiggle)
- Good for textured fills

---

### `wiggle`
Sinusoidal wavy lines.

```
Parameters:
  spacing: float     # Distance between wave rows
  angle: float       # Direction angle (default: 0)
  amplitude: float   # Wave height (default: 3)
  frequency: float   # Waves per unit length (default: 0.5)
  inset: float       # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern wiggle --spacing 4 --amplitude 3 --frequency 0.5

vpype Example:
  vpype read input.svg ratking fill -p wiggle -s 4 --amplitude 3 --frequency 0.5 write output.svg
```

**Notes:**
- Smooth sine wave curves
- frequency controls wave density
- amplitude controls wave height

---

### `wave`
Wave interference pattern (similar to wiggle).

```
Parameters:
  spacing: float     # Distance between wave rows
  angle: float       # Direction angle (default: 0)
  amplitude: float   # Wave height (default: 3)
  frequency: float   # Waves per unit length (default: 0.5)
  inset: float       # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern wave --spacing 4 --amplitude 3 --frequency 0.5

vpype Example:
  vpype read input.svg ratking fill -p wave -s 4 --amplitude 3 --frequency 0.5 write output.svg
```

---

### `spiral`
Archimedean spiral from polygon center.

```
Parameters:
  spacing: float        # Distance between spiral arms
  angle: float          # Starting angle offset (default: 0)
  inset: float          # Edge inset (default: 0)
  over_diameter: float  # Spiral extent multiplier (default: 1.5)

CLI Example:
  rat-king fill input.svg --pattern spiral --spacing 2 --over-diameter 1.5

vpype Example:
  vpype read input.svg ratking fill -p spiral -s 2 --over-diameter 1.5 write output.svg
```

**Notes:**
- Single continuous path from center outward
- over_diameter > 1 extends spiral beyond polygon bounds
- Good for circular shapes

**Single Pattern Mode:**
When filling multiple polygons, can use one global spiral clipped to all shapes:
```
rat-king fill input.svg --pattern spiral --single-pattern
```

---

### `fermat`
Fermat spiral (golden angle spiral).

```
Parameters:
  spacing: float        # Distance between arms
  angle: float          # Starting angle offset (default: 0)
  inset: float          # Edge inset (default: 0)
  over_diameter: float  # Spiral extent multiplier (default: 1.5)

CLI Example:
  rat-king fill input.svg --pattern fermat --spacing 2 --over-diameter 1.5

vpype Example:
  vpype read input.svg ratking fill -p fermat -s 2 --over-diameter 1.5 write output.svg
```

**Notes:**
- Uses golden angle (137.5°) for natural-looking distribution
- Creates sunflower-like pattern
- Very aesthetic for organic designs

---

### `crossspiral`
Two spirals crossed at 90°.

```
Parameters:
  spacing: float        # Distance between spiral arms
  angle: float          # Starting angle offset (default: 0)
  inset: float          # Edge inset (default: 0)
  over_diameter: float  # Spiral extent multiplier (default: 1.5)

CLI Example:
  rat-king fill input.svg --pattern crossspiral --spacing 3

vpype Example:
  vpype read input.svg ratking fill -p crossspiral -s 3 write output.svg
```

**Notes:**
- Combines two spirals for denser coverage
- Creates interesting moiré-like effects

---

### `radial`
Lines radiating from polygon center.

```
Parameters:
  spacing: float       # Angular spacing (degrees) between rays
  start_angle: float   # Starting angle (default: 0)
  inset: float         # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern radial --spacing 15 --start-angle 0

vpype Example:
  vpype read input.svg ratking fill -p radial -s 15 --start-angle 0 write output.svg
```

**Notes:**
- spacing here is angular, not linear
- Creates sunburst effect
- Works best on roughly circular shapes

---

### `hilbert`
Hilbert space-filling curve.

```
Parameters:
  spacing: float  # Approximate cell size
  inset: float    # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern hilbert --spacing 3

vpype Example:
  vpype read input.svg ratking fill -p hilbert -s 3 write output.svg
```

**Notes:**
- Single continuous path
- Recursively subdivided
- May be slow on large areas
- Good for avoiding pen lifts

**Single Pattern Mode:**
Can use one global Hilbert curve clipped to all shapes:
```
rat-king fill input.svg --pattern hilbert --single-pattern
```

---

### `gyroid`
TPMS (Triply Periodic Minimal Surface) inspired pattern.

```
Parameters:
  spacing: float  # Pattern scale
  angle: float    # Rotation angle (default: 0)
  inset: float    # Edge inset (default: 0)

CLI Example:
  rat-king fill input.svg --pattern gyroid --spacing 5

vpype Example:
  vpype read input.svg ratking fill -p gyroid -s 5 write output.svg
```

**Notes:**
- Creates flowing, organic curves
- Mathematically interesting pattern
- Can be computationally intensive

---

### `scribble`
Random walk fill pattern.

```
Parameters:
  spacing: float   # Average distance between scribble points
  inset: float     # Edge inset (default: 0)
  density: float   # Scribble density multiplier (default: 1.0)
  seed: int        # Random seed for reproducibility (default: 12345)

CLI Example:
  rat-king fill input.svg --pattern scribble --spacing 2 --density 1.5 --seed 42

vpype Example:
  vpype read input.svg ratking fill -p scribble -s 2 --density 1.5 --seed 42 write output.svg
```

**Notes:**
- Creates hand-drawn appearance
- Use seed for reproducible results
- density > 1 creates denser scribbles

---

### `custom`
User-defined tile shape.

```
Parameters:
  spacing: float       # Tile spacing
  tile_shape: Point[]  # Array of points defining tile outline
  inset: float         # Edge inset (default: 0)
  angle: float         # Rotation angle (default: 0)
  fill_tiles: bool     # Fill tiles vs outline only (default: false)
  tile_gap: float      # Extra gap between tiles (default: 0)
  tile_scale: float    # Scale factor for tiles (default: 1.0)

CLI Example:
  rat-king fill input.svg --pattern custom --tile-shape "[[0,0],[1,0],[0.5,1]]" --spacing 5

vpype Example:
  vpype read input.svg ratking fill -p custom --tile-shape "triangle" -s 5 write output.svg
```

**Built-in Tile Shapes:**
- `triangle`
- `square`
- `diamond`
- `hexagon`
- `star`
- `plus`
- `circle`

---

## CLI Interface

### Basic Usage
```bash
rat-king fill INPUT_FILE [OPTIONS]
```

### Common Options
```
-o, --output PATH       Output file (default: stdout)
-p, --pattern NAME      Fill pattern type (default: lines)
-s, --spacing FLOAT     Line spacing (default: 2.0)
-a, --angle FLOAT       Angle in degrees (default: 45)
--inset FLOAT           Edge inset (default: 0)
--verbose, -v           Print timing and statistics
```

### Pattern-Specific Options
```
--amplitude FLOAT       Wiggle/wave amplitude (default: 3)
--frequency FLOAT       Wiggle/wave frequency (default: 0.5)
--over-diameter FLOAT   Spiral over-extension (default: 1.5)
--cross-angle FLOAT     Crosshatch cross angle (default: 90)
--connect-loops         Connect concentric loops (flag)
--single-pattern        Use single global pattern for all shapes (flag)
--density FLOAT         Scribble density (default: 1.0)
--seed INT              Random seed for scribble (default: 12345)
```

### Examples
```bash
# Simple line fill
rat-king fill input.svg -o output.svg --pattern lines --spacing 3

# Crosshatch at 30 degrees
rat-king fill input.svg -o output.svg --pattern crosshatch --spacing 2 --angle 30

# Wavy fill with high amplitude
rat-king fill input.svg -o output.svg --pattern wiggle --spacing 4 --amplitude 5 --frequency 0.3

# Spiral with extended coverage
rat-king fill input.svg -o output.svg --pattern spiral --spacing 2 --over-diameter 2.0

# Connected concentric fill
rat-king fill input.svg -o output.svg --pattern concentric --spacing 2 --connect-loops

# Read from stdin, write to stdout
cat input.svg | rat-king fill --pattern honeycomb --spacing 5 > output.svg
```

---

## vpype Plugin Interface

### Registration
```python
# Registered as 'ratking' command group
vpype read input.svg ratking fill [OPTIONS] write output.svg
```

### Usage
```bash
# Basic fill
vpype read input.svg ratking fill -p lines -s 3 write output.svg

# Chain with other vpype commands
vpype read input.svg \
    ratking fill -p honeycomb -s 5 \
    linemerge \
    linesort \
    write output.svg

# Multiple patterns on different layers
vpype read input.svg \
    forlayer \
        ratking fill -p lines -s 3 -a %_i*30% \
    end \
    write output.svg
```

---

## Settings Interface for Test Harness

The test harness UI should expose these controls:

```typescript
interface PatternSettings {
  // Universal
  lineSpacing: number;      // All patterns
  angle: number;            // Most patterns
  inset: number;            // All patterns

  // Wiggle/Wave/Zigzag
  wiggleAmplitude: number;  // wiggle, wave, zigzag
  wiggleFrequency: number;  // wiggle, wave

  // Spiral family
  spiralOverDiameter: number; // spiral, fermat, crossspiral

  // Crosshatch
  crossAngle: number;       // crosshatch only

  // Concentric
  connectLoops: boolean;    // concentric only

  // Scribble
  scribbleDensity: number;  // scribble only
  scribbleSeed: number;     // scribble only

  // Mode flags
  singlePattern: boolean;   // spiral, fermat, hilbert
}
```

### Pattern-to-Settings Map

| Pattern | Uses Settings |
|---------|---------------|
| lines | spacing, angle, inset |
| crosshatch | spacing, angle, inset, crossAngle |
| concentric | spacing, connectLoops |
| honeycomb | spacing, angle, inset |
| zigzag | spacing, angle, inset, wiggleAmplitude |
| wiggle | spacing, angle, inset, wiggleAmplitude, wiggleFrequency |
| wave | spacing, angle, inset, wiggleAmplitude, wiggleFrequency |
| spiral | spacing, angle, inset, spiralOverDiameter |
| fermat | spacing, angle, inset, spiralOverDiameter |
| crossspiral | spacing, angle, inset, spiralOverDiameter |
| radial | spacing, angle (as start_angle), inset |
| hilbert | spacing, inset |
| gyroid | spacing, angle, inset |
| scribble | spacing, inset, scribbleDensity, scribbleSeed |

---

## Default Values

```python
PATTERN_DEFAULTS = {
    'spacing': 2.0,
    'angle': 45.0,
    'inset': 0.0,
    'amplitude': 3.0,
    'frequency': 0.5,
    'over_diameter': 1.5,
    'cross_angle': 90.0,
    'connect_loops': True,
    'density': 1.0,
    'seed': 12345,
    'single_pattern': False,
}
```
