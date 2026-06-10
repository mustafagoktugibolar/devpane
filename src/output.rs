use crate::config::{DevPaneConfig, LayoutNode, SplitDirection};
use anyhow::Result;
use std::fmt::Write;
use std::path::Path;

/// Formats the success output for a validated `.dpane` file.
pub fn format_validation_success(config_path: &Path, config: &DevPaneConfig) -> String {
    format!(
        "Valid .dpane file: {}\nWorkspace: {}\nPanes: {}\n",
        config_path.display(),
        config.name,
        config.panes.len()
    )
}

/// Formats a resolved inspection summary for a `.dpane` file.
///
/// # Errors
///
/// Returns an error if workspace or pane paths cannot be resolved.
pub fn format_inspection(config_path: &Path, config: &DevPaneConfig) -> Result<String> {
    let mut output = String::new();
    let workspace_root = config.workspace_root(config_path)?;

    writeln!(output, "Workspace: {}", config.name)?;
    writeln!(output, "Version: {}", config.version)?;
    writeln!(output, "Workspace root: {}", workspace_root.display())?;
    writeln!(output, "Scrollback: {}", config.scrollback())?;
    writeln!(output)?;
    writeln!(output, "Layout:")?;
    format_layout_node(&mut output, &config.layout, 0)?;
    writeln!(output)?;
    writeln!(output, "Panes:")?;

    for (pane_id, pane) in &config.panes {
        let pane_name = pane.name.as_deref().unwrap_or(pane_id);
        let pane_cwd = config.pane_cwd(config_path, pane)?;
        let pane_shell = config.pane_shell(pane);
        let pane_auto_start = config.pane_auto_start(pane);
        let command = pane.command.as_deref().unwrap_or("<no command>");

        writeln!(output, "- {}", pane_id)?;
        writeln!(output, "  name: {}", pane_name)?;
        writeln!(output, "  cwd: {}", pane_cwd.display())?;
        writeln!(output, "  shell: {}", pane_shell)?;
        writeln!(output, "  auto_start: {}", pane_auto_start)?;
        writeln!(output, "  command: {}", command)?;
    }

    Ok(output)
}

fn format_layout_node(output: &mut String, node: &LayoutNode, depth: usize) -> Result<()> {
    let indent = "  ".repeat(depth);

    match node {
        LayoutNode::Split {
            direction,
            size,
            children,
        } => {
            writeln!(
                output,
                "{}- split {}{}",
                indent,
                format_direction(direction),
                format_size(*size)
            )?;

            for child in children {
                format_layout_node(output, child, depth + 1)?;
            }
        }
        LayoutNode::Pane { pane, size } => {
            writeln!(output, "{}- pane {}{}", indent, pane, format_size(*size))?;
        }
    }

    Ok(())
}

fn format_direction(direction: &SplitDirection) -> &'static str {
    match direction {
        SplitDirection::Horizontal => "horizontal",
        SplitDirection::Vertical => "vertical",
    }
}

fn format_size(size: Option<u16>) -> String {
    match size {
        Some(size) => format!(" size={size}"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{PaneConfig, Settings};
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
    fn format_inspection_includes_layout_and_resolved_pane_settings() {
        let root = test_dir("inspection");
        let pane_dir = root.join("src");
        fs::create_dir_all(&pane_dir).expect("pane directory should be created");

        let mut panes = HashMap::new();
        panes.insert(
            "app".to_string(),
            PaneConfig {
                name: Some("App".to_string()),
                cwd: Some(PathBuf::from("src")),
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
            layout: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                size: Some(100),
                children: vec![LayoutNode::Pane {
                    pane: "app".to_string(),
                    size: Some(50),
                }],
            },
            panes,
        };

        let output = format_inspection(&root.join("workspace.dpane"), &config)
            .expect("inspection should format");

        assert!(output.contains("Workspace: Test Workspace"));
        assert!(output.contains("Scrollback: 2000"));
        assert!(output.contains("- split horizontal size=100"));
        assert!(output.contains("  - pane app size=50"));
        assert!(output.contains("  shell: pwsh"));
        assert!(output.contains("  auto_start: false"));
        assert!(output.contains("  command: cargo run"));
    }
}
