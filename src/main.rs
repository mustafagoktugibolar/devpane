mod config;

use crate::config::{DevPaneConfig, validate_config};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let config_path = Path::new("examples/webclient.dpane");

    println!("Loading config from: {}", config_path.display());

    let config = DevPaneConfig::load_from_file(config_path)?;
    validate_config(&config, config_path)?;

    let workspace_root = config.workspace_root(config_path)?;

    println!("Workspace: {}", config.name);
    println!("Version: {}", config.version);
    println!("Workspace root: {}", workspace_root.display());

    println!("Panes:");

    for (pane_id, pane) in &config.panes {
        let pane_name = pane.name.as_deref().unwrap_or(pane_id);
        let pane_cwd = config.pane_cwd(config_path, pane)?;
        let command = pane.command.as_deref().unwrap_or("<no command>");

        println!("- {}", pane_id);
        println!("  name: {}", pane_name);
        println!("  cwd: {}", pane_cwd.display());
        println!("  command: {}", command);
    }

    Ok(())
}
