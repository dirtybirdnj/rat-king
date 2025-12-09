# Merge Tab Improvements for svg-grouper

This document outlines recommended improvements for the svg-grouper "merge" tab based on real-world SVG path issues discovered while testing with `envelope1-4colors.svg`.

## Problem Summary

SVG files exported from design tools often contain **compound paths** - single `<path>` elements containing multiple independent shapes. These are created when:
- Multiple shapes are "merged" or "combined" in vector editors
- Text is converted to outlines (each character becomes a subpath)
- Complex stamps, logos, or decorative elements are grouped

**Example**: A circular stamp with wavy border text had 11 subpaths in a single `<path>` element. The rat-king backend was only processing the first subpath, losing 10 shapes.

---

## Recommended Features for the Merge Tab

### 1. Compound Path Detection & Display

**What to show:**
- Count of subpaths per `<path>` element
- Visual indicator (badge/warning) on paths with >1 subpath
- Highlight compound paths in the layer tree with a distinctive icon

**Why it matters:**
Backend processing may handle compound paths differently. Users need to know which paths are compound before sending to the fill engine.

**Implementation hints:**
- Count `M` (MoveTo) commands in path `d` attribute
- Each `M` after the first indicates a new subpath
- Regex: `/M[\d.-]/g` gives approximate subpath count

---

### 2. Subpath Separation Tool

**Feature:** "Explode Compound Path" button that splits a compound path into separate `<path>` elements.

**User flow:**
1. Select a path with multiple subpaths
2. Click "Separate Subpaths"
3. Each subpath becomes its own `<path>` element
4. Optionally preserve original path ID with suffix (`stamp_0`, `stamp_1`, etc.)

**Why it matters:**
- Gives users control over how shapes are grouped
- Allows selective processing of specific subpaths
- Fixes issues where backend can't handle compound paths

---

### 3. Path Diagnostics Panel

Display these diagnostics for selected paths:

| Diagnostic | Description | Problem Indicator |
|------------|-------------|-------------------|
| **Subpath Count** | Number of MoveTo commands | >1 = compound path |
| **Winding Direction** | CW vs CCW per subpath | Mixed winding may indicate holes |
| **Closed vs Open** | Has closing `Z` command | Open paths may not fill correctly |
| **Self-Intersections** | Path crosses itself | Can cause fill artifacts |
| **Point Count** | Vertices after curve flattening | Very high count = complex/slow |
| **Bounding Box** | min/max coordinates | Helps identify outliers |

---

### 4. Path Preview Mode

**Feature:** Visual overlay showing path structure:

- **Different colors** for each subpath within a compound path
- **Vertex markers** showing actual points
- **Direction arrows** showing winding direction
- **Highlight mode** for the currently selected subpath

**Implementation:**
- Parse the path `d` attribute
- Render each subpath with a different stroke color
- Add small circles at each vertex
- Draw arrows along path direction

---

### 5. Winding Direction Indicator & Fixer

**What it does:**
- Shows whether each subpath is clockwise (CW) or counter-clockwise (CCW)
- CW subpaths inside CCW outer paths are typically "holes"
- Provides "Reverse Winding" action to flip direction

**Why it matters:**
- Fill algorithms use winding to determine inside/outside
- Incorrectly wound paths cause inverted fills
- Some backends require consistent winding (all CCW or all CW)

**Calculation:**
```javascript
// Shoelace formula - positive = CCW, negative = CW
function getWindingDirection(points) {
  let sum = 0;
  for (let i = 0; i < points.length; i++) {
    const j = (i + 1) % points.length;
    sum += (points[j].x - points[i].x) * (points[j].y + points[i].y);
  }
  return sum > 0 ? 'CW' : 'CCW';
}
```

---

### 6. Degenerate Geometry Detection

Flag these problematic geometries:

| Issue | Detection | Suggested Fix |
|-------|-----------|---------------|
| **Zero-length segments** | Consecutive identical points | Remove duplicate |
| **Collinear points** | 3+ points on same line | Remove middle points |
| **Micro-segments** | Segments < 0.01 units | Merge nearby points |
| **Unclosed paths** | No `Z` command | Auto-close or warn |
| **< 3 points** | Degenerate polygon | Remove or warn |

**"Clean Path" action:** One-click fix for common issues.

---

### 7. Merge Preview with Warnings

Before applying merge operations, show:

1. **Preview render** of what the merged result will look like
2. **Warnings** for detected issues:
   - "This will create a compound path with 11 subpaths"
   - "Mixed winding directions detected - some shapes may render as holes"
   - "Self-intersecting result detected"
3. **Option to proceed** or **separate instead**

---

### 8. Layer-by-Color Analysis

**Feature:** Automatic grouping and analysis by fill color.

Show for each color group:
- Number of paths
- Total subpath count
- Any paths with issues (compound, unclosed, etc.)

**Why it matters:**
The envelope SVG had 4 color groups, each needing different handling:
- Aqua stripes (41 simple paths) - worked fine
- Black shapes (67 paths, many compound) - needed separation
- Red stamp (compound with 11 subpaths) - was broken
- Blue/green accents (3 paths) - worked fine

---

## Implementation Priority

### High Priority (Core functionality)
1. Compound Path Detection & Display
2. Subpath Separation Tool
3. Path Diagnostics Panel

### Medium Priority (Enhanced usability)
4. Path Preview Mode
5. Winding Direction Indicator
6. Degenerate Geometry Detection

### Lower Priority (Polish)
7. Merge Preview with Warnings
8. Layer-by-Color Analysis

---

## Test Assets

Use `test_assets/envelope1-4colors.svg` to verify implementations:
- Contains compound paths (stamp with 11 subpaths)
- Multiple color layers
- Mix of simple and complex paths
- Real-world edge cases from actual design workflow

---

## Backend Fix Applied

The rat-king backend was updated to handle compound paths:

**Before:** `path_to_polygon()` returned `Option<Polygon>`, breaking on second MoveTo
**After:** `path_to_polygons()` returns `Vec<Polygon>`, processing all subpaths

Result:
- Before: 90 polygons extracted, circular stamp missing
- After: 224 polygons extracted, all shapes render correctly

The frontend should still implement these features because:
1. Users need visibility into path structure
2. Some workflows benefit from manual control
3. Separation before export gives predictable results
4. Not all backends will handle compound paths correctly
