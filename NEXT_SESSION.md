# Next Session: Pattern Diagnostic & Feedback System

## Goal
Create tools to systematically evaluate pattern quality, gather performance metrics, and collect human feedback to improve default values and prioritize optimizations.

---

## 1. `diagnose` Command

Generate diagnostic output for a single pattern at multiple detail/spacing levels.

### Usage
```
rat-king diagnose <pattern> [OPTIONS]

Options:
  -o, --output <file>     Output SVG file (default: diagnose_<pattern>.svg)
  --json                  Also output metrics as JSON
  --feedback              Generate feedback template file
  --svg <file>            Use custom test SVG (default: built-in square)
  --levels <list>         Comma-separated spacing levels (default: 1,2,3,4,6,8,10,12,16)
```

### Output Grid
A single SVG showing the pattern at each spacing level in a grid, with metrics annotated:

```
┌─────────────────────────────────────────────────────────────┐
│  lines @ spacing=1     lines @ spacing=2     lines @ spacing=3  │
│  ████████████████      ████  ████  ████      ███    ███    ███  │
│  ████████████████      ████  ████  ████      ███    ███    ███  │
│  847 lines, 12ms       412 lines, 8ms        198 lines, 4ms     │
│  WARN: too dense       OK                    OK                 │
├─────────────────────────────────────────────────────────────┤
│  lines @ spacing=4     lines @ spacing=6     lines @ spacing=8  │
│  ██      ██      ██    █        █        █   █          █       │
│  ██      ██      ██    █        █        █   █          █       │
│  94 lines, 2ms         62 lines, 1ms         31 lines, <1ms     │
│  OK                    OK                    WARN: sparse       │
└─────────────────────────────────────────────────────────────┘
```

### Metrics Collected Per Level

| Metric | Description | Used For |
|--------|-------------|----------|
| `time_ms` | Generation time | Performance optimization |
| `line_count` | Total lines generated | Density indicator |
| `total_length_px` | Sum of all line lengths | Plotter time estimate |
| `points_per_line` | Avg points per line | Complexity (curves vs straight) |
| `bounds_violations` | Lines outside polygon | Bug detection |
| `coverage_pct` | Estimated fill coverage | Gap detection |
| `density` | Lines per square inch | Too dense / too sparse |

### Auto-Detection Flags

```rust
enum DiagnosticFlag {
    TooDense,          // density > threshold (e.g., >100 lines/sq inch)
    TooSparse,         // coverage < threshold (e.g., <70%)
    BoundsViolation,   // Any line extends outside polygon
    SlowGeneration,    // time_ms > threshold (e.g., >100ms)
    Ok,                // No issues detected
}
```

---

## 2. Feedback File Format

When `--feedback` is specified, generate a TOML file for human annotation:

### `diagnose_<pattern>.toml`
```toml
# Auto-generated diagnostic feedback file
# Fill in the [feedback] section after visual review

[meta]
pattern = "lines"
generated = "2024-12-10T10:30:00Z"
test_svg = "builtin:square"

[metrics]
# Auto-populated from diagnostic run

[metrics.spacing_1]
time_ms = 12
line_count = 847
bounds_ok = true
coverage_pct = 98.2
auto_flag = "too_dense"

[metrics.spacing_2]
time_ms = 8
line_count = 412
bounds_ok = true
coverage_pct = 96.5
auto_flag = "ok"

# ... more spacing levels ...

[feedback]
# === HUMAN REVIEW SECTION ===
# Fill in after visually inspecting diagnose_<pattern>.svg

# Overall pattern quality (1-5)
quality_rating = 0

# Is this pattern useful for pen plotting? (true/false/null)
plotter_useful = null

# Recommended spacing range for this pattern
recommended_min_spacing = 0.0  # Below this = too dense
recommended_max_spacing = 0.0  # Above this = incomplete fill

# Known issues (select from list or add custom)
# Options: incomplete_fill, bounds_violation, too_dense,
#          gaps_at_corners, slow_generation, artifacts, none
issues = []

# Free-form notes
notes = ""

# Reviewer name (optional)
reviewer = ""
```

