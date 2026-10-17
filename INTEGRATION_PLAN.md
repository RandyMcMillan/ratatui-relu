# Plan: Integrate `filetreelist` into `ratatui-relu`

## 1. Goal

Bring the standalone `crates/filetreelist` crate (a sorted-path file tree with
fold/collapse, selection, and key-based navigation) into the `ratatui-relu`
project so that:

- it is a first-class, versioned part of the workspace,
- it is exposed to Rust consumers as `ratatui_relu::file_tree`,
- a Ratatui widget renders the tree (visible items, indentation, collapse
  glyphs, selection highlight, scroll viewport),
- it is reachable from the C FFI layer in `sys/`,
- an example and tests demonstrate the integration.

The integration must not break the existing `cargo build` / `cargo test` of
`ratatui-relu` or `sys`.

## 2. Current state (verified)

- Root package: `ratatui-relu` 0.8.0, edition 2024, deps include `ratatui 0.30.2`,
  `crossterm`, `serde`, `dirs`.
- `crates/filetreelist` is a standalone package `gnostr-filetreelist` 971106…,
  edition 2021, dep `thiserror 1.0`, dev-dep `pretty_assertions 1.4`.
- It builds and all 30 tests pass. Two non-fatal deprecation warnings
  (`std::usize` module) exist.
- No Cargo workspace is defined; `crates/` is not referenced by the root.
- `src/lib.rs` exposes: `animation`, `chart`, `layout`, `mouse`, `multi_panel`,
  `report_grid`.
- `sys/` FFI crate depends on `ratatui-relu` via path and exposes `Transition`
  and `MultiPanelState`.

## 3. Integration strategy

Use a **path dependency** first (no workspace migration) to keep the change
minimal and reversible. Optionally convert to a Cargo workspace in a later
phase. Keep the crate's public API (`FileTree`, `FileTreeItem`,
`TreeItemInfo`, `MoveSelection`) unchanged; add a thin Ratatui adapter on top.

## 4. Phases

### Phase 0 — Housekeeping (no behavior change)

1. Add `gnostr-filetreelist` as a path dependency in the root `Cargo.toml`:
   ```toml
   [dependencies]
   gnostr-filetreelist = { path = "crates/filetreelist" }
   ```
2. Re-export from `src/lib.rs`:
   ```rust
   pub mod file_tree;
   ```
   (new module) and `pub use gnostr_filetreelist::*;` where appropriate.
3. Fix the two `std::usize` deprecation warnings in the crate
   (`use std::usize;` → remove; use `usize::try_from` is not needed; the
   imports are unused except as a module path. Verify nothing breaks).
4. Run `cargo build`, `cargo test`, `cargo clippy` for both root and crate.

**Deliverable:** crate reachable from `ratatui_relu`.

### Phase 1 — Rust adapter module `src/file_tree.rs`

Design a Ratatui-friendly wrapper around `FileTree`.

New types:

```rust
pub struct FileTreeState {
    pub tree: FileTree,            // from gnostr_filetreelist
    pub scroll: usize,             // first visible absolute index offset (visual)
    pub viewport_height: usize,    // updated during render
}

pub struct FileTreeWidget<'a> {
    pub items: &'a [&'a Path],    // or owned Vec<PathBuf>
    pub collapsed: &'a BTreeSet<String>,
    pub show_icons: bool,
    pub selected_style: Style,
    pub normal_style: Style,
    pub indent_width: u16,
}
```

Responsibilities:

- Construct `FileTree` from a path list + collapsed set (lazy, on first render
  or explicit `set_items`).
- Render only visible items (`FileTree::iterate(visual_start, max)`).
- Apply scroll offset; compute `viewport_height` from area.
- Highlight the selected row.
- Map Ratatui key events to `MoveSelection` via a helper:
  ```rust
  pub fn handle_key(state: &mut FileTreeState, code: KeyCode) -> bool
  ```
  mapping Up/Down/Left/Right/Home/End/PageUp/PageDown.
- Expose `selected_path() -> Option<&Path>`.

State persistence: optionally serialize collapsed/selection? `FileTree` is not
`Serialize`. Keep state in-memory for v1; document limitation.

Tests (in module):
- renders without panic on empty / single / nested trees,
- selection movement updates visual selection,
- collapse hides children.

