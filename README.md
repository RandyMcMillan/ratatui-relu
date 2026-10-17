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

## License

MIT OR Apache-2.0
