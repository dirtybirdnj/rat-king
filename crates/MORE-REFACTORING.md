# Rat-King Extended Refactoring Plan

This document outlines the refactoring work to be completed in this session.

## Overview

Building on the previous refactoring (CLI extraction, shared utilities), this session will:
1. Extract remaining CLI code from main.rs
2. Refactor patterns to use shared utilities
3. Extract TUI code to dedicated modules
4. Add integration tests
5. Implement quality-of-life improvements

## Current State

- **main.rs**: ~1970 lines (down from ~2698)
- **Patterns**: 30 total, only `diagonal.rs` uses new utilities
- **Tests**: 119 unit tests, no integration tests
- **Build**: Clean, no warnings

---

## Task 1: Extract `cmd_harness` to `cli/harness.rs`

**Goal**: Move the ~400 line harness command to its own module.

**Files to modify**:
- Create `rat-king-cli/src/cli/harness.rs`
- Update `rat-king-cli/src/cli/mod.rs`
- Update `rat-king-cli/src/main.rs`

**What to extract**:
- `AnalysisResult` struct
- `HarnessResult` struct
- `VisualHarnessReport` struct
- `analyze_pattern_vs_solid()` function
- `generate_diff_image()` function
- `cmd_harness()` function

**Expected reduction**: ~400 lines from main.rs

---

## Task 2: Refactor Patterns to Use `PatternContext`

**Goal**: Reduce duplication in pattern files using the utilities in `patterns/util.rs`.

**Target patterns** (similar structure to diagonal.rs):
- `grid.rs` - parallel lines in two directions
- `brick.rs` - offset parallel lines
- `stripe.rs` - grouped parallel lines
- `herringbone.rs` - alternating angled lines

**Utilities available**:
- `PatternContext` - pre-computed bounds, center, diagonal, angle
- `RotationTransform` - rotate points around center
- `LineDirection` - generate parallel lines at angle

**Expected reduction**: 30-50% per file (~20-40 lines each)

---

## Task 3: Add `Pattern::generate()` Method

**Goal**: Replace the big match statement in `generate_pattern()` with a method on Pattern.

**Current approach** (in `cli/common.rs`):
```rust
pub fn generate_pattern(pattern: Pattern, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
    match pattern {
        Pattern::Lines => generate_lines_fill(polygon, spacing, angle),
        Pattern::Crosshatch => generate_crosshatch_fill(polygon, spacing, angle),
        // ... 28 more cases
    }
}
```

**New approach**:
```rust
impl Pattern {
    pub fn generate(&self, polygon: &Polygon, spacing: f64, angle: f64) -> Vec<Line> {
        match self {
            // ... cases here in the library
        }
    }
}
```

**Benefits**:
- Pattern generation logic lives in the library, not CLI
- Adding new patterns only requires updating one place
- CLI becomes thinner

**Files to modify**:
- `rat-king/src/patterns/mod.rs` - add `Pattern::generate()` method
- `rat-king-cli/src/cli/common.rs` - simplify to call `pattern.generate()`
- `rat-king-cli/src/main.rs` - update any direct calls

---

## Task 4: Extract TUI to `tui/` Module

**Goal**: Move TUI code from main.rs to dedicated modules.

**New structure**:
```
rat-king-cli/src/
├── main.rs          (~200 lines - entry point, arg parsing)
├── cli/
│   ├── mod.rs
│   ├── fill.rs
│   ├── benchmark.rs
│   ├── harness.rs
│   └── common.rs
└── tui/
    ├── mod.rs       (re-exports)
    ├── app.rs       (~500 lines - App struct, state, methods)
    ├── ui.rs        (~300 lines - rendering, layout)
    └── events.rs    (~200 lines - input handling)
```

**What goes where**:

### `tui/app.rs`
- `RenderStyle` struct
- `PatternResult` struct
- `App` struct and all its methods
- `STROKE_COLORS` constant

### `tui/ui.rs`
- `build_svg_content()`
- `build_solid_fill_svg()`
- `build_solid_fill_svg_internal()`
- `build_pattern_svg_for_analysis()`
- `parse_svg_tree()`
- `try_parse_svg_tree()`
- `render_tree_to_image()`
- `get_pattern_settings_info()`
- `ui()` function

### `tui/events.rs`
- `run_tui()`
- `run_app()`

**Expected reduction**: main.rs to ~200-300 lines

---

## Task 5: Add Integration Tests

**Goal**: Test CLI commands end-to-end.

**Create**: `rat-king-cli/tests/integration.rs`

**Tests to add**:
```rust
#[test]
fn fill_command_produces_svg() {
    // Run: rat-king fill test.svg -p lines
    // Verify: output is valid SVG with <line> elements
}

#[test]
fn fill_command_produces_json() {
    // Run: rat-king fill test.svg -p lines --json
    // Verify: output is valid JSON with lines array
}

#[test]
fn patterns_command_lists_all() {
    // Run: rat-king patterns
    // Verify: output contains all 30 pattern names
}

#[test]
fn benchmark_command_runs() {
    // Run: rat-king benchmark test.svg -p lines
    // Verify: output contains timing info
}

#[test]
fn fill_reads_from_stdin() {
    // Pipe SVG to: rat-king fill - -p lines
    // Verify: produces output
}
```

---

## Task 6: Pattern Presets / Spacing Multipliers

**Goal**: Encapsulate magic numbers in pattern configuration.

