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
        args: shell_args(pane.command.as_deref(), mode),
    }
}

#[cfg(windows)]
fn shell_args(command: Option<&str>, mode: LaunchMode) -> Vec<String> {
    match command {
        Some(command) => {
            let mut args = Vec::new();

            if mode == LaunchMode::Interactive {
                args.push("-NoExit".to_string());
            }

            args.push("-Command".to_string());
            args.push(command.to_string());
            args
        }
        None => Vec::new(),
    }
}

#[cfg(not(windows))]
fn shell_args(command: Option<&str>, _mode: LaunchMode) -> Vec<String> {
    match command {
        Some(command) => vec!["-lc".to_string(), command.to_string()],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pane(command: Option<&str>) -> WorkspacePane {
        WorkspacePane {
            id: "app".to_string(),
            name: "App".to_string(),
            cwd: PathBuf::from("C:/workspace/app"),
            shell: "pwsh".to_string(),
            command: command.map(str::to_string),
            auto_start: true,
        }
    }

    #[test]
    fn build_launch_uses_pane_shell_as_program() {
        let launch = build_launch(&pane(None));

        assert_eq!(launch.pane_id, "app");
        assert_eq!(launch.program, "pwsh");
        assert_eq!(launch.cwd, PathBuf::from("C:/workspace/app"));
        assert!(launch.args.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn build_launch_wraps_command_for_windows_shell() {
        let launch = build_launch(&pane(Some("cargo run")));

        assert_eq!(
            launch.args,
            vec![
                "-NoExit".to_string(),
                "-Command".to_string(),
                "cargo run".to_string()
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn build_headless_launch_exits_after_windows_command() {
        let launch = build_launch_with_mode(&pane(Some("cargo run")), LaunchMode::Headless);

        assert_eq!(
            launch.args,
            vec!["-Command".to_string(), "cargo run".to_string()]
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn build_launch_wraps_command_for_unix_shell() {
        let launch = build_launch(&pane(Some("cargo run")));

        assert_eq!(
            launch.args,
            vec!["-lc".to_string(), "cargo run".to_string()]
        );
    }
}
