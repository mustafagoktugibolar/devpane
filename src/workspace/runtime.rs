use super::{Workspace, WorkspacePane};

/// Runtime status for a terminal pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneStatus {
    /// Pane process has not started.
    Idle,

    /// Pane process is being started.
    Starting,

    /// Pane process is running.
    #[allow(dead_code)]
    Running,

    /// Pane process exited.
    #[allow(dead_code)]
    Exited {
        /// Process exit code, if the platform provides one.
        code: Option<i32>,
    },

    /// Pane process failed before or during execution.
    #[allow(dead_code)]
    Failed {
        /// Human readable failure message.
        message: String,
    },
}

/// Runtime state for a terminal pane.
#[derive(Debug, Clone)]
pub struct PaneRuntime {
    /// Resolved pane definition.
    pub pane: WorkspacePane,

    /// Current runtime status.
    pub status: PaneStatus,
}

/// Runtime state for an opened workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceRuntime {
    /// Resolved workspace definition.
    pub workspace: Workspace,

    /// Runtime state for each pane.
    pub panes: Vec<PaneRuntime>,
}

impl WorkspaceRuntime {
    /// Creates runtime state for a resolved workspace.
    ///
    /// All panes start as `Idle`. Process management is responsible for moving
    /// panes into `Starting`, `Running`, `Exited`, or `Failed`.
    pub fn new(workspace: Workspace) -> WorkspaceRuntime {
        let panes = workspace
            .panes
            .iter()
            .cloned()
            .map(|pane| PaneRuntime {
                pane,
                status: PaneStatus::Idle,
            })
            .collect();

        WorkspaceRuntime { workspace, panes }
    }

    /// Returns the runtime state for a pane id.
    #[allow(dead_code)]
    pub fn pane(&self, pane_id: &str) -> Option<&PaneRuntime> {
        self.panes.iter().find(|pane| pane.pane.id == pane_id)
    }

    /// Returns mutable runtime state for a pane id.
    pub fn pane_mut(&mut self, pane_id: &str) -> Option<&mut PaneRuntime> {
        self.panes.iter_mut().find(|pane| pane.pane.id == pane_id)
    }

    /// Returns the workspace name for this runtime.
    pub fn workspace_name(&self) -> &str {
        &self.workspace.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LayoutNode;
    use std::path::PathBuf;

    fn workspace() -> Workspace {
        Workspace {
            name: "Test Workspace".to_string(),
            root: PathBuf::from("C:/workspace"),
            scrollback: 1000,
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes: vec![
                WorkspacePane {
                    id: "app".to_string(),
                    name: "App".to_string(),
                    cwd: PathBuf::from("C:/workspace/app"),
                    shell: "pwsh".to_string(),
                    command: Some("cargo run".to_string()),
                    auto_start: true,
                },
                WorkspacePane {
                    id: "worker".to_string(),
                    name: "Worker".to_string(),
                    cwd: PathBuf::from("C:/workspace/worker"),
                    shell: "pwsh".to_string(),
                    command: None,
                    auto_start: false,
                },
            ],
        }
    }

    #[test]
    fn workspace_runtime_starts_all_panes_idle() {
        let runtime = WorkspaceRuntime::new(workspace());

        assert_eq!(runtime.workspace_name(), "Test Workspace");
        assert_eq!(runtime.panes.len(), 2);
        assert!(
            runtime
                .panes
                .iter()
                .all(|pane| pane.status == PaneStatus::Idle)
        );
    }

    #[test]
    fn pane_mut_updates_status_for_matching_pane() {
        let mut runtime = WorkspaceRuntime::new(workspace());

        runtime
            .pane_mut("app")
            .expect("app pane should exist")
            .status = PaneStatus::Starting;

        assert_eq!(
            runtime.pane("app").expect("app pane should exist").status,
            PaneStatus::Starting
        );
        assert_eq!(
            runtime
                .pane("worker")
                .expect("worker pane should exist")
                .status,
            PaneStatus::Idle
        );
    }
}
