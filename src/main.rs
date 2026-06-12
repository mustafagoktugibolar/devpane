mod cli;

use crate::cli::{Cli, Command};
use clap::Parser;
use devpane::config::DevPaneConfig;
use devpane::output::{
    format_launch_plan, format_process_results, format_validation_success,
    format_workspace_inspection,
};
use devpane::process::launch::LaunchMode;
use devpane::process::manager::ProcessManager;
use devpane::process::runner::{RunOutcome, run_launches_until_interrupted};
use devpane::workspace::WorkspaceRuntime;
use devpane::workspace::build_workspace;
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

fn load_config(config_path: &Path) -> anyhow::Result<DevPaneConfig> {
    println!("Loading config from: {}", config_path.display());

    DevPaneConfig::load_from_file(config_path)
}

fn validate_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
    build_workspace(config_path, &config)?;

    print!("{}", format_validation_success(config_path, &config));

    Ok(())
}

fn inspect_workspace(config_path: &Path) -> anyhow::Result<()> {
    let config = load_config(config_path)?;
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

    let (results, interrupt_message) = match outcome {
        RunOutcome::Completed(results) => (results, None),
        RunOutcome::Interrupted(results) => {
            (results, Some("Interrupted. Child processes stopped."))
        }
        RunOutcome::ForcedInterrupted(results) => {
            (results, Some("Interrupted again. Exiting DevPane."))
        }
    };

    for result in &results {
        manager.mark_exited(&mut runtime, &result.pane_id, result.code)?;
    }

    print!("{}", format_process_results(&results));

    if let Some(message) = interrupt_message {
        println!("{message}");
    }

    Ok(())
}

fn build_auto_start_launches(
    config_path: &Path,
    launch_mode: LaunchMode,
) -> anyhow::Result<(
    String,
    Vec<devpane::process::launch::ProcessLaunch>,
    WorkspaceRuntime,
)> {
    let config = load_config(config_path)?;
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
        launches.push(manager.start_pane_with_mode(&mut runtime, &pane_id, launch_mode)?);
    }

    Ok((workspace_name, launches, runtime))
}
