use crate::config::{DevPaneConfig, LayoutNode, SplitDirection};
use crate::workspace::Workspace;
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

/// Formats a resolved runtime workspace summary.
pub fn format_workspace_inspection(workspace: &Workspace) -> String {
    let mut output = String::new();

    writeln!(output, "Workspace: {}", workspace.name).expect("writing to String should not fail");
    writeln!(output, "Workspace root: {}", workspace.root.display())
        .expect("writing to String should not fail");
    writeln!(output, "Scrollback: {}", workspace.scrollback)
        .expect("writing to String should not fail");
    writeln!(output).expect("writing to String should not fail");
    writeln!(output, "Layout:").expect("writing to String should not fail");
    format_layout_node(&mut output, &workspace.layout);
    writeln!(output).expect("writing to String should not fail");
    writeln!(output, "Panes:").expect("writing to String should not fail");

    for pane in &workspace.panes {
        let command = pane.command.as_deref().unwrap_or("<no command>");

        writeln!(output, "- {}", pane.id).expect("writing to String should not fail");
        writeln!(output, "  name: {}", pane.name).expect("writing to String should not fail");
        writeln!(output, "  cwd: {}", pane.cwd.display())
            .expect("writing to String should not fail");
        writeln!(output, "  shell: {}", pane.shell).expect("writing to String should not fail");
        writeln!(output, "  auto_start: {}", pane.auto_start)
            .expect("writing to String should not fail");
        writeln!(output, "  command: {}", command).expect("writing to String should not fail");
    }

    output
}

fn format_layout_node(output: &mut String, node: &LayoutNode) {
    format_layout_node_at_depth(output, node, 0);
}

fn format_layout_node_at_depth(output: &mut String, node: &LayoutNode, depth: usize) {
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
            )
            .expect("writing to String should not fail");

            for child in children {
                format_layout_node_at_depth(output, child, depth + 1);
            }
        }
        LayoutNode::Pane { pane, size } => {
            writeln!(output, "{}- pane {}{}", indent, pane, format_size(*size))
                .expect("writing to String should not fail");
        }
    }
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
    use crate::workspace::WorkspacePane;
    use std::path::PathBuf;

    #[test]
    fn format_workspace_inspection_includes_layout_and_resolved_pane_settings() {
        let workspace = Workspace {
            name: "Test Workspace".to_string(),
            root: PathBuf::from("C:/workspace"),
            scrollback: 2000,
            layout: LayoutNode::Split {
                direction: SplitDirection::Horizontal,
                size: Some(100),
                children: vec![LayoutNode::Pane {
                    pane: "app".to_string(),
                    size: Some(50),
                }],
            },
            panes: vec![WorkspacePane {
                id: "app".to_string(),
                name: "App".to_string(),
                cwd: PathBuf::from("C:/workspace/src"),
                shell: "pwsh".to_string(),
                command: Some("cargo run".to_string()),
                auto_start: false,
            }],
        };

        let output = format_workspace_inspection(&workspace);

        assert!(output.contains("Workspace: Test Workspace"));
        assert!(output.contains("Scrollback: 2000"));
        assert!(output.contains("- split horizontal size=100"));
        assert!(output.contains("  - pane app size=50"));
        assert!(output.contains("  shell: pwsh"));
        assert!(output.contains("  auto_start: false"));
        assert!(output.contains("  command: cargo run"));
    }
}