---

## 3. Feedback Aggregation Commands

### `rat-king feedback summary`
Display summary of all reviewed patterns:

```
Pattern Feedback Summary
========================

Reviewed: 18/30 patterns

By Quality Rating:
  ★★★★★ (5): lines, crosshatch, zigzag
  ★★★★☆ (4): spiral, concentric, honeycomb
  ★★★☆☆ (3): hilbert, guilloche
  ★★☆☆☆ (2): phyllotaxis
  ★☆☆☆☆ (1): (none)
  Unrated:   12 patterns

Common Issues:
  gaps_at_corners: 5 patterns
  bounds_violation: 3 patterns
  slow_generation: 2 patterns

Plotter Useful: 15 yes, 2 no, 1 unknown
```

### `rat-king feedback defaults`
Generate recommended defaults JSON from feedback:

```json
{
  "pattern_defaults": {
    "lines": {
      "min_spacing": 2.0,
      "max_spacing": 12.0,
      "default_spacing": 4.0,
      "plotter_safe": true,
      "quality": 5
    },
    "crosshatch": {
      "min_spacing": 3.0,
      "max_spacing": 10.0,
      "default_spacing": 5.0,
      "plotter_safe": true,
      "quality": 5
    }
  },
  "global_defaults": {
    "safe_spacing": 4.0,
    "safe_patterns": ["lines", "crosshatch", "zigzag", "concentric"]
  }
}
```

---

## 4. Implementation Plan

### Phase 1: Basic Diagnostics
1. Create `cli/diagnose.rs` module
2. Implement multi-spacing grid generation
3. Add timing and line count metrics
4. Generate diagnostic SVG output

### Phase 2: Auto-Detection
1. Implement bounds checking (line outside polygon)
2. Implement coverage estimation (sampling-based)
3. Implement density calculation
4. Add warning flags to output

### Phase 3: Feedback System
1. Generate TOML feedback template
2. Create feedback summary command
3. Create defaults export command
4. Store feedback files in `feedback/` directory

### Phase 4: Integration
1. Update README with workflow documentation
2. Add feedback-driven defaults to CLI help
3. Create GitHub issue templates for pattern bugs

---

## 5. Error Classification Reference

### Automated Detection
| Error Class | Detection Method | Threshold |
|-------------|------------------|-----------|
| Too Dense | lines/sq inch | >100 |
| Too Sparse | coverage % | <70% |
| Bounds Violation | line endpoint check | any outside |
| Slow | generation time | >100ms |

### Human Assessment (Feedback File)
| Assessment | Question |
|------------|----------|
| Plotter Useful | Would this work well on a pen plotter? |
| Quality Rating | Overall aesthetic/functional quality (1-5) |
| Recommended Range | What spacing range produces good results? |
| Issues | What problems exist at any spacing level? |

---

## 6. Workflow for Pattern Review

```
1. Generate diagnostics for all patterns:
   for p in $(rat-king patterns); do
     rat-king diagnose $p --feedback -o feedback/diagnose_$p.svg
   done

2. Review each SVG visually, fill in TOML feedback section

3. Generate summary and defaults:
   rat-king feedback summary
   rat-king feedback defaults > pattern_defaults.json

4. Use defaults to improve CLI and agent recommendations
```

---

## Files to Create

| File | Purpose |
|------|---------|
| `cli/diagnose.rs` | Main diagnose command implementation |
| `cli/feedback.rs` | Feedback summary and defaults commands |
| `feedback/.gitkeep` | Directory for feedback files |

## Files to Modify

| File | Changes |
|------|---------|
| `cli/mod.rs` | Add diagnose and feedback modules |
| `main.rs` | Add command dispatch for diagnose/feedback |
| `README.md` | Document diagnostic workflow |
