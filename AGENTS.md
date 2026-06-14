# Repository Guidelines

## Project Structure & Module Organization

DevPane is a Rust workspace with a Tauri/Vue desktop UI. Core Rust code lives in `crates/devpane/src`, organized by domain modules such as `config`, `process`, and `workspace`. The CLI entry point is in `crates/cli/src`. Tauri backend code and app configuration are in `src-tauri/`, with icons under `src-tauri/icons` and capabilities under `src-tauri/capabilities`. The Vue frontend is in `ui/src`, split into `api/`, `components/`, and shared files. Example `.dpane` workspaces are in `examples/`.

## Build, Test, and Development Commands

Run commands from the repository root unless noted.

- `cargo test --workspace` or `cargo check-all`: runs Rust tests for all workspace members.
- `cargo run -- validate examples/devpane.dpane`: validates an example workspace without running pane commands.
- `cargo run -- inspect examples/devpane.dpane`: prints resolved workspace details.
- `npm install --prefix ui`: installs frontend dependencies.
- `npm run build --prefix ui`: type-checks and builds the frontend.
- `cargo app`: starts the Tauri desktop app in development mode; this also starts Vite.
- `cargo app-build`: builds the packaged Tauri app.
- `cargo fmt --check`: checks Rust formatting.

## Coding Style & Naming Conventions

Use `rustfmt` defaults for Rust and keep modules focused around existing domain boundaries. Prefer clear names for validation, launch planning, and workspace resolution code. Rust tests are usually colocated in `mod tests` blocks. Frontend code uses Vue 3 single-file components with PascalCase component filenames, TypeScript modules in camelCase, and API wrappers under `ui/src/api`.

## Testing Guidelines

Add or update Rust unit tests near the code under test, especially for `.dpane` parsing, validation, path resolution, process planning, and workspace behavior. Use `cargo test --workspace` before opening a PR. For frontend changes, run `npm run build --prefix ui`; there is currently no separate frontend test command in `package.json`.

## Commit & Pull Request Guidelines

Use conventional commits when possible, matching existing history and `CONTRIBUTING.md`: `feat: add workspace persistence`, `fix: resolve pane resize issue`, `docs: update examples`. Keep each PR focused on one change, include a clear description, link related issues when applicable, and update docs or examples for user-facing behavior. UI changes should include screenshots or a short visual summary. All relevant Rust and frontend checks should pass before review.

## Security & Configuration Tips

Do not commit machine-specific workspace paths except in clearly documented examples. `examples/webclient.dpane` may not validate on every machine; prefer `examples/devpane.dpane` for portable checks. Report vulnerabilities through the process in `SECURITY.md`.
