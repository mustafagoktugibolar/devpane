mod cli;
mod config;

use crate::cli::{Cli, Command};
use crate::config::{DevPaneConfig, validate_config};
use clap::Parser;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { path } => validate_workspace(&path),
        Command::Inspect { path } => inspect_workspace(&path),
    }
}

fn load_validated_config(config_path: &Path) -> anyhow::Result<DevPaneConfig> {
    println!("Loading config from: {}", config_path.display());

    let config = DevPaneConfig::load_from_file(config_path)?;
    validate_config(&config, config_path)?;

    Ok(config)
}

fn validate_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_validated_config(config_path)?;

    println!("Valid .dpane file: {}", config_path.display());
    println!("Workspace: {}", config.name);
    println!("Panes: {}", config.panes.len());

    Ok(())
}

fn inspect_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_validated_config(config_path)?;
    let workspace_root = config.workspace_root(config_path)?;

    println!("Workspace: {}", config.name);
    println!("Version: {}", config.version);
    println!("Workspace root: {}", workspace_root.display());
    println!("Scrollback: {}", config.scrollback());

    println!("Panes:");

    for (pane_id, pane) in &config.panes {
        let pane_name = pane.name.as_deref().unwrap_or(pane_id);
        let pane_cwd = config.pane_cwd(config_path, pane)?;
        let pane_shell = config.pane_shell(pane);
        let pane_auto_start = config.pane_auto_start(pane);
        let command = pane.command.as_deref().unwrap_or("<no command>");

        println!("- {}", pane_id);
        println!("  name: {}", pane_name);
        println!("  cwd: {}", pane_cwd.display());
        println!("  shell: {}", pane_shell);
        println!("  auto_start: {}", pane_auto_start);
        println!("  command: {}", command);
    }

    Ok(())
}