### Phase 2 — Integration with `MultiPanel`

- Allow `FileTreeState` to be stored alongside app state (not necessarily
  serialized).
- Provide a demo content closure example for left sidebar file tree.
- Add key bindings in `MultiPanelApp::handle_key` only if a file-tree mode is
  active (gated by a flag to avoid clashing with existing keys). Prefer
  keeping file-tree keys in the widget/adapter, not the app, to keep concerns
  separate.

### Phase 3 — FFI (`sys/src/lib.rs`)

Add opaque handle + functions mirroring the crate API:

- `ratatui_relu_filetree_new(paths, collapsed, count)` — tricky because paths
  are strings; design a simple builder: accept a NUL-joined buffer or an array
  of `*const c_char`. Recommend array of `*const c_char` + count.
- `ratatui_relu_filetree_free`
- `ratatui_relu_filetree_move(ptr, dir: i32) -> bool`
- `ratatui_relu_filetree_collapse(ptr, index, recursive)`
- `ratatui_relu_filetree_expand(ptr, index, recursive)`
- `ratatui_relu_filetree_selected_path(ptr) -> *mut c_char` (caller frees)
- `ratatui_relu_filetree_count_visible(ptr) -> usize`

Also expose a render helper is out of scope for FFI (rendering is app-side).

### Phase 4 — Example & docs

- Add `examples/file_tree.rs` demonstrating navigation and collapse.
- Update README feature list.
- Add doc comments and module-level docs.

### Phase 5 — Quality

- `cargo fmt`, `cargo clippy --all-targets` for root, crate, sys.
- Add tests for adapter and FFI smoke (if feasible).
- Consider converting root + crate + sys into a Cargo workspace for unified
  builds (optional, last).

## 5. File change summary

| File | Change |
|------|--------|
| `Cargo.toml` (root) | add path dep |
| `src/lib.rs` | add `pub mod file_tree;` |
| `src/file_tree.rs` | new adapter module + tests |
| `crates/filetreelist/src/filetree.rs` | fix deprecation import |
| `crates/filetreelist/src/filetreeitems.rs` | fix deprecation import |
| `sys/src/lib.rs` | FFI functions |
| `sys/Cargo.toml` | (unchanged; inherits) |
| `examples/file_tree.rs` | new example |
| `README.md` | feature + example entries |
| `INTEGRATION_PLAN.md` | this plan |

## 6. Risks & mitigations

- **Edition mismatch (2021 vs 2024):** path deps across editions are fine; Cargo
  handles per-crate edition.
- **Clippy strictness:** the crate denies `unwrap`/`panic`/etc. Consuming it
  does not propagate denies to dependents. But our adapter must itself pass
  root clippy (root allows? root has no clippy config; default). Keep adapter
  clippy-clean by avoiding unwrap in production paths.
- **Path lifetime & ownership:** `FileTree::new` takes `&[&Path]`. Adapter
  should own `Vec<PathBuf>` and build `Vec<&Path>` views; rebuild on change.
- **Serialization:** `FileTree` not serializable; state is runtime-only. Document.
- **Name collision:** crate exported as `gnostr_filetreelist`. Re-export under
  `ratatui_relu::file_tree` to give a stable brand.
- **Unicode/indent cast:** `u8::try_from` may fail on deep nesting; surface as
  `Result` and handle gracefully (already in crate).

## 7. Success criteria

- `cargo build` (root) succeeds.
- `cargo test` (root) succeeds and new adapter tests pass.
- `cargo test` in `crates/filetreelist` still passes.
- `cargo build -p ratatui-relu-sys` succeeds.
- `cargo run --example file_tree` (after Phase 4) runs.
- No new clippy warnings in touched files.

## 8. Open questions

1. Should the crate be renamed to `ratatui-relu-filetreelist` for branding?
   (Recommend: keep upstream name, alias in re-export.)
2. Should file-tree state be persisted? (Recommend: v1 no; revisit.)
3. Should FFI expose rendering or only data/navigation? (Recommend: data only.)

## 9. First concrete step

Apply Phase 0 (dependency + re-export + deprecation fix) and verify builds
before writing the adapter. This is the smallest reversible change and unblocks
all later phases.
