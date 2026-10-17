# Plan: Integrate `labtui-asyncgit` into `ratatui-relu`

## 1. Goal

Add non-blocking Git operations (status, diff, log, staging) to `ratatui-relu`
using `labtui-asyncgit 0.28.3`, and surface them through the existing
`FileTreeWidget` and `MultiPanelApp`:

- Annotate file-tree rows with Git status (new/modified/deleted/renamed/conflict).
- Show a diff for the selected file in the main panel.
- Keep the UI responsive via async fetch + notification channel.
- Reuse the existing key handling/focus machinery.

## 2. Current state (verified)

- `labtui-asyncgit = "0.28.3"` added to `Cargo.toml`; `cargo build` succeeds.
- `ratatui-relu` already integrates `gnostr-filetreelist` as
  `file_tree` (`FileTreeState`/`FileTreeWidget`).
- `MultiPanelApp` has a file tree in the left panel, file preview in the main
  panel, and focus handling (file tree vs. main view).
- `labtui-asyncgit` exposes `AsyncStatus`, `AsyncDiff`, `AsyncLog`, etc. and a
  crossbeam `Sender<AsyncGitNotification>`/`Receiver` pattern.

## 3. Phases

### Phase 0 — Dependency & smoke (done)

- Dependency added; build passes.
- Verify `cargo doc` for the crate is available locally.

### Phase 1 — Async plumbing in `MultiPanelApp`

1. Add fields (serde-skipped):
   - `repo_path: Option<PathBuf>`
   - `async_status: Option<AsyncStatus>`
   - `async_diff: Option<AsyncDiff>`
   - `git_receiver: Option<Receiver<AsyncGitNotification>>`
   - `git_sender: Option<Sender<AsyncGitNotification>>`
   - `git_status: git::Status` (last fetched)
   - `git_diff: Option<git::FileDiff>`
2. In `MultiPanelApp::new`/`with_config`, detect a Git repo (walk up from cwd
   for `.git`); if found, create the channel and `AsyncStatus`/`AsyncDiff`.
3. In `update(dt)`, poll the receiver (non-blocking) and on notification
   refresh `git_status`/`git_diff` from `.last()`.
4. Add a `git_refresh()` method that calls `async_status.fetch(...)` and
   `async_diff.fetch(...)` for the selected file.

### Phase 2 — Git status in file tree

1. Extend `FileTreeState` or a wrapper with a `status_map: HashMap<PathBuf, StatusItemType>`.
2. In render, colorize rows (and add a status glyph) based on the map.
3. Refresh status map whenever `AsyncGitNotification::Status` arrives or a
   mutating action completes.
4. Sort/group optionally by status (keep upstream sort for now).

### Phase 3 — Diff view in main panel

1. On file selection (Enter/Right), call `git_refresh()` for that path.
2. When `AsyncGitNotification::Diff` arrives, store `FileDiff`.
3. In `demo_widget_builder`, if a diff is available, render it (with hunk
   headers and line colors) instead of the plain file preview; otherwise fall
   back to the file preview.
4. Add scroll handling for the diff view (reuse `main_scroll`).

### Phase 4 — Staging & actions (optional, later)

1. Key bindings: `s` stage/unstage selected file, `d` discard, `c` commit
   (opens editor).
2. Use `labtui-asyncgit::sync::staging` and `commit` APIs.
3. Refresh status after each action.

### Phase 5 — FFI (optional)

1. Expose repo detection and status fetch to C via `sys/`.
2. Keep rendering app-side.

## 4. Risks & mitigations

- **Async runtime**: `labtui-asyncgit` uses `rayon_core::spawn` internally;
  ensure a global rayon pool is initialized (default is fine).
- **Thread safety**: all shared state behind `Arc<Mutex<_>>`; only read on the
  main thread via `.last()`.
- **Repo path**: handle non-repo directories gracefully (disable git features).
- **Performance**: status fetch on every notification; debounce in `update`.
- **Edition/deps**: asyncgit pulls `git2`/`gix`; may increase build time and
  require system libgit2/openssl. Document build deps.

## 5. Success criteria

- App detects a Git repo and shows file status colors in the tree.
- Selecting a file shows its diff in the main panel.
- UI remains responsive while fetching.
- `cargo build`, `cargo test`, `cargo clippy` pass; `sys` builds.

## 6. First concrete step

Implement Phase 1 (repo detection + channel + AsyncStatus) behind a feature
flag or optional module, without changing rendering, to validate plumbing.
