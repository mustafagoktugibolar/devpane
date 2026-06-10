use super::{DevPaneConfig, PaneConfig};

/// Default shell used when neither the pane nor global settings define one.
#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "pwsh";

/// Default shell used when neither the pane nor global settings define one.
#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

/// Default pane auto-start behavior.
pub const DEFAULT_AUTO_START: bool = true;

/// Default number of terminal lines kept in memory.
pub const DEFAULT_SCROLLBACK: u32 = 1000;

impl DevPaneConfig {
    /// Resolves the shell used by a pane.
    ///
    /// Precedence:
    /// - pane-level `shell`,
    /// - global `settings.shell`,
    /// - platform default shell.
    pub fn pane_shell<'a>(&'a self, pane: &'a PaneConfig) -> &'a str {
        pane.shell
            .as_deref()
            .or_else(|| {
                self.settings
                    .as_ref()
                    .and_then(|settings| settings.shell.as_deref())
            })
            .unwrap_or(DEFAULT_SHELL)
    }

    /// Resolves whether a pane should start automatically.
    ///
    /// Precedence:
    /// - pane-level `auto_start`,
    /// - global `settings.auto_start`,
    /// - `DEFAULT_AUTO_START`.
    pub fn pane_auto_start(&self, pane: &PaneConfig) -> bool {
        pane.auto_start
            .or_else(|| {
                self.settings
                    .as_ref()
                    .and_then(|settings| settings.auto_start)
            })
            .unwrap_or(DEFAULT_AUTO_START)
    }

    /// Resolves the configured terminal scrollback length.
    ///
    /// If `settings.scrollback` is missing, `DEFAULT_SCROLLBACK` is used.
    pub fn scrollback(&self) -> u32 {
        self.settings
            .as_ref()
            .and_then(|settings| settings.scrollback)
            .unwrap_or(DEFAULT_SCROLLBACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LayoutNode, Settings};
    use std::collections::HashMap;

    fn pane() -> PaneConfig {
        PaneConfig {
            name: None,
            cwd: None,
            shell: None,
            command: None,
            auto_start: None,
        }
    }

    fn config(settings: Option<Settings>) -> DevPaneConfig {
        DevPaneConfig {
            version: 1,
            name: "Test Workspace".to_string(),
            root: None,
            settings,
            layout: LayoutNode::Pane {
                pane: "app".to_string(),
                size: None,
            },
            panes: HashMap::new(),
        }
    }

    #[test]
    fn pane_shell_prefers_pane_shell_over_global_setting() {
        let config = config(Some(Settings {
            shell: Some("pwsh".to_string()),
            auto_start: None,
            scrollback: None,
        }));
        let pane = PaneConfig {
            shell: Some("cmd".to_string()),
            ..pane()
        };

        assert_eq!(config.pane_shell(&pane), "cmd");
    }

    #[test]
    fn pane_shell_uses_global_setting_when_pane_shell_is_missing() {
        let config = config(Some(Settings {
            shell: Some("pwsh".to_string()),
            auto_start: None,
            scrollback: None,
        }));

        assert_eq!(config.pane_shell(&pane()), "pwsh");
    }

    #[test]
    fn pane_auto_start_prefers_pane_value_over_global_setting() {
        let config = config(Some(Settings {
            shell: None,
            auto_start: Some(true),
            scrollback: None,
        }));
        let pane = PaneConfig {
            auto_start: Some(false),
            ..pane()
        };

        assert!(!config.pane_auto_start(&pane));
    }

    #[test]
    fn scrollback_uses_global_setting_when_present() {
        let config = config(Some(Settings {
            shell: None,
            auto_start: None,
            scrollback: Some(2500),
        }));

        assert_eq!(config.scrollback(), 2500);
    }

    #[test]
    fn scrollback_uses_default_when_missing() {
        let config = config(None);

        assert_eq!(config.scrollback(), DEFAULT_SCROLLBACK);
    }
}
