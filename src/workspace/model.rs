use crate::config::LayoutNode;
use std::path::PathBuf;

/// Fully resolved workspace state used by runtime components.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Human readable workspace name.
    pub name: String,

    /// Canonical workspace root directory.
    pub root: PathBuf,

    /// Number of terminal lines kept in memory.
    pub scrollback: u32,

    /// Resolved layout tree.
    pub layout: LayoutNode,

    /// Resolved terminal panes.
    pub panes: Vec<WorkspacePane>,
}

/// Fully resolved terminal pane state.
#[derive(Debug, Clone)]
pub struct WorkspacePane {
    /// Stable pane id from the `.dpane` file.
    pub id: String,

    /// Human readable pane name.
    pub name: String,

    /// Canonical working directory.
    pub cwd: PathBuf,

    /// Shell executable used for this pane.
    pub shell: String,

    /// Optional startup command.
    pub command: Option<String>,

    /// Whether this pane should start automatically.
    pub auto_start: bool,
}
