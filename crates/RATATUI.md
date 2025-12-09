# Rat-King TUI Extraction Plan

This document outlines the plan, architecture, and risks for extracting the TUI (Terminal User Interface) code from `main.rs` into dedicated modules.

## Current State

The TUI code in `main.rs` is approximately **1,000 lines** and consists of:

| Component | Lines | Description |
|-----------|-------|-------------|
| `RenderStyle` | 15 | Stroke styling configuration |
| `PatternResult` | 6 | Background generation result |
| `App` struct | 74 | Application state (~30 fields) |
| `App` impl | 450 | Methods for state management |
| `run_tui()` | 20 | Terminal initialization/cleanup |
| `run_app()` | 120 | Main event loop |
| `ui()` | 180 | Widget rendering |
| SVG builders | 160 | `build_svg_content()`, `build_solid_fill_svg()`, etc. |

### Key Dependencies

The TUI code depends on:
- **ratatui** - Terminal UI framework
- **ratatui-image** - Sixel image rendering
- **crossterm** - Terminal event handling
- **resvg/usvg** - SVG parsing and rendering
- **tiny_skia** - Rasterization
- **image** - Image manipulation
- **chrono** - Timestamps for screenshots

---

## Proposed Architecture

```
rat-king-cli/src/
├── main.rs              (~300 lines - entry point, CLI dispatch)
├── cli/
│   ├── mod.rs
│   ├── fill.rs
│   ├── benchmark.rs
│   ├── harness.rs
│   └── common.rs
└── tui/
    ├── mod.rs           (~30 lines - re-exports)
    ├── app.rs           (~500 lines - App struct, state, methods)
    ├── render.rs        (~200 lines - SVG building, image rendering)
    ├── ui.rs            (~200 lines - widget layout and drawing)
    └── events.rs        (~150 lines - event loop, input handling)
```

---

## Detailed Module Breakdown

### `tui/mod.rs`
Public interface for the TUI module.

```rust
mod app;
mod render;
mod ui;
mod events;

pub use app::{App, RenderStyle, PatternResult};
pub use events::run_tui;

// Constants
pub const DEFAULT_STROKE_WIDTH: f64 = 1.5;
pub const PIXELS_PER_CELL_X: u32 = 10;
pub const PIXELS_PER_CELL_Y: u32 = 20;
pub const STROKE_COLORS: &[(&str, &str)] = &[...];
```

### `tui/app.rs`
Application state and business logic.

**Move to this file:**
- `RenderStyle` struct
- `PatternResult` struct
- `App` struct (all 30+ fields)
- All `App` methods:
  - `new()` - initialization
  - `selected_pattern()` - pattern access
  - `regenerate_pattern()` - background generation
  - `rebuild_svg_tree()` - SVG cache refresh
  - `check_pattern_result()` - channel polling
  - `update_image()` - render refresh
  - `update_render_size()` - resize handling
  - `toggle_solid_fill()` - view mode
  - `save_screenshot()` - PNG export
  - `save_comparison_screenshots()` - comparison export
  - `set_status()` / `clear_old_status()` - status messages
  - `zoom_in()` / `zoom_out()` / `reset_view()` - zoom
  - `pan()` - pan
  - `next_color()` / `prev_color()` - color cycling
  - `increase_stroke_width()` / `decrease_stroke_width()` - line width
  - `toggle_strokes()` - stroke visibility
  - `select_pattern()` - pattern selection
  - `handle_click()` / `handle_drag()` / `handle_release()` - mouse
  - `next_pattern()` / `prev_pattern()` - navigation
  - `adjust_current_setting()` - setting adjustment

**Internal dependencies:**
- Calls `render::build_svg_content()` and friends
- Uses `STROKE_COLORS` constant

### `tui/render.rs`
SVG generation and image rendering.

**Move to this file:**
- `build_svg_content()` - lines + polygons to SVG
- `build_solid_fill_svg()` - solid fill SVG
- `build_solid_fill_svg_internal()` - internal helper
- `build_pattern_svg_for_analysis()` - analysis-only SVG
- `parse_svg_tree()` - usvg parsing
- `try_parse_svg_tree()` - safe usvg parsing
- `render_tree_to_image()` - tree to pixels

**Signature example:**
```rust
pub fn build_svg_content(
    lines: &[Line],
    polygons: &[Polygon],
    bounds: (f64, f64, f64, f64),
    style: &RenderStyle,
) -> String

pub fn render_tree_to_image(
    tree: &usvg::Tree,
    bounds: (f64, f64, f64, f64),
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    width: u32,
    height: u32,
) -> DynamicImage
```

### `tui/ui.rs`
Widget rendering and layout.

**Move to this file:**
- `ui()` function - main render function
- `get_pattern_settings_info()` - pattern metadata for display

**Note:** The `ui()` function is tightly coupled to `App` state access. Options:
1. Pass `&App` and access fields directly (current approach)
2. Create a `UiState` trait that `App` implements
3. Extract only the data needed into a `UiData` struct

### `tui/events.rs`
Event loop and input handling.

**Move to this file:**
- `run_tui()` - terminal setup/teardown
- `run_app()` - main event loop with all keybindings

**Complexity:** The event loop directly mutates `App` state. Options:
1. Keep direct `&mut App` access (simplest)
2. Use message passing (complex, benefits unclear)
3. Create an `Action` enum dispatched to `App::handle_action()`

---

## Implementation Order

### Phase 1: Extract render utilities (Low risk)
1. Create `tui/mod.rs` with constants
2. Create `tui/render.rs` with SVG/image functions
3. Update imports in `main.rs`
4. Test: verify TUI still works

