use crate::process::launch::{build_launch_with_mode, LaunchMode, ProcessLaunch};
use crate::workspace::{PaneStatus, WorkspaceRuntime};
use anyhow::{bail, Result};

/// Coordinates pane lifecycle transitions.
///
/// The manager owns process lifecycle rules. Actual OS process or PTY spawning
/// will be added behind this API.
#[derive(Debug, Default)]
pub struct ProcessManager;

impl ProcessManager {
    /// Creates a process manager.
    pub fn new() -> ProcessManager {
        ProcessManager
    }

    /// Starts a pane lifecycle using the requested launch mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the pane id is unknown or if the pane is already
    /// starting or running.
    pub fn start_pane_with_mode(
        &self,
        runtime: &mut WorkspaceRuntime,
        pane_id: &str,
        mode: LaunchMode,
    ) -> Result<ProcessLaunch> {
        let pane = runtime
            .pane_mut(pane_id)
            .ok_or_else(|| anyhow::anyhow!("unknown pane: {}", pane_id))?;

        match pane.status {
            PaneStatus::Idle | PaneStatus::Exited { .. } | PaneStatus::Failed { .. } => {
                let launch = build_launch_with_mode(&pane.pane, mode);
                pane.status = PaneStatus::Starting;
                Ok(launch)
            }
            PaneStatus::Starting => bail!("pane '{}' is already starting", pane_id),
            PaneStatus::Running => bail!("pane '{}' is already running", pane_id),
        }
    }

    /// Marks a pane as running after successful startup.
    ///
    /// # Errors
    ///
    /// Returns an error if the pane id is unknown or the pane is not starting.
    pub fn mark_running(&self, runtime: &mut WorkspaceRuntime, pane_id: &str) -> Result<()> {
        let pane = runtime
            .pane_mut(pane_id)
            .ok_or_else(|| anyhow::anyhow!("unknown pane: {}", pane_id))?;

        if pane.status != PaneStatus::Starting {
            bail!("pane '{}' is not starting", pane_id);
        }

        pane.status = PaneStatus::Running;
        Ok(())
    }

    /// Marks a pane as exited.
    ///
    /// # Errors
    ///
    /// Returns an error if the pane id is unknown.
    pub fn mark_exited(
        &self,
        runtime: &mut WorkspaceRuntime,
        pane_id: &str,
        code: Option<i32>,
    ) -> Result<()> {
        let pane = runtime
            .pane_mut(pane_id)
            .ok_or_else(|| anyhow::anyhow!("unknown pane: {}", pane_id))?;

        pane.status = PaneStatus::Exited { code };
        Ok(())
    }

    /// Marks a pane as failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the pane id is unknown.
    pub fn mark_failed(
        &self,
        runtime: &mut WorkspaceRuntime,
        pane_id: &str,
        message: impl Into<String>,
    ) -> Result<()> {
        let pane = runtime
            .pane_mut(pane_id)
            .ok_or_else(|| anyhow::anyhow!("unknown pane: {}", pane_id))?;

        pane.status = PaneStatus::Failed {
            message: message.into(),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LayoutNode;
    use crate::workspace::{Workspace, WorkspacePane};
    use std::path::PathBuf;

    fn runtime() -> WorkspaceRuntime {
        WorkspaceRuntime::new(Workspace {
            name: "Test Workspace".to_string(),
            root: PathBuf::from("C:/workspace"),
            scrollback: 1000,
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes: vec![WorkspacePane {
                id: "app".to_string(),
                name: "App".to_string(),
                cwd: PathBuf::from("C:/workspace/app"),
                shell: "pwsh".to_string(),
                command: Some("cargo run".to_string()),
                auto_start: true,
            }],
        })
    }

    fn pane_status(runtime: &WorkspaceRuntime, pane_id: &str) -> PaneStatus {
        runtime
            .panes
            .iter()
            .find(|pane| pane.pane.id == pane_id)
            .expect("pane should exist")
            .status
            .clone()
    }

    #[test]
    fn start_pane_moves_idle_pane_to_starting() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect("pane should start");

        assert_eq!(pane_status(&runtime, "app"), PaneStatus::Starting);
    }

    #[test]
    fn start_pane_rejects_running_pane() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect("pane should start");
        manager
            .mark_running(&mut runtime, "app")
            .expect("pane should become running");

        let error = manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect_err("running pane should not start again");

        assert!(
            error.to_string().contains("already running"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn mark_running_requires_starting_status() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        let error = manager
            .mark_running(&mut runtime, "app")
            .expect_err("idle pane should not become running directly");

        assert!(
            error.to_string().contains("is not starting"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn exited_pane_can_be_started_again() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        manager
            .mark_exited(&mut runtime, "app", Some(0))
            .expect("pane should be marked exited");
        manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect("exited pane should start again");

        assert_eq!(pane_status(&runtime, "app"), PaneStatus::Starting);
    }

    #[test]
    fn unknown_pane_returns_error() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        let error = manager
            .start_pane_with_mode(&mut runtime, "missing", LaunchMode::Interactive)
            .expect_err("unknown pane should fail");

        assert!(
            error.to_string().contains("unknown pane"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn failed_pane_can_be_started_again() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        manager
            .mark_failed(&mut runtime, "app", "spawn failed")
            .expect("pane should be marked failed");
        manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect("failed pane should start again");

        assert_eq!(pane_status(&runtime, "app"), PaneStatus::Starting);
    }

    #[test]
    fn start_pane_returns_launch_plan() {
        let manager = ProcessManager::new();
        let mut runtime = runtime();

        let launch = manager
            .start_pane_with_mode(&mut runtime, "app", LaunchMode::Interactive)
            .expect("pane should start");

        assert_eq!(launch.pane_id, "app");
        assert_eq!(launch.program, "pwsh");
        assert_eq!(launch.cwd, PathBuf::from("C:/workspace/app"));
    }
}
