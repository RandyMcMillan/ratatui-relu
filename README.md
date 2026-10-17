# ratatui-relu [![Release](https://github.com/RandyMcMillan/ratatui-relu/actions/workflows/release.yml/badge.svg?event=release)](https://github.com/RandyMcMillan/ratatui-relu/actions/workflows/release.yml)

Continuous, differentiable layout transitions for [Ratatui](https://github.com/ratatui/ratatui).

`ratatui-relu` replaces hard `if`/`else` layout branches with smooth ReLU-gated transitions, making it easy to animate panels, optimize layouts with gradients, and build responsive TUIs.

## Features

- **`Transition`** — smooth interpolation between two values, useful for animating panel sizes.
- **`MultiPanel`** — a reusable five-area widget (header, footer, left/right sidebars, main content) with custom content closures.
- **`MultiPanelApp`** — a complete reusable application driver with key handling, persistence, boost modes, and an automated test matrix.
- **`NeighborReportGrid`** — a 3×3 grid widget that reports the dimensions of neighboring panels.
- **`FileTreeWidget`** — a file/folder tree widget backed by `filetreelist`, with fold, selection, and keyboard navigation.- **Exhaustive test matrix** — verifies all 256 panel/boost states × 4 arrow keys (1024 cases).

## Quick start

Add the dependency:

```toml
[dependencies]
ratatui-relu = { git = "https://github.com/RandyMcMillan/ratatui-relu.git" }
```

Use the `MultiPanel` widget:

```rust
use ratatui::widgets::StatefulWidget;
use ratatui_relu::multi_panel::{MultiPanel, MultiPanelState};

let mut state = MultiPanelState::open(4.0);

let widget = MultiPanel::new()
    .header_title("Header")
    .left_title("Sidebar")
    .main_title("Main")
    .footer_title("Footer")
    .panel_height(5)
    .sidebar_width_fraction(0.25);

frame.render_stateful_widget(widget, area, &mut state);
```

Or run a complete multi-panel app:

```rust
use ratatui_relu::multi_panel::MultiPanelApp;

let mut app = MultiPanelApp::new();

// in your event loop:
app.update(dt);
app.handle_key(key_code);
terminal.draw(|f| app.render(f))?;

// on exit:
app.save()?;
```

## Examples

```bash
cargo run --example multi_panel
cargo run --example animation
cargo run --example accessibility
cargo run --example side_menu_panel
cargo run --example smoothstep
cargo run --example content_aware
cargo run --example optimized_layout
cargo run --example differentiable_constraints
cargo run --example file_tree```

## Customizing `MultiPanelApp`

Supply your own widget builder and save path:

```rust
use std::path::PathBuf;
use ratatui_relu::multi_panel::{MultiPanelApp, MultiPanelAppConfig, MultiPanel};

let config = MultiPanelAppConfig {
    widget_builder: Box::new(|_app| {
        MultiPanel::new()
            .header_title("My App")
            .main_title("Content")
    }),
    save_path: Some(PathBuf::from("/tmp/my_app_state.json")),
};

let mut app = MultiPanelApp::with_config(config);
```

## Persistence and app state

`MultiPanelApp` automatically tries to load persisted state on construction and
can save it again later with [`MultiPanelApp::save`].

### Choosing where state is stored

State is read from and written to `MultiPanelAppConfig::save_path`:

```rust
use std::path::PathBuf;
use ratatui_relu::multi_panel::{MultiPanelApp, MultiPanelAppConfig, MultiPanel};

let config = MultiPanelAppConfig {
    widget_builder: Box::new(|_app| MultiPanel::new()),
    // Custom location. Parent directories are created automatically.
    save_path: Some(PathBuf::from("/tmp/my_app_state.json")),
    // `None` uses the default path:
    // ~/.config/ratatui-relu/multi_panel_app.json
    // save_path: None,
};

let mut app = MultiPanelApp::with_config(config);

// Persist the current state.
app.save()?;

// Load state from the configured path (returns None if the file is missing
// or cannot be parsed).
if let Some(loaded) = app.load() {
    app = loaded;
}
```

Options:

- `save_path: None` — use `MultiPanelApp::default_config_path()`,
  i.e. `<config_dir>/ratatui-relu/multi_panel_app.json`.
- `save_path: Some(path)` — use the given file. A relative path is resolved
  against the process working directory. Parent directories are created on save.
- `MultiPanelApp::load()` reads from the same configured path.
- `MultiPanelApp::save()` writes pretty-printed JSON and creates parent dirs.

### App state syntax

State is serialized as JSON. Persisted fields include:

- `state` — panel open/closed and transition state.
- `left_boost`, `right_boost`, `top_boost`, `bottom_boost` — boost flags.
- `show_grid` — whether the neighbor report grid is visible.
- `file_tree_data` — persisted file tree collapse/selection/scroll state.
- `file_view_scrolls` — per-file scroll positions.
- `selected_file` — currently selected file path.
- `line_numbers` — line number display toggle.

Fields marked `#[serde(skip)]` (such as `file_tree_state`, `git`, `test_mode`,
`config`, and transient UI state) are rebuilt at runtime and are not part of the
saved file. Unknown fields are ignored on load, so older or newer state files
remain compatible where possible.

## License

MIT OR Apache-2.0
