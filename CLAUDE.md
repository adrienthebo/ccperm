# CLAUDE.md

ccperm is a TUI viewer/editor for Claude Code permission settings files (`settings.json`). Built with ratatui + crossterm.

## Commands

```bash
cargo build          # Build
cargo test           # Run all tests
cargo test <name>    # Run a single test by name
cargo run            # Run the app (takes over terminal)
```

No linter or formatter is configured.

## Architecture

The main loop (`src/main.rs`): `draw UI → handle event → repeat`. All state lives in `App` (`src/app.rs`).

**Modal architecture**: `AppMode` enum drives both event handling dispatch (`event/handler.rs`) and which overlay renders (`ui/`). Adding a new mode means: add variant to `AppMode`, add handler function, add render arm.

**Three settings sources**: User (`~/.claude/settings.json`), Project (`<git-root>/.claude/settings.json`), Local (`<git-root>/.claude/settings.local.json`). Project and Local require a git repository. Each source is loaded/saved independently with per-source dirty tracking via `dirty: HashSet<SettingsSource>`.

**Permission tree**: Permissions are grouped by `PermissionCategory` into a collapsible tree. `FlatItem` enum represents the flattened view (category headers + permission rows).

## Gotchas

- **Category arrays must stay in sync**: `TreeState::default()` and `App::build_flat_items()` in `app.rs` both have hardcoded `PermissionCategory` arrays — and they currently have *different orderings*. Both must be updated when adding categories.
- **`Settings` preserves unknown fields**: Uses `#[serde(flatten)] other: serde_json::Value` so round-tripping doesn't drop fields the app doesn't know about.
- **Permission validation**: `Permission::validate()` checks tool names against a `KNOWN_TOOLS` constant (`Bash`, `Read`, `Edit`, `Write`, `WebFetch`, `Agent`) and the `mcp__` prefix.
- **Unused dependencies**: `tui-tree-widget` and `thiserror` are in Cargo.toml but not used in source.
