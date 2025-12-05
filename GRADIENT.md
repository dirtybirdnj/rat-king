# Gradient Techniques for Pen Plotters

Research on creating color gradients and tonal variations for pen plotter output.

## Overview

Pen plotters can't vary line thickness or color intensity - they draw with fixed-width pens. Gradients must be achieved through:
1. **Line density variation** - more lines = darker
2. **Multi-pen color mixing** - optical blending of overlapping colors
3. **Positional color mapping** - assign colors based on location

---

## Technique 1: Variable Line Density (Monochrome Gradients)

Darker areas receive denser line patterns, lighter areas get sparser lines. Creates tonal gradation with a single pen.

### How It Works
- Calculate brightness/intensity for each region
- Map intensity to line spacing: `spacing = min_spacing + (1 - intensity) * (max_spacing - min_spacing)`
- Darker = tighter spacing, lighter = wider spacing

### Tools
- **[vpype-flow-imager](https://github.com/serycjon/vpype-flow-imager)** - Flow field line art with density control
  - `-ms, --min_sep` - minimum flowline separation (default: 0.8)
  - `-Ms, --max_sep` - maximum flowline separation (default: 10)
  - Density varies based on source image brightness

- **[StippleGen](https://wiki.evilmadscientist.com/StippleGen)** - Weighted Voronoi stippling
  - More dots placed in darker regions
  - Can export as SVG for plotting

### Pros/Cons
| Pros | Cons |
|------|------|
| Single pen needed | Monochrome only |
| Simple to implement | Limited dynamic range |
| Fast to plot | Can look "screentone-ish" |

---

## Technique 2: CMYK Color Separation

Split image into Cyan, Magenta, Yellow, and Black (Key) layers. Each layer is hatched separately and plotted with the corresponding pen color. Optical color mixing occurs when layers overlap.

### How It Works
1. Convert RGB image to CMYK color space
2. Generate separate hatch pattern for each channel
3. Rotate each layer's hatch angle (e.g., C=15°, M=75°, Y=0°, K=45°)
4. Plot each layer with corresponding pen
5. Eye blends overlapping colors

### Tools
- **[Drawing Bot V3](https://docs.drawingbotv3.com/en/latest/cmyk.html)** - Full-featured CMYK separation
  - Automatic layer splitting
  - All path-finding modules supported
  - Recommended "Darken" blend mode

- **[MorphPlot](https://github.com/cbrunschen/MorphPlot)** - Color hatching with error diffusion
  - Processes colors by pen width (widest first)
  - Linear error diffusion for gray level approximation
  - Supports HPGL, HP/GL-2, PostScript output

- **[Hackaday Color Crosshatching](https://hackaday.io/project/4823-color-crosshatching-with-pen-plotter)**
  - Pipeline: Bitmap → CMYK separation → crosshatch → gcode
  - Processing sketches for implementation

### Angle Separation
Critical for good color mixing - each color should cross others, not align:

| Color | Traditional Angle | Alternative |
|-------|------------------|-------------|
| Cyan | 15° | 0° |
| Magenta | 75° | 45° |
| Yellow | 0° | 90° |
| Black | 45° | 135° |

### Pros/Cons
| Pros | Cons |
|------|------|
| Full color reproduction | Requires 4 pens |
| Industry-standard approach | Multiple plotting passes |
| Good for photographs | Registration must be precise |

---

## Technique 3: Custom Palette Mapping (Sunset Example)

Map regions to a custom color palette based on position, brightness, or other criteria. Perfect for artistic effects like sunsets, ocean depths, heat maps.

### Sunset Gradient Example: Orange → Pink → Purple

```
Color Stops:
  Position 0.0 (top):    Orange  #FF6B35
  Position 0.5 (middle): Pink    #FF1493
  Position 1.0 (bottom): Purple  #8B008B
```

### Implementation Approaches

#### A. Y-Position Based
```python
def get_color_for_polygon(polygon, bbox, color_stops):
    """Assign color based on polygon's Y position in overall bbox."""
    centroid_y = sum(p.y for p in polygon.outer) / len(polygon.outer)
    # Normalize to 0-1 range
    t = (centroid_y - bbox.min_y) / (bbox.max_y - bbox.min_y)
    return interpolate_color_stops(color_stops, t)
```

#### B. Brightness Based
```python
def get_color_for_polygon(polygon, source_image, color_stops):
    """Assign color based on average brightness in source image."""
    # Sample pixels within polygon bounds
    brightness = sample_average_brightness(source_image, polygon)
    return interpolate_color_stops(color_stops, brightness)
```

#### C. Blend Zone Interleaving
Where colors meet, interleave both at reduced density:
```
Zone A (100% orange):     ||||||||||||
Blend Zone (50/50):       ||  ||  ||  ||  (orange)
                            ||  ||  ||    (pink)
Zone B (100% pink):       ||||||||||||
```

### Output: Multiple Layers
Generate separate SVG layers (or vpype layers) for each color:
```bash
# Each layer plotted with different pen
Layer 1: Orange pen  - polygons with t < 0.33
Layer 2: Pink pen    - polygons with 0.33 < t < 0.66
Layer 3: Purple pen  - polygons with t > 0.66
```

---

## Technique 4: Density + Color Combined

Combine variable density within each color zone for smoother transitions.

### Algorithm
```python
def generate_gradient_fill(polygon, y_position, color_stops, base_spacing):
    # 1. Determine primary and secondary colors based on position
    color_a, color_b, blend_factor = get_blend_colors(y_position, color_stops)

    # 2. Adjust density based on blend factor
    # At blend boundaries, reduce density of each color
    density_a = 1.0 - (blend_factor * 0.5)  # Fades out
    density_b = blend_factor * 0.5           # Fades in

    # 3. Generate lines for each color at appropriate density
    spacing_a = base_spacing / density_a if density_a > 0 else float('inf')
    spacing_b = base_spacing / density_b if density_b > 0 else float('inf')

    lines_a = generate_hatch(polygon, spacing_a, angle=0)
    lines_b = generate_hatch(polygon, spacing_b, angle=45)  # Different angle!

    return {'color_a': lines_a, 'color_b': lines_b}
```

---

## Implementation Plan for rat-king

### Phase 1: Single-Color Density Gradients
Add `--gradient` option to fill command:
```bash
rat-king fill input.svg --pattern lines --gradient vertical --min-spacing 1 --max-spacing 10
```

### Phase 2: Multi-Color Palette
Add `--palette` option with color stops:
```bash
rat-king fill input.svg --pattern lines --palette "0:#FF6B35,0.5:#FF1493,1:#8B008B" --gradient vertical
```

### Phase 3: CMYK Mode
Add `--cmyk` flag for automatic 4-layer separation:
```bash
rat-king fill photo.svg --cmyk --pattern crosshatch
```

### vpype Integration
```bash
# As vpype plugin
vpype read input.svg ratking fill --gradient vertical --palette "sunset" write output.svg
```

---

## Resources

### Tools
- [Drawing Bot V3](https://drawingbotv3.com/) - Full-featured image to plotter conversion
- [vpype-flow-imager](https://github.com/serycjon/vpype-flow-imager) - Flow field density hatching
- [MorphPlot](https://github.com/cbrunschen/MorphPlot) - Multi-color hatching with error diffusion
- [StippleGen](https://wiki.evilmadscientist.com/StippleGen) - Weighted Voronoi stippling

### References
- [Plotting Raster Images](https://mattwidmann.net/notes/plotting-raster-images/) - Comprehensive overview of techniques
- [Color Crosshatching with Pen Plotter](https://hackaday.io/project/4823-color-crosshatching-with-pen-plotter) - CMYK workflow
- [Evil Mad Scientist Multicolor Tips](https://wiki.evilmadscientist.com/Multicolor_Plot_Tips) - Practical multi-pen advice
- [Pen Plotter Art & Algorithms](https://mattdesl.svbtle.com/pen-plotter-1) - Matt DesLauriers' techniques

### Academic
- "Creating evenly-spaced streamlines of arbitrary density" - Jobard and Lefer (flow field basis for vpype-flow-imager)
