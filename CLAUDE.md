# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```powershell
# Run all tests (also used for full workspace check)
cargo check-all

# Type-check the frontend
npm run build --prefix ui

# Check formatting (does not auto-fix)
cargo fmt --check

# Run a single test
cargo test -p devpane <test_name>

# CLI modes
cargo run -- validate examples/devpane.dpane
cargo run -- inspect examples/devpane.dpane
cargo run -- plan examples/devpane.dpane
cargo run -- run examples/devpane.dpane

# Desktop app (dev mode, starts Vite automatically)
cargo app

# Release build
cargo app-build
```

The aliases `cargo app`, `cargo app-build`, and `cargo check-all` are defined in `.cargo/config.toml`.

## Skills to Use

- Use `.claude/skills/rust-best-practices` before writing, reviewing, or refactoring Rust in `crates/devpane`, `crates/cli`, or `src-tauri`.
- Use `.claude/skills/error-handling-patterns` when changing validation errors, process launch failures, PTY lifecycle handling, persistence failures, or user-facing diagnostics.
- For security-sensitive work, apply the security rules in this file and `AGENTS.md`, especially around `.dpane` parsing, command execution, workspace paths, Tauri capabilities, and filesystem access.

## Architecture

This is a Cargo workspace with three members:

- **`crates/devpane`** — Pure core library. No Tauri, no PTY. Handles `.dpane` YAML parsing (`config/`), path resolution and settings merging, validation, workspace building (`workspace/`), and process launch planning and running (`process/`).
- **`crates/cli`** — Thin CLI binary. Parses subcommands (`validate`, `inspect`, `plan`, `run`) and delegates to `devpane`.
- **`src-tauri`** — Tauri desktop backend. Wraps the `devpane` library and adds PTY management (`portable-pty`). All Tauri IPC commands are in `src-tauri/src/lib.rs`. Exposes workspace load/save/delete, recent session management, and terminal lifecycle (start/write/resize/stop).
- **`ui/`** — Vue 3 + TypeScript + xterm.js frontend. Talks to the Tauri backend exclusively through the typed wrappers in `ui/src/api/`. Layout state manipulation is in `ui/src/layout.ts`. Active terminal sessions (PTY lifecycle) are tracked in `ui/src/terminalSessions.ts`.

### Ownership map

- `.dpane` parsing, validation, root/path resolution, and settings merging: `crates/devpane/src/config` and `crates/devpane/src/workspace`
- Process launch planning and lifecycle behavior: `crates/devpane/src/process`
- CLI subcommands and output shape: `crates/cli/src`
- Desktop IPC bridge, PTY management, app configuration, and persistence: `src-tauri/`
- Frontend API boundary for Tauri commands/events: `ui/src/api`
- Vue components, layout, and terminal UI behavior: `ui/src/components`, `ui/src/layout.ts`, and `ui/src/terminalSessions.ts`

Keep `crates/devpane` as the reusable core. It should not depend on Tauri, Vue, PTY-specific UI behavior, or desktop persistence details. CLI and Tauri code should delegate parsing, validation, workspace building, and launch planning to the core crate.

### Data flow for the desktop app

1. UI calls a Tauri command (e.g., `load_workspace`) → `src-tauri/src/lib.rs` handler
2. Handler calls `devpane::config::DevPaneConfig::load_from_file` + `devpane::workspace::build_workspace`
3. Resolved `WorkspaceSummary` is serialized as JSON and returned to the UI
4. UI renders panes via `TerminalWorkspace.vue` → `WorkspaceNodeView.vue` → `TerminalPane.vue`
5. `TerminalPane` calls `start_terminal` (IPC) → Tauri spawns a PTY, starts a reader thread
6. PTY output is emitted as Tauri events: `terminal-output-{pane_id}` and `terminal-exited-{pane_id}`

### `.dpane` YAML schema

- `version` must be `1`
- `layout` is a recursive tree of `{ pane: <id> }` leaves and `{ direction: horizontal|vertical, children: [...] }` split nodes
- `panes` is a map of pane id → `{ name, cwd, shell, command, auto_start }`
- `root` is optional; defaults to the directory containing the `.dpane` file

Treat `.dpane` files as untrusted input. Validation and inspection must not execute commands. Avoid shell interpolation when structured command/argument handling is available, preserve useful validation errors, and cover path resolution, environment handling, and command planning with focused tests.

### Windows-specific details

- On Windows, PTY reader threads do not see EOF when the child exits (ConPTY limitation). A separate waiter thread closes the master PTY to unblock the reader.
- CWD paths from `portable-pty` may carry a `\\?\` UNC prefix that is stripped before being passed to the PTY.
- Default shell is `powershell`; shell candidates in order: PowerShell, PowerShell 7 (`pwsh`), Command Prompt.

### Recent sessions

Stored as `sessions.json` in the Tauri app data directory. The desktop keeps up to 20 entries. Workspace files saved via the UI without an explicit path are placed in a `workspaces/` subdirectory of the app data dir.

## Validation Matrix

- Core parser, validation, workspace, or process-planning changes: run `cargo test --workspace`.
- CLI behavior changes: run `cargo run -- validate examples/devpane.dpane` and `cargo run -- inspect examples/devpane.dpane`.
- User-visible `.dpane` behavior changes: update or add an example under `examples/` and add focused Rust tests near the affected code.
- Tauri backend or IPC changes: run `cargo test --workspace`; if the change affects frontend data contracts, also run `npm run build --prefix ui`.
- Vue, TypeScript, layout, or terminal UI changes: run `npm run build --prefix ui`; for terminal lifecycle, pane resize, start/stop, or event behavior, also smoke-test with `cargo app` when practical.
- Packaging, icon, capability, or release changes: run `cargo app-build` when practical.

## Implementation Notes

- Use typed data structures and existing parser/serializer APIs for `.dpane` data instead of ad hoc string manipulation.
- Keep frontend components behind the wrappers in `ui/src/api`; avoid scattering raw Tauri `invoke` calls or event names through Vue components.
- When process behavior changes, cover paths with spaces, missing executables, invalid working directories, shell/command combinations, and auto-start behavior where relevant.
- When UI terminal behavior changes, manually verify resize, start, stop, output, and cleanup paths.
