use super::{Workspace, WorkspacePane};
use crate::config::{validate_config, DevPaneConfig};
use anyhow::Result;
use std::path::Path;

/// Builds a fully resolved runtime workspace from a loaded config.
///
/// # Errors
///
/// Returns an error if the config is invalid or any workspace path cannot be
/// resolved.
pub fn build_workspace(config_path: &Path, config: &DevPaneConfig) -> Result<Workspace> {
    validate_config(config, config_path)?;

    let root = config.workspace_root(config_path)?;
    let mut panes = Vec::with_capacity(config.panes.len());

    for (pane_id, pane) in &config.panes {
        panes.push(WorkspacePane {
            id: pane_id.clone(),
            name: pane.name.clone().unwrap_or_else(|| pane_id.clone()),
            cwd: DevPaneConfig::pane_cwd_in(&root, pane)?,
            shell: config.pane_shell(pane).to_string(),
            command: pane.command.clone(),
            auto_start: config.pane_auto_start(pane),
        });
    }

    panes.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(Workspace {
        name: config.name.clone(),
        root,
        scrollback: config.scrollback(),
        layout: config.layout.clone(),
        panes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LayoutNode, PaneConfig, Settings};
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

    #[test]
    fn build_workspace_resolves_runtime_pane_state() {
        let root = test_dir("workspace-builder");
        let pane_dir = root.join("app");
        fs::create_dir_all(&pane_dir).expect("pane directory should be created");

        let mut panes = HashMap::new();
        panes.insert(
            "app".to_string(),
            PaneConfig {
                name: Some("Application".to_string()),
                cwd: Some(PathBuf::from("app")),
                shell: None,
                command: Some("cargo run".to_string()),
                auto_start: Some(false),
            },
        );

        let config = DevPaneConfig {
            version: 1,
            name: "Test Workspace".to_string(),
            root: Some(root.clone()),
            settings: Some(Settings {
                shell: Some("pwsh".to_string()),
                auto_start: Some(true),
                scrollback: Some(2000),
            }),
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes,
        };

        let workspace = build_workspace(&root.join("workspace.dpane"), &config)
            .expect("workspace should build");

        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.scrollback, 2000);
        assert_eq!(
            workspace.root,
            root.canonicalize()
                .expect("workspace root should canonicalize")
        );
        assert_eq!(workspace.panes.len(), 1);
        assert_eq!(workspace.panes[0].id, "app");
        assert_eq!(workspace.panes[0].name, "Application");
        assert_eq!(
            workspace.panes[0].cwd,
            pane_dir
                .canonicalize()
                .expect("pane directory should canonicalize")
        );
        assert_eq!(workspace.panes[0].shell, "pwsh");
        assert_eq!(workspace.panes[0].command.as_deref(), Some("cargo run"));
        assert!(!workspace.panes[0].auto_start);
    }
}
