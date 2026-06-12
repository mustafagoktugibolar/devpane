use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root workspace configuration loaded from a `.dpane` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Recursive layout node.
///
/// A node can either be a split container or a terminal pane reference.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum LayoutNode {
    /// Split container with child layout nodes.
    Split {
        direction: SplitDirection,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u16>,
        children: Vec<LayoutNode>,
    },

    /// Reference to a pane defined in the `panes` section.
    Pane {
        pane: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u16>,
    },
}

impl<'de> Deserialize<'de> for LayoutNode {
    /// Deserializes a layout node with clear errors instead of the generic
    /// "did not match any variant" message an untagged enum would produce.
    fn deserialize<D>(deserializer: D) -> Result<LayoutNode, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawLayoutNode {
            pane: Option<String>,
            direction: Option<SplitDirection>,
            children: Option<Vec<LayoutNode>>,
            size: Option<u16>,
        }

        let raw = RawLayoutNode::deserialize(deserializer)?;

        match (raw.pane, raw.direction, raw.children) {
            (Some(pane), None, None) => Ok(LayoutNode::Pane {
                pane,
                size: raw.size,
            }),
            (None, Some(direction), Some(children)) => Ok(LayoutNode::Split {
                direction,
                size: raw.size,
                children,
            }),
            (Some(_), _, _) => Err(D::Error::custom(
                "layout pane node cannot also define `direction` or `children`",
            )),
            (None, Some(_), None) => Err(D::Error::custom("layout split node requires `children`")),
            (None, None, Some(_)) => {
                Err(D::Error::custom("layout split node requires `direction`"))
            }
            (None, None, None) => Err(D::Error::custom(
                "layout node must define either `pane` or `direction` and `children`",
            )),
        }
    }
}
