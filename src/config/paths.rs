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
