use crate::process::launch::ProcessLaunch;
use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::sync::OnceLock;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

static INTERRUPTS: OnceLock<Arc<AtomicUsize>> = OnceLock::new();

/// Completed process result for a pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessResult {
    /// Pane id that owned this process.
    pub pane_id: String,

    /// Process exit code, if the platform provides one.
    pub code: Option<i32>,
}

/// Result of running a set of pane processes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    /// All processes exited normally from DevPane's perspective.
    Completed(Vec<ProcessResult>),

    /// The user interrupted the run and DevPane attempted to stop children.
    Interrupted(Vec<ProcessResult>),

    /// The user interrupted again while DevPane was stopping children.
    ForcedInterrupted(Vec<ProcessResult>),
}

struct RunningProcess {
    pane_id: String,
    child: Child,
}

/// Starts all launch plans and waits for them to finish.
///
/// Processes are spawned first, then waited on. This lets multiple auto-start
/// panes run concurrently.
///
/// # Errors
///
/// Returns an error if any process cannot be spawned or waited on.
#[cfg(test)]
fn run_launches(launches: &[ProcessLaunch]) -> Result<Vec<ProcessResult>> {
    match run_launches_with_interrupt_counter(launches, Arc::new(AtomicUsize::new(0)))? {
        RunOutcome::Completed(results)
        | RunOutcome::Interrupted(results)
        | RunOutcome::ForcedInterrupted(results) => Ok(results),
    }
}

/// Starts all launch plans and waits until completion or Ctrl+C.
///
/// # Errors
///
/// Returns an error if any process cannot be spawned, waited on, or if the
/// Ctrl+C handler cannot be registered.
pub fn run_launches_until_interrupted(launches: &[ProcessLaunch]) -> Result<RunOutcome> {
    let interrupts = ctrl_c_counter()?;
    interrupts.store(0, Ordering::SeqCst);

    run_launches_with_interrupt_counter(launches, interrupts)
}

fn ctrl_c_counter() -> Result<Arc<AtomicUsize>> {
    if let Some(interrupts) = INTERRUPTS.get() {
        return Ok(Arc::clone(interrupts));
    }

    let interrupts = Arc::new(AtomicUsize::new(0));
    let handler_interrupts = Arc::clone(&interrupts);

    ctrlc::set_handler(move || {
        handler_interrupts.fetch_add(1, Ordering::SeqCst);
    })
    .context("failed to register Ctrl+C handler")?;

    let _ = INTERRUPTS.set(Arc::clone(&interrupts));

    Ok(interrupts)
}

/// Kills a child process and, on Windows, its whole process tree.
///
/// `Child::kill` only terminates the direct child; on Windows the shell's own
/// children (e.g. a `cargo run` server) would survive it.
fn kill_process_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .output();
    }

    let _ = child.kill();
}

fn run_launches_with_interrupt_counter(
    launches: &[ProcessLaunch],
    interrupts: Arc<AtomicUsize>,
) -> Result<RunOutcome> {
    let mut running: Vec<RunningProcess> = Vec::with_capacity(launches.len());

    for launch in launches {
        let spawned = Command::new(&launch.program)
            .args(&launch.args)
            .current_dir(&launch.cwd)
            .spawn()
            .with_context(|| format!("failed to start pane '{}'", launch.pane_id));

        match spawned {
            Ok(child) => running.push(RunningProcess {
                pane_id: launch.pane_id.clone(),
                child,
            }),
            Err(error) => {
                for process in &mut running {
                    kill_process_tree(&mut process.child);
                    let _ = process.child.wait();
                }

                return Err(error);
            }
        }
    }

    let mut results = Vec::with_capacity(running.len());

    while !running.is_empty() {
        let interrupt_count = interrupts.load(Ordering::SeqCst);

        if interrupt_count > 0 {
            for process in &mut running {
                kill_process_tree(&mut process.child);
            }

            for mut process in running {
                let status = process
                    .child
                    .wait()
                    .with_context(|| format!("failed to wait for pane '{}'", process.pane_id))?;

                results.push(ProcessResult {
                    pane_id: process.pane_id,
                    code: status.code(),
                });
            }

            if interrupts.load(Ordering::SeqCst) > 1 {
                return Ok(RunOutcome::ForcedInterrupted(results));
            }

            return Ok(RunOutcome::Interrupted(results));
        }

        let mut index = 0;

        while index < running.len() {
            match running[index]
                .child
                .try_wait()
                .with_context(|| format!("failed to poll pane '{}'", running[index].pane_id))?
            {
                Some(status) => {
                    let process = running.remove(index);

                    results.push(ProcessResult {
                        pane_id: process.pane_id,
                        code: status.code(),
                    });
                }
                None => {
                    index += 1;
                }
            }
        }

        if !running.is_empty() {
            thread::sleep(Duration::from_millis(100));
        }
    }

    Ok(RunOutcome::Completed(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn exit_launch(code: i32) -> ProcessLaunch {
        ProcessLaunch {
            pane_id: "app".to_string(),
            cwd: std::env::current_dir().expect("current directory should be available"),
            program: "cmd".to_string(),
            args: vec!["/C".to_string(), format!("exit {code}")],
        }
    }

    #[cfg(not(windows))]
    fn exit_launch(code: i32) -> ProcessLaunch {
        ProcessLaunch {
            pane_id: "app".to_string(),
            cwd: std::env::current_dir().expect("current directory should be available"),
            program: "sh".to_string(),
            args: vec!["-c".to_string(), format!("exit {code}")],
        }
    }

    #[test]
    fn run_launches_returns_process_exit_code() {
        let results = run_launches(&[exit_launch(7)]).expect("launch should run");

        assert_eq!(
            results,
            vec![ProcessResult {
                pane_id: "app".to_string(),
                code: Some(7)
            }]
        );
    }

    #[test]
    fn run_launches_handles_empty_launch_list() {
        let results = run_launches(&[]).expect("empty launch list should run");

        assert!(results.is_empty());
    }

    #[test]
    fn run_launches_with_interrupt_counter_returns_interrupted() {
        let interrupts = Arc::new(AtomicUsize::new(1));
        let outcome = run_launches_with_interrupt_counter(&[exit_launch(0)], interrupts)
            .expect("run should finish");

        match outcome {
            RunOutcome::Interrupted(results) => {
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].pane_id, "app");
            }
            RunOutcome::Completed(_) | RunOutcome::ForcedInterrupted(_) => {
                panic!("expected interrupted outcome")
            }
        }
    }

    #[test]
    fn run_launches_with_interrupt_counter_returns_forced_interrupted() {
        let interrupts = Arc::new(AtomicUsize::new(2));
        let outcome = run_launches_with_interrupt_counter(&[exit_launch(0)], interrupts)
            .expect("run should finish");

        match outcome {
            RunOutcome::ForcedInterrupted(results) => {
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].pane_id, "app");
            }
            RunOutcome::Completed(_) | RunOutcome::Interrupted(_) => {
                panic!("expected forced interrupted outcome")
            }
        }
    }
}
