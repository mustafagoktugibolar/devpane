use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Root workspace configuration loaded from a `.dpane` file.
#[derive(Debug, Clone, Deserialize)]
pub struct DevPaneConfig {
    /// Configuration schema version.
    pub version: u16,

    /// Human readable workspace name.
    pub name: String,

    /// Workspace root directory.
    ///
    /// If omitted, the directory containing the `.dpane` file is used.
    pub root: Option<PathBuf>,

    /// Global default settings applied to panes.
    pub settings: Option<Settings>,

    /// Workspace layout definition.
    pub layout: LayoutNode,

    /// Registered terminal panes keyed by pane id.
    pub panes: HashMap<String, PaneConfig>,
}

/// Global workspace settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Default shell used by panes.
    ///
    /// Examples: `pwsh`, `powershell`, `cmd`.
    pub shell: Option<String>,

    /// Whether panes should start automatically.
    pub auto_start: Option<bool>,

    /// Maximum number of terminal lines kept in memory.
    pub scrollback: Option<u32>,
}

/// Terminal pane definition.
#[derive(Debug, Clone, Deserialize)]
pub struct PaneConfig {
    /// Human readable pane name.
    pub name: Option<String>,

    /// Working directory of the pane.
    pub cwd: Option<PathBuf>,

    /// Shell used by this pane.
    ///
    /// Overrides `settings.shell`.
    pub shell: Option<String>,

    /// Startup command executed when the pane starts.
    pub command: Option<String>,

    /// Whether this pane should start automatically.
    ///
    /// Overrides `settings.auto_start`.
    pub auto_start: Option<bool>,
}

/// Split direction for a layout container.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Recursive layout node.
///
/// A node can either be a split container or a terminal pane reference.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LayoutNode {
    /// Split container with child layout nodes.
    Split {
        direction: SplitDirection,
        size: Option<u16>,
        children: Vec<LayoutNode>,
    },

    /// Reference to a pane defined in the `panes` section.
    Pane { pane: String, size: Option<u16> },
}
