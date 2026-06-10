mod cli;
mod config;
mod output;
mod process;
mod workspace;

use crate::cli::{Cli, Command};
use crate::config::{DevPaneConfig, validate_config};
use crate::output::{
    format_launch_plan, format_process_results, format_validation_success,
    format_workspace_inspection,
};
use crate::process::launch::LaunchMode;
use crate::process::manager::ProcessManager;
use crate::process::runner::{RunOutcome, run_launches_until_interrupted};
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
        Command::Run { path } => run_workspace(&path),
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
    let (workspace_name, launches, _runtime) =
        build_auto_start_launches(config_path, LaunchMode::Interactive)?;

    print!("{}", format_launch_plan(&workspace_name, &launches));

    Ok(())
}

fn run_workspace(config_path: &Path) -> anyhow::Result<()> {
    let (workspace_name, launches, mut runtime) =
        build_auto_start_launches(config_path, LaunchMode::Headless)?;
    let manager = ProcessManager::new();

    print!("{}", format_launch_plan(&workspace_name, &launches));

    for launch in &launches {
        manager.mark_running(&mut runtime, &launch.pane_id)?;
    }

    let outcome = match run_launches_until_interrupted(&launches) {
        Ok(outcome) => outcome,
        Err(error) => {
            for launch in &launches {
                manager.mark_failed(&mut runtime, &launch.pane_id, error.to_string())?;
            }

            return Err(error);
        }
    };

    match outcome {
        RunOutcome::Completed(results) => {
            for result in &results {
                manager.mark_exited(&mut runtime, &result.pane_id, result.code)?;
            }

            print!("{}", format_process_results(&results));
        }
        RunOutcome::Interrupted(results) => {
            for result in &results {
                manager.mark_exited(&mut runtime, &result.pane_id, result.code)?;
            }

            println!("Interrupted. Child processes stopped.");
        }
        RunOutcome::ForcedInterrupted(results) => {
            for result in &results {
                manager.mark_exited(&mut runtime, &result.pane_id, result.code)?;
            }

            println!("Interrupted again. Exiting DevPane.");
        }
    }

    Ok(())
}

fn build_auto_start_launches(
    config_path: &Path,
    launch_mode: LaunchMode,
) -> anyhow::Result<(
    String,
    Vec<crate::process::launch::ProcessLaunch>,
    WorkspaceRuntime,
)> {
    let config = load_validated_config(config_path)?;
    let workspace = build_workspace(config_path, &config)?;
    let mut runtime = WorkspaceRuntime::new(workspace);
    let workspace_name = runtime.workspace_name().to_string();
    let manager = ProcessManager::new();
    let mut launches = Vec::new();

    let auto_start_ids: Vec<String> = runtime
        .panes
        .iter()
        .filter(|pane| pane.pane.auto_start)
        .map(|pane| pane.pane.id.clone())
        .collect();

    for pane_id in auto_start_ids {
        let launch = if launch_mode == LaunchMode::Interactive {
            manager.start_pane(&mut runtime, &pane_id)?
        } else {
            manager.start_pane_with_mode(&mut runtime, &pane_id, launch_mode)?
        };

        launches.push(launch);
    }

    Ok((workspace_name, launches, runtime))
}
