# DevPane Development Plan

## Current Status
DevPane has moved past the original backend learning milestones. The Rust CLI can load, validate, inspect, plan, and run `.dpane` files. Workspace resolution, config validation, process launch planning, runtime state transitions, and CLI parsing all have focused unit coverage.

The desktop app has been converted to Vue and now opens a real terminal workspace. The first screen is a session picker, `New Session` opens a live terminal pane immediately, titlebar menus handle session and terminal commands, and each pane is backed by a local PTY through Tauri commands/events.

## Completed
- Config model and YAML loading are split into `src/config`.
- Validation checks schema version, workspace name, pane definitions, layout references, split children, root paths, and pane cwd paths.
- Path resolution handles config-relative workspace roots and workspace-relative pane cwd values.
- CLI commands exist for `validate`, `inspect`, `plan`, and `run`.
- Process launch planning and lifecycle transitions are implemented for auto-start panes.
- Tauri scaffolding exists with recent session persistence and a `load_workspace` command.
- Vue frontend shell is in place with session picker, titlebar menus, save dialog, and responsive terminal grid.
- Tauri terminal commands exist for starting, writing to, resizing, and stopping PTY-backed terminals.
- Multiple terminals can be added without replacing existing panes.
- The custom titlebar supports menu commands, drag, resize hitboxes, and window controls.
- Workspaces can be saved from the terminal screen into new `.dpane` files.

## Active Goal: Terminal Workspace Polish
Tighten the first real terminal workflow:
- Manually verify drag, resize, maximize, close, and terminal refit behavior in the running Tauri app.
- Improve pane layout persistence so saved `.dpane` files preserve the on-screen layout more accurately than the current simple split.
- Add explicit pane command editing without bringing back a separate editor/sidebar surface.
- Add better terminal lifecycle state in the UI, such as start failure and exited process indicators.
- Consider keyboard shortcuts for menu actions once the basic mouse workflow is stable.

## Next Goal: App QA Pass
After the terminal workspace is usable in manual testing:
- Run the app with `cargo app`.
- Check `File -> New Session`, `File -> Sessions`, `File -> Save`, and `Terminal -> Add Terminal`.
- Check that adding terminals keeps existing terminal sessions alive.
- Check that maximize and manual resize refit all terminals.
- Check that saved workspaces reopen from recent sessions.

## Verification
- `cargo test --workspace`
- `npm install` when `ui/node_modules` is missing
- `npm run build --prefix ui`
- Manual UI checks:
  - valid `.dpane` path loads and starts terminal panes
  - invalid path or invalid config shows an error
  - successful load is added to recent sessions
  - reopening a recent session loads the workspace again
  - adding a terminal does not remove existing terminals
  - resizing or maximizing the window refits all terminals
