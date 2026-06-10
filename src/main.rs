mod cli;
mod config;
mod output;
mod process;
mod workspace;

use crate::cli::{Cli, Command};
use crate::config::{DevPaneConfig, validate_config};
use crate::output::{format_launch_plan, format_validation_success, format_workspace_inspection};
use crate::process::manager::ProcessManager;
use crate::workspace::WorkspaceRuntime;
use crate::workspace::build_workspace;
use clap::Parser;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { path } => validate_workspace(&path),
        Command::Inspect { path } => inspect_workspace(&path),
        Command::Plan { path } => plan_workspace(&path),
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

    print!("{}", format_validation_success(config_path, &config));

    Ok(())
}

fn inspect_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_validated_config(config_path)?;
    let workspace = build_workspace(config_path, &config)?;

    print!("{}", format_workspace_inspection(&workspace));

    Ok(())
}

fn plan_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_validated_config(config_path)?;
    let workspace = build_workspace(config_path, &config)?;
    let mut runtime = WorkspaceRuntime::new(workspace);
    let manager = ProcessManager::new();
    let mut launches = Vec::new();

    let auto_start_ids: Vec<String> = runtime
        .panes
        .iter()
        .filter(|pane| pane.pane.auto_start)
        .map(|pane| pane.pane.id.clone())
        .collect();

    for pane_id in auto_start_ids {
        launches.push(manager.start_pane(&mut runtime, &pane_id)?);
    }

    print!(
        "{}",
        format_launch_plan(runtime.workspace_name(), &launches)
    );

    Ok(())
}
