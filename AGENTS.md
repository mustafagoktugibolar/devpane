# Repository Guidelines

## Project Structure & Module Organization

`devpane` is a Rust CLI for reading `.dpane` YAML workspace files, validating them, resolving paths, building launch plans, and managing pane process lifecycles. Source lives in `src/`: `cli.rs` defines Clap commands, `main.rs` wires execution, and `output.rs` contains pure formatting helpers. Domain modules are split into `src/config/` for YAML models, validation, settings, and path resolution; `src/workspace/` for resolved workspace and runtime state; and `src/process/` for launch planning, lifecycle management, and process spawning. Example workspace files live in `examples/`, currently `examples/webclient.dpane`.

## Build, Test, and Development Commands

- `cargo build`: compile the binary in debug mode.
- `cargo test`: run all unit tests.
- `cargo test <test_name>`: run tests matching a specific name.
- `cargo run -- validate examples/webclient.dpane`: validate an example config.
- `cargo run -- inspect examples/webclient.dpane`: print resolved workspace details.
- `cargo run -- plan examples/webclient.dpane`: show process launch plans.
- `cargo run -- run examples/webclient.dpane`: execute auto-start panes.
- `cargo fmt`: format Rust code before committing.
- `cargo clippy`: run lint checks when available.

## Coding Style & Naming Conventions

Use standard Rust formatting from `rustfmt` with 4-space indentation. Prefer small modules with explicit responsibilities matching the existing `config`, `workspace`, and `process` boundaries. Use `snake_case` for functions, variables, and module files; `PascalCase` for structs and enums; and descriptive enum variants such as `PaneStatus::Running`. Keep formatting logic in `output.rs` side-effect free.

## Testing Guidelines

Tests are inline unit tests under `mod tests` in the relevant source file. Add tests near the behavior being changed, especially for config validation, path resolution, process launch arguments, runtime transitions, and CLI parsing. Use focused test names that describe the expected behavior, for example `pane_shell_prefers_pane_override`. Run `cargo test` before submitting changes.

## Commit & Pull Request Guidelines

Recent commit history uses short imperative messages such as `Add launch plan command` and `Add process lifecycle manager`. Follow that style: start with a verb, keep the subject concise, and describe one logical change. Pull requests should include a brief summary, tests run, and any behavior changes to `.dpane` parsing or command output. Include terminal output snippets or screenshots only when they clarify CLI behavior.

## Security & Configuration Tips

Treat `.dpane` commands as executable user input. Avoid adding behavior that silently runs panes during validation, inspection, or planning. Keep example configs minimal and avoid committing machine-specific absolute paths or secrets.