### Phase 2: Extract App struct (Medium risk)
1. Create `tui/app.rs`
2. Move `RenderStyle`, `PatternResult`, `App`
3. Update imports
4. Test: all functionality preserved

### Phase 3: Extract UI rendering (Medium risk)
1. Create `tui/ui.rs`
2. Move `ui()` and helpers
3. Resolve any borrowing issues
4. Test: visual rendering correct

### Phase 4: Extract event loop (Higher risk)
1. Create `tui/events.rs`
2. Move `run_tui()` and `run_app()`
3. Test: all keybindings work
4. Test: mouse interactions work
5. Test: loading spinner animation works

---

## Risk Assessment

### High Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Borrow checker issues** | App state is accessed mutably during rendering and event handling. Moving to separate modules may cause lifetime conflicts. | Keep `App` in a single module initially; only extract pure functions first. |
| **Image state protocol** | `ratatui-image` uses `StatefulProtocol` which has ownership requirements. Moving this between modules needs care. | Keep image state management in `app.rs`, only extract rendering. |
| **Background thread coordination** | Pattern generation uses `mpsc` channels. Thread spawning with cloned data is complex. | Keep `regenerate_pattern()` in same module as channel endpoints. |
| **Terminal state corruption** | If extraction introduces panics in the event loop, terminal may be left in raw mode. | Wrap all TUI code in panic handler that restores terminal. |

### Medium Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Debounce timing** | View change debouncing uses `Instant` comparisons. Logic is straightforward but spread across methods. | Keep all timing logic in `App` methods. |
| **Mouse hit testing** | Click coordinates are compared against stored `Rect` areas. Layout changes could break this. | Add integration tests for mouse interactions. |
| **Status message lifecycle** | Messages have 3-second timeout. Edge cases in message clearing. | Add unit tests for status timing. |

### Low Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| **SVG building functions** | Pure functions, no state. Easy to extract. | Extract these first as proof of concept. |
| **Constants** | Static data, no logic. | Extract to `tui/mod.rs` immediately. |
| **RenderStyle struct** | Simple data struct with Default impl. | Move with App. |

---

## Testing Strategy

### Before Extraction
1. Document current behavior with manual test checklist
2. Record expected output for specific inputs
3. Note any existing quirks/bugs

### Manual Test Checklist
- [ ] TUI launches with default SVG
- [ ] TUI launches with custom SVG path
- [ ] Pattern list scrolls (j/k, up/down)
- [ ] Pattern selection changes preview
- [ ] Spacing adjustment (h/l, left/right, [/])
- [ ] Angle adjustment
- [ ] Stroke width adjustment
- [ ] Color cycling
- [ ] Stroke toggle
- [ ] Zoom in/out (+/-)
- [ ] Pan (WASD)
- [ ] Reset view (0)
- [ ] Solid fill toggle (f)
- [ ] Screenshot (Shift+S)
- [ ] Mouse click pattern selection
- [ ] Mouse drag pan
- [ ] Mouse scroll zoom
- [ ] Loading spinner animation
- [ ] Status messages appear and disappear
- [ ] Quit (q/Esc)
- [ ] Window resize handling

### After Each Phase
1. Run manual test checklist
2. Verify no regressions
3. Check for terminal state issues (raw mode left enabled)

---

## Alternative Approaches

### Option A: Minimal Extraction (Recommended)
Extract only pure functions (SVG builders, render helpers) to `tui/render.rs`. Keep all state and event handling in `main.rs`.

**Pros:**
- Low risk
- Immediate benefit (cleaner main.rs)
- Can iterate later

**Cons:**
- main.rs still ~800 lines
- Limited modularity

### Option B: Full Module Split
Complete extraction as described above.

**Pros:**
- Clean architecture
- Each module has single responsibility
- Easier to test in isolation

**Cons:**
- Higher risk of bugs
- Complex refactoring
- May surface borrow checker issues

### Option C: Rewrite with Different Architecture
Use an Elm-style architecture (Model-View-Update) with message passing.

**Pros:**
- Cleaner separation of concerns
- Easier to test
- Better for future features

**Cons:**
- Significant rewrite
- New bugs likely
- Requires learning new patterns
- Overkill for current complexity

---

## Recommendations

1. **Start with Option A** - Extract render utilities only
2. **Add tests** - Create the manual test checklist as automated checks where possible
3. **Iterate** - After Option A is stable, consider Option B
4. **Skip Option C** - The current architecture works; a rewrite isn't justified

### Immediate Next Steps
1. Create `tui/mod.rs` and `tui/render.rs`
2. Move `build_svg_content()` and related functions
3. Update imports in `main.rs`
4. Test manually
5. Commit if successful

---

## Dependencies to Watch

These crates have version-specific APIs that may complicate extraction:

- **ratatui 0.28+** - StatefulWidget trait changes
- **ratatui-image 1.0+** - Protocol trait changes
- **crossterm 0.27+** - Event enum changes

Check `Cargo.lock` before starting to ensure API compatibility.

---

## Estimated Effort

| Phase | Effort | Risk |
|-------|--------|------|
| Phase 1 (render) | 1-2 hours | Low |
| Phase 2 (App) | 2-3 hours | Medium |
| Phase 3 (UI) | 1-2 hours | Medium |
| Phase 4 (events) | 2-3 hours | High |
| Testing | 1-2 hours per phase | - |

**Total: 8-15 hours** including testing

---

## Success Criteria

1. All manual tests pass
2. No terminal state corruption on exit
3. No performance regression (render time, responsiveness)
4. Code compiles without warnings
5. main.rs reduced to ~300 lines (entry point only)