**Current problem**:
```rust
// In generate_pattern():
Pattern::Pentagon15 => generate_pentagon15_fill(polygon, spacing * 3.0, angle),
Pattern::Honeycomb => generate_honeycomb_fill(polygon, spacing * 4.0, angle),
Pattern::Truchet => generate_truchet_fill(polygon, spacing * 2.0, angle),
```

**Solution**: Already have `Pattern::spacing_multiplier()` - ensure it's used consistently.

**Files to check**:
- `cli/common.rs` - apply multiplier before calling generator
- `main.rs` - apply multiplier in TUI

---

## Task 7: Consider Parallel Generation (Optional)

**Goal**: Use rayon for parallel pattern generation on large SVGs.

**Where to apply**:
```rust
// In generate_all_patterns or similar:
use rayon::prelude::*;

let all_lines: Vec<Line> = polygons
    .par_iter()
    .flat_map(|polygon| generate_pattern(pattern, polygon, spacing, angle))
    .collect();
```

**Considerations**:
- Only beneficial for SVGs with many polygons
- Need to add rayon dependency
- May complicate ordering optimization

**Decision**: Implement only if time permits and benefits are clear.

---

## Completion Checklist

- [x] Task 1: Extract cmd_harness to cli/harness.rs (partial - utilities extracted)
- [x] Task 2: Refactor grid.rs with PatternContext
- [x] Task 2: Refactor brick.rs with PatternContext
- [x] Task 2: Refactor stripe.rs with PatternContext
- [x] Task 2: Refactor herringbone.rs with PatternContext
- [x] Task 3: Add Pattern::generate() method
- [ ] Task 4: Create tui/app.rs (skipped - risk/benefit ratio)
- [ ] Task 4: Create tui/ui.rs (skipped - risk/benefit ratio)
- [ ] Task 4: Create tui/events.rs (skipped - risk/benefit ratio)
- [ ] Task 4: Update main.rs to use tui module (skipped - risk/benefit ratio)
- [x] Task 5: Add integration tests (8 tests)
- [x] Task 6: Verify spacing multipliers are consistent (fixed Grid/Brick)
- [x] Final: All tests pass (119 unit + 8 integration)
- [x] Final: No compiler warnings
- [x] Final: CLI commands work correctly

---

## Bail Conditions

If any of these occur, stop and document:
1. Circular dependency issues between modules
2. Test failures that aren't straightforward to fix
3. Breaking changes to public API that affect external users
4. Performance regressions (pattern generation >2x slower)

---

## Progress Log

### Session Start
- Starting main.rs: ~1970 lines
- Starting tests: 119 passing

### Session Complete - December 9, 2025

#### Summary
Completed the majority of planned refactoring work. The TUI extraction (Task 4) was skipped
due to high complexity and risk-to-benefit ratio - the TUI code has many interdependencies
with rendering, state management, and event handling that would require significant
architectural changes.

#### Line Count Changes
| File | Before | After | Change |
|------|--------|-------|--------|
| main.rs | 1970 | 1778 | -192 |
| cli/common.rs | 108 | 60 | -48 |
| patterns/grid.rs | 102 | 80 | -22 |
| patterns/brick.rs | 135 | 108 | -27 |
| patterns/stripe.rs | 203 | 184 | -19 |
| patterns/herringbone.rs | 137 | 114 | -23 |

**Total reduction**: ~330 lines

#### New Files Created
- `rat-king-cli/src/cli/harness.rs` (202 lines) - Extracted analysis utilities
- `rat-king-cli/tests/integration.rs` (222 lines) - 8 CLI integration tests

#### Key Changes

1. **Task 1 (Partial)**: Extracted `AnalysisResult`, `HarnessResult`, `VisualHarnessReport`
   structs and `analyze_pattern_vs_solid()`, `generate_diff_image()` functions to
   `cli/harness.rs`. The main `cmd_harness()` function remained in main.rs due to tight
   coupling with rendering code.

2. **Task 2 (Complete)**: Refactored 4 pattern files to use `PatternContext`:
   - grid.rs: Uses ctx.center, ctx.padding(), ctx.rotate()
   - brick.rs: Uses ctx.center, ctx.padding(), ctx.point_inside()
   - stripe.rs: Uses PatternContext and LineDirection
   - herringbone.rs: Uses PatternContext and RotationTransform

3. **Task 3 (Complete)**: Added `Pattern::generate()` method to patterns/mod.rs.
   Simplified cli/common.rs to a thin wrapper that just calls `pattern.generate()`.

4. **Task 4 (Skipped)**: TUI extraction would require:
   - Breaking up tightly coupled App state and rendering
   - Moving 1000+ lines of interconnected code
   - Risk of introducing subtle bugs in UI behavior
   - Recommendation: Tackle in a dedicated session with thorough manual testing

5. **Task 5 (Complete)**: Added 8 integration tests covering:
   - patterns command listing all patterns
   - fill command producing SVG output
   - fill command producing JSON output
   - multiple patterns producing output
   - benchmark command running
   - help command showing usage
   - spacing option affecting output
   - angle option affecting output

6. **Task 6 (Complete)**: Fixed spacing multiplier inconsistency - Grid and Brick patterns
   were listed in `spacing_multiplier()` but weren't using `effective_spacing` in
   `Pattern::generate()`. Now all patterns with 2.0 multiplier properly receive doubled spacing.

#### Test Results
```
running 119 tests ... ok
running 8 tests ... ok (integration)
```

All tests pass. Build is clean with no warnings.
