use super::{DevPaneConfig, LayoutNode};
use anyhow::{Result, bail};
use std::path::Path;

/// Validates a loaded `.dpane` configuration before the workspace is opened.
///
/// This function checks the rules that cannot be guaranteed by YAML
/// deserialization alone:
/// - the schema version is supported,
/// - the workspace has a name and at least one pane,
/// - the workspace root exists,
/// - every layout pane reference points to a declared pane,
/// - every pane working directory exists.
pub fn validate_config(config: &DevPaneConfig, config_path: &Path) -> Result<()> {
    if config.version != 1 {
        bail!("unsupported .dpane version: {}", config.version);
    }

    if config.name.trim().is_empty() {
        bail!("workspace name cannot be empty");
    }

    if config.panes.is_empty() {
        bail!("workspace must define at least one pane");
    }

    config.workspace_root(config_path)?;

    validate_layout_node(&config.layout, config)?;

    for (pane_id, pane) in &config.panes {
        config
            .pane_cwd(config_path, pane)
            .map_err(|error| error.context(format!("invalid cwd for pane '{}'", pane_id)))?;
    }

    Ok(())
}

/// Validates a recursive layout node and all of its descendants.
///
/// Split nodes must contain at least one child. Pane nodes must reference an
/// existing entry from `config.panes`.
fn validate_layout_node(node: &LayoutNode, config: &DevPaneConfig) -> Result<()> {
    match node {
        LayoutNode::Split { children, .. } => {
            if children.is_empty() {
                bail!("layout split must contain at least one child");
            }

            for child in children {
                validate_layout_node(child, config)?;
            }
        }
        LayoutNode::Pane { pane, .. } => {
            if !config.panes.contains_key(pane) {
                bail!("layout references unknown pane: {}", pane);
            }
        }
    }

    Ok(())
}
