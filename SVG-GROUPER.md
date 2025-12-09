# SVG Grouper: Bitmap-to-Vector Color Pipeline

This document covers tools for extracting dominant colors from bitmap images, which can then be traced to SVG and filled with rat-king patterns.

## Pipeline Overview

```
Bitmap Image (PNG/JPG)
    ↓
[Color Extraction] ← pylette, swatchify, kmeans-colors
    ↓
Color Palette (N dominant colors)
    ↓
[Color Separation] (split image by color regions)
    ↓
[Tracing] (potrace, vtracer, etc.)
    ↓
Multi-layer SVG (one layer per color)
    ↓
[rat-king fill] (patterns per layer)
    ↓
Plotter-ready output
```

---

## 1. Pylette (Python)

**Repository:** https://github.com/qTipTip/Pylette
**Install:** `pip install pylette`

### Features
- K-Means and Median-Cut quantization algorithms
- Batch processing with parallel execution
- JSON export with hex colors and percentages
- RGB, HSV, HLS color space support
- Alpha channel handling for transparency

### Usage

```python
from pylette import extract_colors

# Extract 5 dominant colors
palette = extract_colors(
    image='input.png',
    palette_size=5,
    mode='KM',  # K-Means (or 'MC' for Median-Cut)
    sort_mode='luminance'
)

# Access colors
for color in palette:
    print(f"#{color.hex}: {color.frequency:.1%}")

# Random color selection weighted by frequency
random_color = palette.random_color(weighted=True)
```

### CLI

```bash
pylette extract input.png --palette-size 5 --output palette.json
pylette batch ./images/ --palette-size 8 --workers 4
```

### Relevance to rat-king
- Pre-analyze images before tracing
- Determine optimal number of color layers
- Weight patterns by color prominence

---

## 2. Swatchify (Go)

**Repository:** https://github.com/james-see/swatchify
**Website:** https://james-see.github.io/swatchify/
**Install:** `go install github.com/james-see/swatchify@latest` or `brew install james-see/tap/swatchify`

### Features
- K-means clustering for dominant color extraction
- Automatic image downscaling for speed
- JSON output with hex values and percentage weights
- Visual palette generation with proportional color blocks
- Cross-platform CLI

### Usage

```bash
# Extract 5 dominant colors
swatchify -i image.png -n 5

# JSON output
swatchify -i image.png -n 5 -json

# Generate visual palette
swatchify -i image.png -n 5 -palette output.png
```

### Output Format

```json
{
  "colors": [
    {"hex": "#2C3E50", "percent": 35.2},
    {"hex": "#E74C3C", "percent": 28.1},
    {"hex": "#ECF0F1", "percent": 20.5},
    {"hex": "#3498DB", "percent": 16.2}
  ]
}
```

### Relevance to rat-king
- Fast CLI integration
- JSON output for pipeline automation
- Percentage weights for layer ordering

---

## 3. kmeans-colors (Rust) - RECOMMENDED

**Repository:** https://github.com/okaneco/kmeans-colors
**Crates.io:** https://crates.io/crates/kmeans_colors
**Install:** `cargo install kmeans-colors`

### Why This One?
- **Pure Rust** - Same ecosystem as rat-king
- Library AND CLI - Can integrate directly into rat-king
- Lab color space support - Better perceptual clustering
- K-means++ initialization - More stable results

### CLI Usage

```bash
# Extract 5 colors, output palette image
kmeans-colors -i input.png -k 5 -o palette.png

# Get average color (k=1)
kmeans-colors -i input.png -k 1

# Proportional swatches sorted by frequency
kmeans-colors -i input.png -k 8 --proportional --sort

# Print color percentages
kmeans-colors -i input.png -k 5 --percentage

# Apply palette to another image ("color style transfer")
kmeans-colors -i source.png -k 5 --replace target.png -o styled.png
```

### Library Usage

