use crate::workspace::WorkspacePane;
use std::path::PathBuf;

/// Resolved process launch plan for a pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessLaunch {
    /// Pane id that owns this launch plan.
    pub pane_id: String,

    /// Working directory used when the process starts.
    pub cwd: PathBuf,

    /// Program executable to start.
    pub program: String,

    /// Program arguments.
    pub args: Vec<String>,
}

/// Launch mode used when wrapping pane startup commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchMode {
    /// Keep the shell open after the startup command where the platform shell supports it.
    Interactive,

    /// Exit the shell when the startup command finishes.
    Headless,
}

/// Builds a process launch plan for a workspace pane.
pub fn build_launch_with_mode(pane: &WorkspacePane, mode: LaunchMode) -> ProcessLaunch {
    ProcessLaunch {
        pane_id: pane.id.clone(),
        cwd: pane.cwd.clone(),
        program: pane.shell.clone(),
        args: shell_args(&pane.shell, pane.command.as_deref(), mode),
    }
}

/// Returns the lowercase program name of a shell path, without extension.
pub fn shell_program_name(shell: &str) -> String {
    std::path::Path::new(shell)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| shell.to_lowercase())
}

/// Joins a multi-line startup command into a single shell statement.
///
/// `.dpane` pane commands may contain one command per line (recorded session
/// steps); each shell needs its own statement separator.
fn join_command_steps(command: &str, separator: &str) -> String {
    command
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

fn quote_shell_word(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

fn unix_command_args(shell: &str, command: Option<&str>, mode: LaunchMode) -> Vec<String> {
    match command.filter(|value| !value.trim().is_empty()) {
        Some(command) => {
            let command = join_command_steps(command, " && ");
            let command = match mode {
                LaunchMode::Interactive => {
                    format!("{}; exec {} -i", command, quote_shell_word(shell))
                }
                LaunchMode::Headless => command,
            };

            vec!["-lc".to_string(), command]
        }
        None => Vec::new(),
    }
}

#[cfg(windows)]
fn powershell_prompt_script() -> String {
    r#"function global:prompt { $location = $executionContext.SessionState.Path.CurrentLocation.Path; if ($location.StartsWith('\\?\')) { $location = $location.Substring(4) }; "[$env:DEVPANE_PANE_NAME] PS $location> " }"#.to_string()
}

#[cfg(windows)]
fn powershell_args(command: Option<&str>, mode: LaunchMode, pane_prompt: bool) -> Vec<String> {
    let command = command.filter(|value| !value.trim().is_empty());

    if !pane_prompt && command.is_none() {
        return Vec::new();
    }

    let mut args = vec!["-NoLogo".to_string()];

    if mode == LaunchMode::Interactive {
        args.push("-NoExit".to_string());
    }

    if pane_prompt || command.is_some() {
        let mut startup = String::new();

        if pane_prompt {
            startup.push_str(&powershell_prompt_script());
        }

        if let Some(command) = command {
            if !startup.is_empty() {
                startup.push_str("; ");
            }

            // Windows PowerShell 5.1 has no `&&`; `;` runs steps in order.
            startup.push_str(&join_command_steps(command, "; "));
        }

        args.push("-Command".to_string());
        args.push(startup);
    }

    args
}

#[cfg(windows)]
fn windows_command_args(
    shell: &str,
    command: Option<&str>,
    mode: LaunchMode,
    pane_prompt: bool,
) -> Vec<String> {
    match shell_program_name(shell).as_str() {
        "powershell" | "pwsh" => powershell_args(command, mode, pane_prompt),
        "cmd" => match command.filter(|value| !value.trim().is_empty()) {
            Some(command) => vec![
                match mode {
                    LaunchMode::Interactive => "/K",
                    LaunchMode::Headless => "/C",
                }
                .to_string(),
                join_command_steps(command, " && "),
            ],
            None => Vec::new(),
        },
        _ => unix_command_args(shell, command, mode),
    }
}

/// Builds shell arguments for a pane command in the requested launch mode.
pub fn shell_args(shell: &str, command: Option<&str>, mode: LaunchMode) -> Vec<String> {
    #[cfg(windows)]
    {
        windows_command_args(shell, command, mode, false)
    }

    #[cfg(not(windows))]
    {
        unix_command_args(shell, command, mode)
    }
}

/// Builds shell arguments for an interactive PTY terminal pane.
pub fn terminal_shell_args(shell: &str, command: Option<&str>) -> Vec<String> {
    #[cfg(windows)]
    {
        windows_command_args(shell, command, LaunchMode::Interactive, true)
    }

    #[cfg(not(windows))]
    {
        shell_args(shell, command, LaunchMode::Interactive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn test_shell() -> &'static str {
        "pwsh"
    }

    #[cfg(not(windows))]
    fn test_shell() -> &'static str {
        "zsh"
    }

    fn pane(command: Option<&str>) -> WorkspacePane {
        WorkspacePane {
            id: "app".to_string(),
            name: "App".to_string(),
            cwd: PathBuf::from("C:/workspace/app"),
            shell: test_shell().to_string(),
            command: command.map(str::to_string),
            auto_start: true,
        }
    }

    #[test]
    fn build_launch_uses_pane_shell_as_program() {
        let launch = build_launch_with_mode(&pane(None), LaunchMode::Interactive);

        assert_eq!(launch.pane_id, "app");
        assert_eq!(launch.program, test_shell());
        assert_eq!(launch.cwd, PathBuf::from("C:/workspace/app"));
        assert!(launch.args.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn build_launch_wraps_command_for_windows_shell() {
        let launch = build_launch_with_mode(&pane(Some("cargo run")), LaunchMode::Interactive);

        assert_eq!(
            launch.args,
            vec![
                "-NoLogo".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                "cargo run".to_string()
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn build_launch_joins_multi_step_command_for_windows_shell() {
        let launch = build_launch_with_mode(
            &pane(Some("cd web\nnpm install\n\nnpm run dev")),
            LaunchMode::Interactive,
        );

        assert_eq!(
            launch.args,
            vec![
                "-NoLogo".to_string(),
                "-NoExit".to_string(),
                "-Command".to_string(),
                "cd web; npm install; npm run dev".to_string()
            ]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_launch_joins_multi_step_command_for_unix_shell() {
        let launch = build_launch_with_mode(
            &pane(Some("cd web\nnpm install\n\nnpm run dev")),
            LaunchMode::Interactive,
        );

        assert_eq!(
            launch.args,
            vec![
                "-lc".to_string(),
                "cd web && npm install && npm run dev; exec 'zsh' -i".to_string()
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn build_headless_launch_exits_after_windows_command() {
        let launch = build_launch_with_mode(&pane(Some("cargo run")), LaunchMode::Headless);

        assert_eq!(
            launch.args,
            vec![
                "-NoLogo".to_string(),
                "-Command".to_string(),
                "cargo run".to_string()
            ]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_launch_wraps_command_for_unix_shell() {
        let launch = build_launch_with_mode(&pane(Some("cargo run")), LaunchMode::Interactive);

        assert_eq!(
            launch.args,
            vec!["-lc".to_string(), "cargo run; exec 'zsh' -i".to_string()]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_headless_launch_exits_after_unix_command() {
        let launch = build_launch_with_mode(&pane(Some("cargo run")), LaunchMode::Headless);

        assert_eq!(
            launch.args,
            vec!["-lc".to_string(), "cargo run".to_string()]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn shell_program_name_strips_path_and_extension() {
        assert_eq!(shell_program_name("/bin/zsh"), "zsh");
        assert_eq!(shell_program_name("C:/Windows/System32/cmd.exe"), "cmd");
    }
}
