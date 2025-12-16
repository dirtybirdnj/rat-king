# Handwriting Service Research

This document captures research on handwriting capture, single-line font generation, and text rendering for pen plotters. The goal is to make rat-king a swiss-army-knife for single-line text rendering that can be used by multiple tools (writetyper, map labeling, artwork metadata).

## Background: Bond.co

Bond was a company (2013-2019) that built robotic handwriting machines:
- Founded by Sonny Caberwal, acquired by Newell Brands (Sharpie, Paper Mate) in 2016 for ~$20M
- Had 9-11 production machines producing ~1,500 notes daily
- Shut down March 2019, robots rescued and now operate as **Wami** serving luxury brands (LVMH, Kering, Richemont)
- Architecture: Mobile app → Backend handwriting engine → Custom robot arms with Montblanc pens

Our architecture mirrors this:
```
writetyper (UI) → rat-king (patterns/text) → vpype (optimization) → Duet3 → pen plotter
```

## Hershey Fonts Licensing

The original Hershey fonts are **safe to use and bundle**:

- Created by Dr. A.V. Hershey at US Naval Weapons Laboratory (1967)
- US government work = public domain in US
- JHF format by James Hurt requires only attribution
- **Can be bundled in rat-king with attribution notice**

Required attribution:
```
Hershey Fonts originally created by Dr. A. V. Hershey at the
U.S. National Bureau of Standards. JHF format by James Hurt, Cognition, Inc.
```

## Handwriting Capture Approaches

### 1. Template-Based Capture (Simplest)

**Tools:**
- **Calligraphr** (calligraphr.com) - Print template → write → scan → get TTF/OTF
- **Handwrite** (github.com/yashlamba/handwrite) - Open source, uses Potrace + FontForge
- **writetyper** - Already has glyph template generator built in

**Limitation:** Produces outline fonts, not single-stroke. Requires centerline tracing.

### 2. Centerline Tracing (Convert outlines → strokes)

**Tools:**
- **AutoTrace** with `-centerline` flag (github.com/autotrace/autotrace)
- **Inkscape 1.0+** - Built-in: Path → Trace Bitmap → Centerline tracing
- **pyautotrace** (github.com/lemonyte/pyautotrace) - Python bindings

```bash
# Convert scanned handwriting to single-stroke SVG
autotrace -centerline -output-format svg input.png -output-file output.svg
```

**Note:** Potrace does NOT support centerline tracing - it only does outline tracing.

### 3. Neural Network Synthesis (Most Sophisticated)

Based on Alex Graves' paper "Generating Sequences with Recurrent Neural Networks"

**Key Projects:**
- **sjvasquez/handwriting-synthesis** - RNN that outputs stroke sequences directly
- **hardmaru/write-rnn-tensorflow** - LSTM Mixture Density Network
- **Calligrapher.ai** - Web demo using similar technology, outputs SVG strokes
- **handwriting-helper** (github.com/Aptimex/handwriting-helper) - Extracts stroke data from calligrapher.ai

**Advantages:**
- Outputs actual stroke paths, not outlines
- Can be trained on custom handwriting samples
- Natural variation in output
- No centerline extraction needed

**Model Architecture:**
- 2-layer LSTM with skip connections
- Window layer for attention
- Mixture Density Network (MDN) output layer
- Trained on IAM On-Line Handwriting Database

## Recommended Workflow

### Phase 1: Capture
```
writetyper template → print → write with pen → scan at 300+ DPI
```

### Phase 2: Vectorize
```
For each glyph:
  autotrace -centerline -output-format svg glyph.png > glyph.svg
```

### Phase 3: Package
```
Combine glyphs into JHF or SVG font format
Store in ~/.config/rat-king/fonts/ or bundle with app
```

### Phase 4: Use
```bash
rat-king text "Hello World" -f myhandwriting --size 24 -o output.svg
rat-king text "2024-12-16" -f myhandwriting | vpype read - write out.gcode
```

## rat-king Text Command (Planned)

```bash
# Basic usage
rat-king text "Hello World" -f futural --size 24 -o label.svg

# List available fonts
rat-king text --list-fonts

# Use custom font
rat-king text "Map Title" -f ~/.fonts/myhand.jhf --size 18

# JSON output for programmatic use
rat-king text "Label" -f cursive --json

# Pipe to vpype
rat-king text "Signature" -f myhand | vpype read - linesort write out.gcode
```

## Font Formats to Support

| Format | Extension | Description |
|--------|-----------|-------------|
| Hershey JHF | `.jhf` | Original plotter font format, coordinates as ASCII |
| SVG Font | `.svg` | XML-based, glyphs as path data |
| Custom JSON | `.json` | Our own format for easy editing |

## Resources

### Open Source Projects
- github.com/autotrace/autotrace - Bitmap to vector with centerline
- github.com/sjvasquez/handwriting-synthesis - RNN handwriting generation
- github.com/Aptimex/handwriting-helper - Calligrapher.ai stroke extraction
- github.com/yashlamba/handwrite - Handwriting to font converter
- github.com/isdat-type/Relief-SingleLine - Single-line sans serif font
- github.com/fablabnbg/inkscape-centerline-trace - Inkscape extension

### Commercial/Web Tools
- calligraphr.com - Template-based font creation
- calligrapher.ai - Neural network handwriting synthesis (SVG output)

### Academic Papers
- "Generating Sequences with Recurrent Neural Networks" - Alex Graves (2013)
- "StrokeStyles: Stroke-based Segmentation and Stylization of Fonts" - ACM/Adobe Research

## Implementation Status

- [x] Hershey font parser in rat-king (`cli/hershey.rs`)
- [x] Showcase command uses Hershey fonts for labels
- [ ] `rat-king text` command for standalone text rendering
- [ ] Font discovery and listing
- [ ] Custom font format support
- [ ] Integration with autotrace for capture workflow
- [ ] Neural network synthesis (future)
