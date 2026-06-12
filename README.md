# DevPane

DevPane is a Rust CLI and Tauri desktop app for loading `.dpane` YAML workspace files, validating them, resolving paths, building launch plans, and running terminal panes.

## Quick Start

Install dependencies once:

```powershell
cargo test --workspace
npm install --prefix ui
```

The desktop commands use Tauri's Cargo subcommand. If `cargo tauri` is missing:

```powershell
cargo install tauri-cli --version "^2" --locked
```

Use the portable example for local testing:

```powershell
examples/devpane.dpane
```

`examples/webclient.dpane` is a machine-specific sample and may not validate unless that workspace exists on your machine.

## CLI Modes

Run these from the repo root.

```powershell
cargo run -- validate examples/devpane.dpane
```

Validates the `.dpane` file. This checks schema version, workspace name, panes, layout references, split children, workspace root, and pane cwd paths. It does not run commands.

```powershell
cargo run -- inspect examples/devpane.dpane
```

Prints the resolved workspace: root path, scrollback, layout, pane cwd, shell, auto-start, and command.

```powershell
cargo run -- plan examples/devpane.dpane
```

Prints the auto-start process launch plan without starting processes.

```powershell
cargo run -- run examples/devpane.dpane
```

Starts auto-start panes in the CLI runner. This is not the final terminal UI. Press `Ctrl+C` to stop child processes.

## Desktop App

Run the Tauri app in development mode. This also starts the Vite frontend through Tauri's `beforeDevCommand`:

```powershell
cargo app
```

Equivalent long form:

```powershell
cargo tauri dev
```

The frontend dev server is pinned to `http://localhost:17777` to avoid the common Vite default port.

The desktop app opens on the session picker. Choose `New Session` to open a terminal workspace. The app uses a compact titlebar menu, similar to desktop editor menus:

- `File -> New Session` starts a fresh workspace.
- `File -> Sessions` returns to the recent-session picker.
- `File -> Save` asks for the workspace name and `.dpane` path.
- `Terminal -> Add Terminal` adds another live terminal pane to the active split.
- `Terminal -> Add Horizontal` splits the active pane into stacked panes.
- `Terminal -> Add Vertical` splits the active pane into side-by-side panes.
- In the session picker, use the `x` button or the `Delete` key on a recent workspace, then confirm in the app dialog to remove its `.dpane` file and recent-session entry.

Saving writes a `.dpane` file to the path you enter, such as:

```text
C:\Development\devpane\examples\devpane.dpane
```

Recent sessions load existing `.dpane` files back into the same terminal workspace screen. Each pane is an xterm terminal backed by a local PTY. Resize the app window or maximize it and the terminals refit automatically.
Drag the gutters between panes to resize neighboring terminals.

Build the frontend only:

```powershell
npm run build --prefix ui
```

Build the Tauri app:

```powershell
cargo app-build
```

Equivalent long form:

```powershell
cargo tauri build
```

## Development Checks

```powershell
cargo check-all
npm run build --prefix ui
cargo fmt --check
```

`cargo fmt --check` may report formatting in existing `src-tauri` files until those files are formatted.

## `.dpane` Basics

A workspace file defines:

- `version`: currently `1`
- `name`: display name
- `root`: workspace root; relative paths resolve from the `.dpane` file directory
- `settings`: defaults like `auto_start` and `scrollback`
- `layout`: pane/split layout tree
- `panes`: pane definitions keyed by id

Validation and inspection are safe: they must not run pane commands.
