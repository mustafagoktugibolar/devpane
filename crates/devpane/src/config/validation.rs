use super::{DevPaneConfig, LayoutNode};
use anyhow::{bail, Result};
use std::collections::HashSet;
use std::path::Path;

/// Validates a loaded `.dpane` configuration before the workspace is opened.
///
/// This function checks the rules that cannot be guaranteed by YAML
/// deserialization alone:
/// - the schema version is supported,
/// - the workspace has a name and at least one pane,
/// - the workspace root exists,
/// - every layout pane reference points to a declared pane,
/// - no pane is referenced more than once in the layout,
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

    let workspace_root = config.workspace_root(config_path)?;

    let mut seen_panes = HashSet::new();
    validate_layout_node(&config.layout, config, &mut seen_panes)?;

    for (pane_id, pane) in &config.panes {
        DevPaneConfig::pane_cwd_in(&workspace_root, pane)
            .map_err(|error| error.context(format!("invalid cwd for pane '{}'", pane_id)))?;
    }

    Ok(())
}

/// Validates a recursive layout node and all of its descendants.
///
/// Split nodes must contain at least one child. Pane nodes must reference an
/// existing entry from `config.panes`, and each pane may appear at most once
/// in the layout.
fn validate_layout_node(
    node: &LayoutNode,
    config: &DevPaneConfig,
    seen_panes: &mut HashSet<String>,
) -> Result<()> {
    match node {
        LayoutNode::Split { children, .. } => {
            if children.is_empty() {
                bail!("layout split must contain at least one child");
            }

            for child in children {
                validate_layout_node(child, config, seen_panes)?;
            }
        }
        LayoutNode::Pane { pane, .. } => {
            if !config.panes.contains_key(pane) {
                bail!("layout references unknown pane: {}", pane);
            }

            if !seen_panes.insert(pane.clone()) {
                bail!("layout references pane '{}' more than once", pane);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PaneConfig, SplitDirection};
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("devpane-{name}-{unique}"))
    }

    fn valid_config(root: PathBuf) -> DevPaneConfig {
        let mut panes = HashMap::new();
        panes.insert(
            "app".to_string(),
            PaneConfig {
                name: Some("App".to_string()),
                cwd: None,
                shell: None,
                command: Some("cargo run".to_string()),
                auto_start: None,
            },
        );

        DevPaneConfig {
            version: 1,
            name: "Test Workspace".to_string(),
            root: Some(root),
            settings: None,
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes,
        }
    }

    #[test]
    fn validate_config_accepts_valid_workspace() {
        let root = test_dir("valid-workspace");
        fs::create_dir_all(&root).expect("workspace root should be created");
        let config_path = root.join("workspace.dpane");
        let config = valid_config(root);

        validate_config(&config, &config_path).expect("valid config should pass validation");
    }

    #[test]
    fn validate_config_rejects_unknown_layout_pane() {
        let root = test_dir("unknown-pane");
        fs::create_dir_all(&root).expect("workspace root should be created");
        let config_path = root.join("workspace.dpane");
        let mut config = valid_config(root);
        config.layout = LayoutNode::Pane {
            pane: "missing".to_string(),
            size: None,
        };

        let error = validate_config(&config, &config_path).expect_err("unknown pane should fail");

        assert!(
            error.to_string().contains("layout references unknown pane"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn validate_config_rejects_duplicate_layout_pane() {
        let root = test_dir("duplicate-pane");
        fs::create_dir_all(&root).expect("workspace root should be created");
        let config_path = root.join("workspace.dpane");
        let mut config = valid_config(root);
        config.layout = LayoutNode::Split {
            direction: SplitDirection::Horizontal,
            size: None,
            children: vec![
                LayoutNode::Pane {
                    pane: "app".to_string(),
                    size: None,
                },
                LayoutNode::Pane {
                    pane: "app".to_string(),
                    size: None,
                },
            ],
        };

        let error = validate_config(&config, &config_path).expect_err("duplicate pane should fail");

        assert!(
            error
                .to_string()
                .contains("layout references pane 'app' more than once"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn validate_config_rejects_empty_split() {
        let root = test_dir("empty-split");
        fs::create_dir_all(&root).expect("workspace root should be created");
        let config_path = root.join("workspace.dpane");
        let mut config = valid_config(root);
        config.layout = LayoutNode::Split {
            direction: SplitDirection::Horizontal,
            size: None,
            children: Vec::new(),
        };

        let error = validate_config(&config, &config_path).expect_err("empty split should fail");

        assert!(
            error
                .to_string()
                .contains("layout split must contain at least one child"),
            "unexpected error: {error:#}"
        );
    }
}
