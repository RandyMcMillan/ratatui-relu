# Reusability Review & Plan

## Overview

`ratatui-relu` is a TUI library focused on continuous/differentiable layout transitions with [Ratatui](https://github.com/ratatui/ratatui). It has both reusable primitives (`Transition`/animation utilities, layout helpers) and a large opinionated application (`MultiPanelApp`) with lots of integrated features (file tree, git, terminal, keyboard, etc.).

## What's well-structured for reuse

- **`animation.rs`** - Clean, minimal abstractions (`Transition`, `ColorTransition`, `AlphaTransition`, `blend`). Very reusable as standalone utilities.
- **`layout.rs`** - Nice demonstration of ReLU-style continuous gates vs `if`/`else`; self-contained and educational.
- **`mouse.rs`** - Generic `Button` with `ButtonState`, `VisualState`, `ButtonTheme`, and `is_mouse_over` helper. Clean separation of concerns.
- **`file_tree.rs`** - Wraps `filetreelist` with its own `FileTreeState` (serialization-friendly, git status overlay) and `FileTreeWidget`. Good adapter pattern. The `snapshot()`/`apply_persisted_state()` split is thoughtful.
- **`report_grid.rs`** - Simple, focused widget; takes dimensions/fractions as params. Easy to drop into other layouts.
- **`keyboard.rs`** - Self-contained virtual keyboard widget with responsive sizing.
- **`chart.rs`** - Pure Ratatui Chart wrapper for demos; easy to reuse.
- **`MultiPanel` widget** - Builder pattern is nice, `PanelContent` closures give flexibility. Good foundation for custom layouts.

## Reusability issues

### 1. `MultiPanel` / `MultiPanelApp` is too monolithic
- **`MultiPanel` (widget)** - Pretty reusable in principle, but has hardcoded defaults (header "Header", left "Files", etc.), a fixed panel structure (header/statusbar/footer + 5 areas) with no way to customize layout beyond fractions/heights. TODOs in `MultiPanel::new()` suggest unfinished work.
- **`MultiPanelApp`** - At ~2000+ lines, it couples everything together: file tree, git integration, embedded terminal (PTY), virtual keyboard, mouse handling, persistence, test matrix, color demo, chart mode, help/settings overlays, editor launching. It's more of an example app than a reusable component.
  - `MultiPanelAppConfig` with `widget_builder: Box<dyn Fn(&MultiPanelApp) -> MultiPanel>` is clever for customization, but passing `&MultiPanelApp` everywhere creates tight coupling.
  - Many fields are `#[serde(skip)]` with complex reconstruction logic in `with_config()` - fragile and hard to test.
  - `main.rs` duplicates some logic from `examples/multi_panel.rs` (both have similar event loops).

### 2. Tight coupling & hidden dependencies
- **File tree + git**: `FileTreeState` takes `StatusItemType` from `labtui-asyncgit` directly. Optional at runtime via `git_status_map`, but the type leak is awkward.
- **Terminal embedding**: `TerminalState` spawns threads and manages PTY - hard to reuse/test.
- **Global side effects**: `MultiPanel::render` logs dimensions to `/tmp/ratatui_dims.log` on every render - side effect, not configurable, bad for libraries.
- **Statusbar height hardcoded**: In `MultiPanel::render`, `let statusbar_height = 3_u16;` ignores `self.statusbar_height` field entirely. The field exists and has a builder method but isn't used. That looks like a bug.
- **Footer height calculation oddity**: Footer height uses `area.height.saturating_sub(statusbar_height) / 2` which doesn't match `self.footer_height` in an intuitive way.

### 3. Duplication
- **Button implementations**: `mouse.rs::Button` (stateful widget with `ButtonState`) vs `keyboard.rs::Button` (stateless widget with internal state enum). Different APIs, both exist. `keyboard.rs` only imports `is_mouse_over` from `crate::mouse`.
- **Event handling patterns**: Key handling is spread across `MultiPanelApp::handle_key`, `handle_key_event`, and example apps reimplement parts of it.
- **Terminal setup/teardown**: Repeated in `main.rs` and `examples/multi_panel.rs` (`setup_terminal`, `restore_terminal`, editor launch patterns).

### 4. API design issues
- **`MultiPanelState` missing statusbar updates**: `update()` only updates header/left_sidebar/right_sidebar/footer (missing `statusbar`). `snap()` and `toggle_all()` also miss `statusbar`. Statusbar is initialized but not animated/toggled consistently - likely bugs.
- **`PanelContent` takes `&mut MultiPanelState`** - forces mutable access even if not needed; could use `&MultiPanelState`.
- **Serialization**: `MultiPanelApp` does manual field-by-field reconstruction on load with lots of `#[serde(skip)]`. Error-prone. `MultiPanelState` serializes fine, but the app state is complex.
- **Inconsistent naming**: `statusbar_title` vs `footer_title` builder methods both exist - confusing. `statusbar_height` vs `footer_height` are separate concepts but layout treats them differently.
- **Feature creep in lib**: `lib.rs` exposes everything publicly. Some modules are very app-specific (`git`, `terminal`, `keyboard` in app context) while others are general utilities.
- **Heavy dependencies**: `labtui-asyncgit`, `portable-pty`, `tui-term`, `vt100` are required even if using just `Transition`/`MultiPanel` widget - bloats the crate.

### 5. Missing abstractions for reusability
- No trait for pluggable content providers beyond closures
- No way to inject custom key bindings cleanly
- No separation between "layout engine" and "application logic"
- `MultiPanelApp` hardcodes key mappings throughout `handle_key` (very long method)
- Difficult to test due to tight coupling and global side effects

## Concrete recommendations

### High value, easy wins
1. **Fix the statusbar bugs** - Add statusbar to `MultiPanelState::update()`, `snap()`, `toggle_all()`. Either use `self.statusbar_height` in `MultiPanel::render` or remove the unused field/builder methods.
2. **Remove `/tmp/ratatui_dims.log` logging** from library code - or gate behind feature flag/env var (e.g., `RATARELU_DEBUG_DIMS`).
3. **Consolidate Button implementations** - Pick one (`mouse.rs` is more flexible as a stateful widget), move to a shared module or re-export. Remove duplication from `keyboard.rs`.
4. **Extract common terminal setup** into a helper module (used by `main.rs` and examples) to reduce duplication.
5. **Make dependencies optional** via Cargo features: `git`, `terminal`, `keyboard`, `file-tree`, `charts` etc., so minimal users only pay for what they need. Keep `animation`, `layout`, `mouse` (core utilities) as default.

### For better architecture
6. **Split `MultiPanelApp`** into smaller pieces: `AppState` (serializable), `AppController` (logic), view/render separate. Or provide `MultiPanelApp` as an example/reference implementation in `examples/` and keep it out of the core public API.
7. **Make `MultiPanel` more layout-flexible** - Consider a layout configuration struct or trait instead of hardcoded 5-panel structure. Allow custom panel arrangements.
8. **Reduce `MultiPanelApp` coupling** - Use traits/interfaces for file system access, git, terminal spawning to make it testable.
9. **Improve `PanelContent`** - Consider `&MultiPanelState` instead of `&mut MultiPanelState` where mutation isn't needed; or provide both.
10. **Separate concerns in lib.rs** - Consider feature-gated modules and clearer re-exports. Document which modules are "primitives" vs "application components".

### API polish
11. **Clarify naming** - Resolve confusion between `statusbar` and `footer`. Either consolidate into one concept or document the difference clearly and use them consistently.
12. **Add `Default` impls** more consistently where sensible.
13. **Re-export key types from `lib.rs`** for convenience (e.g., `pub use animation::Transition;`, `pub use multi_panel::{MultiPanel, MultiPanelState};`).
14. **Make `MultiPanelApp` construction more predictable** - Simplify the complex reconstruction logic in `with_config()` or break it into smaller, testable methods.
15. **Add builder for `MultiPanelAppConfig`** for better ergonomics.

## Quick assessment

The core primitives (`Transition`, animation utilities, `MultiPanel` widget) are genuinely reusable and well-designed. `MultiPanelApp` is impressive as a demo/showcase but would need significant refactoring to be a clean library component others can embed. 

**Biggest wins for reusability:** fix the statusbar inconsistencies, remove side effects/logging from render, make heavy deps optional, and either slim down `MultiPanelApp` or clearly document it as an example application rather than a core reusable API.
