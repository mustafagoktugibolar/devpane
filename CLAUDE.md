# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this project is

`devpane` is a CLI tool that reads `.dpane` workspace files (YAML) and manages terminal pane lifecycles. It validates configs, resolves paths, builds launch plans, and spawns processes for auto-start panes. A Tauri desktop app (`src-tauri/` + `ui/`) reuses the same library to render PTY-backed terminal panes (xterm.js + portable-pty).

## Commands

```powershell
# Build
cargo build

# Run tests (all)
cargo test

# Run a single test by name
cargo test <test_name>

# Run tests in a specific module
cargo test --test-module config::settings

# Run the binary
cargo run -- validate examples/webclient.dpane
cargo run -- inspect examples/webclient.dpane
cargo run -- plan    examples/webclient.dpane
cargo run -- run     examples/webclient.dpane

# Desktop app (Tauri); aliases defined in .cargo/config.toml
cargo app          # = cargo tauri dev
cargo app-build    # = cargo tauri build

# UI type check (covers .vue files via vue-tsc)
npm run check --prefix ui
```

## Architecture

The pipeline from CLI input to process execution follows this sequence:

```
.dpane file → DevPaneConfig (raw YAML) → Workspace (resolved paths) → WorkspaceRuntime (with PaneStatus) → ProcessLaunch → run_launches
```

**`crates/devpane/src/config/`** — Raw deserialized config from YAML. `model.rs` defines the structs (`DevPaneConfig`, `PaneConfig`, `LayoutNode`). `settings.rs` implements resolution helpers (`pane_shell`, `pane_auto_start`, `scrollback`) with pane → global → platform-default precedence. `paths.rs` resolves `workspace_root` and `pane_cwd` (relative paths are anchored to the `.dpane` file directory, then to workspace root). `validation.rs` checks semantic constraints that YAML deserialization cannot catch.

**`crates/devpane/src/workspace/`** — `builder.rs` calls `build_workspace` to produce a fully resolved `Workspace` with concrete paths and settings. `runtime.rs` wraps it in `WorkspaceRuntime`, which tracks per-pane `PaneStatus` (`Idle → Starting → Running → Exited/Failed`). The runtime is the mutable state passed through the process lifecycle.

**`crates/devpane/src/process/`** — `launch.rs` maps a `WorkspacePane` to a `ProcessLaunch` (program + args). Shell args differ by platform: on Windows, pwsh gets `-NoExit -Command <cmd>` (interactive) or `-Command <cmd>` (headless); on Unix, the shell gets `-lc <cmd>`. `manager.rs` owns lifecycle transitions and produces `ProcessLaunch` values. `runner.rs` actually spawns OS processes via `std::process::Command` and waits for them all.

**`crates/devpane/src/output.rs`** — Pure formatting functions for all CLI output. No side effects; all return `String`.

**`crates/devpane/src/cli.rs`** — Clap-based CLI with four subcommands: `validate`, `inspect`, `plan`, `run`. All take a single path argument pointing to a `.dpane` file.

## .dpane file format

```yaml
version: 1
name: My Workspace
root: ../       # optional; defaults to the .dpane file directory

settings:       # optional global defaults
  shell: pwsh
  auto_start: true
  scrollback: 1000

layout:
  direction: horizontal   # split node
  children:
    - pane: app           # pane reference
    - pane: worker

panes:
  app:
    name: App
    cwd: src              # relative to workspace root
    shell: pwsh           # overrides settings.shell
    command: cargo run
    auto_start: true      # overrides settings.auto_start
```

Only `version`, `name`, `layout`, and `panes` are required. `settings` and per-pane overrides are all optional. Unknown fields are rejected (`deny_unknown_fields`), and each pane may appear in the layout at most once.

## Dependency notes

- `serde_yaml` is unmaintained (archived upstream). It works fine today; revisit (e.g. `serde-yaml-ng`) before adding YAML-heavy features.
- The UI build runs `vue-tsc`, not plain `tsc` — plain `tsc` silently skips `.vue` files and misses template/script errors.
