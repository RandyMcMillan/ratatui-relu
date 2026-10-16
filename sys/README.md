# ratatui-relu-sys

C FFI bindings for `ratatui-relu`.

## Build & run the C example

```bash
cd sys
./build.sh
```

This builds the Rust sys crate as a static library and links `example.c`
against it.

## What's exposed

- `Transition` — create, free, set target, update, and read current/target values.
- `MultiPanelState` — create, free, update, toggle panels, and read panel sizes.

See `include/ratatui_relu.h` for the full C API.
