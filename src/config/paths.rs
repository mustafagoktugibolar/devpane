use super::{DevPaneConfig, PaneConfig};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

impl DevPaneConfig {
    /// Loads and deserializes a `.dpane` file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or the YAML content cannot
    /// be deserialized into the expected config schema.
    pub fn load_from_file(path: &Path) -> Result<DevPaneConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let config: DevPaneConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Resolves the workspace root directory for this configuration.
    ///
    /// Resolution rules:
    /// - absolute `root` values are used as-is,
    /// - relative `root` values are resolved from the `.dpane` file directory,
    /// - missing `root` values default to the `.dpane` file directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the resolved workspace root does not exist or cannot
    /// be converted to a canonical path.
    pub fn workspace_root(&self, config_path: &Path) -> Result<PathBuf> {
        let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

        let root = match &self.root {
            Some(root) if root.is_absolute() => root.clone(),
            Some(root) => config_dir.join(root),
            None => config_dir.to_path_buf(),
        };

        root.canonicalize()
            .with_context(|| format!("failed to resolve workspace root: {}", root.display()))
    }

    /// Resolves the working directory for a pane.
    ///
    /// Absolute pane `cwd` values are used as-is. Relative `cwd` values are
    /// resolved from the workspace root. Missing `cwd` values default to the
    /// workspace root.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace root or resolved pane working
    /// directory does not exist or cannot be converted to a canonical path.
    pub fn pane_cwd(&self, config_path: &Path, pane: &PaneConfig) -> Result<PathBuf> {
        let workspace_root = self.workspace_root(config_path)?;

        let cwd = match &pane.cwd {
            Some(cwd) if cwd.is_absolute() => cwd.clone(),
            Some(cwd) => workspace_root.join(cwd),
            None => workspace_root,
        };

        cwd.canonicalize()
            .with_context(|| format!("failed to resolve pane cwd: {}", cwd.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LayoutNode;
    use std::collections::HashMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after UNIX epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("devpane-{name}-{unique}"))
    }

    fn config_with_root(root: Option<PathBuf>) -> DevPaneConfig {
        DevPaneConfig {
            version: 1,
            name: "Test Workspace".to_string(),
            root,
            settings: None,
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes: HashMap::new(),
        }
    }

    #[test]
    fn workspace_root_resolves_relative_root_from_config_directory() {
        let root = test_dir("relative-root");
        let config_dir = root.join("examples");
        let workspace_dir = root.join("workspace");
        fs::create_dir_all(&config_dir).expect("config directory should be created");
        fs::create_dir_all(&workspace_dir).expect("workspace directory should be created");

        let config_path = config_dir.join("workspace.dpane");
        let config = config_with_root(Some(PathBuf::from("../workspace")));

        let resolved = config
            .workspace_root(&config_path)
            .expect("workspace root should resolve");

        assert_eq!(
            resolved,
            workspace_dir
                .canonicalize()
                .expect("workspace directory should canonicalize")
        );
    }

    #[test]
    fn pane_cwd_resolves_relative_cwd_from_workspace_root() {
        let root = test_dir("pane-cwd");
        let config_dir = root.join("examples");
        let workspace_dir = root.join("workspace");
        let pane_dir = workspace_dir.join("src");
        fs::create_dir_all(&config_dir).expect("config directory should be created");
        fs::create_dir_all(&pane_dir).expect("pane directory should be created");

        let config_path = config_dir.join("workspace.dpane");
        let config = config_with_root(Some(PathBuf::from("../workspace")));
        let pane = PaneConfig {
            name: None,
            cwd: Some(PathBuf::from("src")),
            shell: None,
            command: None,
            auto_start: None,
        };

        let resolved = config
            .pane_cwd(&config_path, &pane)
            .expect("pane cwd should resolve");

        assert_eq!(
            resolved,
            pane_dir
                .canonicalize()
                .expect("pane directory should canonicalize")
        );
    }
}