```rust
use kmeans_colors::{get_kmeans, Kmeans};
use palette::{Lab, Srgb, FromColor};

// Load image pixels as Lab colors
let pixels: Vec<Lab> = load_image_as_lab("input.png");

// Run k-means
let result = get_kmeans(
    5,           // k clusters
    20,          // max iterations
    0.0001,      // convergence threshold
    false,       // verbose
    &pixels,
    42           // random seed
);

// Get centroids (dominant colors)
for (i, centroid) in result.centroids.iter().enumerate() {
    let rgb = Srgb::from_color(*centroid);
    println!("Color {}: #{:02X}{:02X}{:02X}",
        i,
        (rgb.red * 255.0) as u8,
        (rgb.green * 255.0) as u8,
        (rgb.blue * 255.0) as u8
    );
}
```

### Integration with rat-king

```toml
# Cargo.toml
[dependencies]
kmeans_colors = { version = "0.6", default-features = false, features = ["palette_color"] }
palette = "0.7"
```

### Relevance to rat-king
- Direct Rust integration possible
- Could add `rat-king analyze` command
- Lab color space = better visual clustering
- Same dependency tree (palette crate)

---

## 4. Cushy (Rust GUI Framework)

**Repository:** https://github.com/khonsulabs/cushy
**Docs:** https://cushy.rs/
**Status:** Alpha, actively developed

### Why Consider It?

Currently rat-king uses ratatui for TUI. Cushy offers a path to a native GUI:

- **wgpu-powered** - Hardware-accelerated 2D rendering
- **Reactive data model** - Familiar if you know React/Vue
- **Cross-platform** - Windows, macOS, Linux
- **Pure Rust** - No Electron, no web views

### Key Features

- Widget-based architecture
- Async support (tokio integration)
- OKLab color calculations (palette crate)
- Built-in theming

### Example

```rust
use cushy::Run;
use cushy::widget::MakeWidget;
use cushy::widgets::Label;

fn main() -> cushy::Result {
    "Hello, Cushy!"
        .into_label()
        .centered()
        .run()
}
```

### Comparison with Alternatives

| Framework | Rendering | Maturity | Use Case |
|-----------|-----------|----------|----------|
| **Cushy** | wgpu | Alpha | Native apps, custom rendering |
| **Iced** | wgpu/tiny-skia | Beta | Elm-like architecture |
| **egui** | Many backends | Stable | Immediate mode, games |
| **Tauri** | System webview | Stable | Web skills → desktop |
| **Slint** | Multiple | Stable | Embedded, QML-like |

### Relevance to rat-king

For a full GUI pattern editor with:
- Real-time SVG preview (current TUI limitation)
- Color picker integration
- Drag-and-drop file handling
- Export dialogs

Cushy could be the path forward, but it's early. Consider:
1. Keep ratatui TUI for CLI users
2. Add optional Cushy GUI behind feature flag
3. Share core logic between both

---

## Recommended Pipeline

```bash
# 1. Analyze image for dominant colors
kmeans-colors -i photo.png -k 5 --percentage > colors.json

# 2. Separate image by color (requires custom tool or ImageMagick)
# This creates 5 single-color layers

# 3. Trace each layer to SVG (using potrace or vtracer)
vtracer --input layer1.png --output layer1.svg

# 4. Fill each layer with rat-king
rat-king fill layer1.svg -p lines -s 2.0 -o layer1_filled.svg
rat-king fill layer2.svg -p crosshatch -s 3.0 -o layer2_filled.svg

# 5. Combine layers for plotting
# (order by color percentage for best visual effect)
```

---

## Future rat-king Integration Ideas

1. **`rat-king analyze <image>`** - Extract dominant colors using kmeans_colors
2. **`rat-king prepare <image> -k 5`** - Full pipeline: analyze → separate → trace → fill
3. **Color-aware pattern selection** - Denser fills for darker colors
4. **Layer ordering optimization** - Minimize pen changes on plotter

---

## References

- [Pylette Documentation](https://qtiptip.github.io/Pylette/)
- [Swatchify GitHub](https://github.com/james-see/swatchify)
- [kmeans-colors Docs](https://docs.rs/kmeans_colors/latest/kmeans_colors/)
- [Cushy Documentation](https://cushy.rs/main/docs/cushy/)
- [palette crate](https://docs.rs/palette/latest/palette/) - Shared color library
