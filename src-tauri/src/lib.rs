use devpane::config::{DEFAULT_SHELL, DevPaneConfig, PaneConfig, Settings};
use devpane::workspace::build_workspace;
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Emitter, Manager};

#[derive(Default)]
struct TerminalStore {
    terminals: Mutex<HashMap<String, RunningTerminal>>,

    /// Pane ids whose terminals were stopped on purpose. Their reader threads
    /// must not emit a `terminal-exited` event for the kill.
    stopping: Mutex<HashSet<String>>,

    /// Monotonic id distinguishing successive terminals for the same pane.
    next_generation: AtomicU64,
}

struct RunningTerminal {
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    generation: u64,
}

#[derive(Serialize, Deserialize, Clone)]
/// Recent workspace entry stored by the desktop app.
pub struct RecentSession {
    /// Absolute or user-provided path to the `.dpane` file.
    pub path: String,

    /// Display name shown in the session picker.
    pub name: String,

    /// UNIX timestamp, in seconds, for the last time this session was opened.
    pub last_opened: u64,
}

#[derive(Serialize, Deserialize)]
struct SessionStore {
    sessions: Vec<RecentSession>,
}

#[derive(Serialize)]
/// Minimal pane data sent from Rust to the desktop UI.
pub struct PaneSummary {
    /// Stable pane id from the `.dpane` file.
    pub id: String,

    /// Human readable pane name.
    pub name: String,

    /// Whether this pane should start automatically.
    pub auto_start: bool,

    /// Optional startup command for the pane.
    pub command: Option<String>,

    /// Resolved working directory for the pane.
    pub cwd: String,

    /// Resolved shell program for the pane.
    pub shell: String,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
/// Split direction used by the desktop UI layout tree.
#[serde(rename_all = "lowercase")]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

#[derive(Serialize, Deserialize, Clone)]
/// Recursive layout tree used by the desktop UI.
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WorkspaceLayoutNode {
    /// Terminal pane leaf.
    Pane {
        /// Stable pane id from the `.dpane` file.
        pane: String,

        /// Relative size used when splitting or resizing.
        size: Option<u16>,
    },

    /// Split container with ordered child nodes.
    Split {
        /// Split direction.
        direction: LayoutDirection,

        /// Relative size used when this split sits under a parent split.
        size: Option<u16>,

        /// Child layout nodes.
        children: Vec<WorkspaceLayoutNode>,
    },
}

#[derive(Serialize)]
/// Resolved workspace data sent to the desktop UI.
pub struct WorkspaceSummary {
    /// Human readable workspace name.
    pub name: String,

    /// Resolved workspace root path.
    pub root: String,

    /// Recursive workspace layout tree.
    pub layout: WorkspaceLayoutNode,

    /// Resolved terminal panes in the workspace.
    pub panes: Vec<PaneSummary>,

    /// Resolved terminal scrollback line count.
    pub scrollback: u32,
}

#[derive(Deserialize)]
/// Draft pane data received from the desktop save dialog.
pub struct DraftPane {
    /// Stable pane id to write into the generated `.dpane` file.
    pub id: String,

    /// Human readable pane name.
    pub name: String,

    /// Optional startup command to persist for this pane.
    pub command: Option<String>,
}

#[derive(Deserialize)]
/// Request payload for saving a draft workspace to disk.
pub struct SaveWorkspaceRequest {
    /// Target `.dpane` path.
    pub path: String,

    /// Workspace name to write.
    pub name: String,

    /// Recursive layout tree to persist.
    pub layout: WorkspaceLayoutNode,

    /// Draft panes to persist.
    pub panes: Vec<DraftPane>,
}

#[derive(Deserialize)]
/// Request payload for deleting a saved workspace.
pub struct DeleteWorkspaceRequest {
    /// Target `.dpane` path to remove from disk and recent sessions.
    pub path: String,
}

#[derive(Deserialize)]
/// Request payload for starting a PTY-backed terminal pane.
pub struct StartTerminalRequest {
    /// Pane id used to route terminal input, output, and lifecycle commands.
    pub pane_id: String,

    /// Human readable pane name shown in shell prompts and UI labels.
    pub pane_name: String,

    /// Optional working directory for the terminal process.
    ///
    /// Defaults to the user's home directory when missing.
    pub cwd: Option<String>,

    /// Optional shell program for the terminal.
    ///
    /// Defaults to the platform default shell when missing.
    pub shell: Option<String>,

    /// Optional startup command run inside the terminal shell.
    pub command: Option<String>,

    /// Initial terminal row count.
    pub rows: u16,

    /// Initial terminal column count.
    pub cols: u16,
}

#[derive(Deserialize)]
/// Request payload for writing user input to a terminal pane.
pub struct TerminalInputRequest {
    /// Pane id for the target running terminal.
    pub pane_id: String,

    /// Raw terminal input bytes encoded as UTF-8 text.
    pub data: String,
}

#[derive(Deserialize)]
/// Request payload for resizing a running terminal pane.
pub struct ResizeTerminalRequest {
    /// Pane id for the target running terminal.
    pub pane_id: String,

    /// New terminal row count.
    pub rows: u16,

    /// New terminal column count.
    pub cols: u16,
}

#[derive(Serialize, Clone)]
/// Terminal output event payload emitted to the desktop UI.
pub struct TerminalOutput {
    /// Pane id that produced this terminal output.
    pub pane_id: String,

    /// Raw terminal output bytes decoded as UTF-8 text.
    pub data: String,
}

#[derive(Serialize, Clone)]
/// Terminal lifecycle event emitted when a pane process exits or its PTY closes.
pub struct TerminalExit {
    /// Pane id whose terminal has exited.
    pub pane_id: String,
}

/// Serialized `.dpane` file with stable pane ordering.
///
/// Optional fields are omitted instead of written as defaults so a re-save
/// preserves what the original file left unspecified.
#[derive(Serialize)]
struct DpaneFile<'a> {
    version: u16,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    root: Option<&'a std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<&'a Settings>,
    layout: devpane::config::LayoutNode,
    panes: std::collections::BTreeMap<&'a str, DpanePane<'a>>,
}

#[derive(Serialize)]
struct DpanePane<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    cwd: Option<&'a std::path::PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shell: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_start: Option<bool>,
}

fn sessions_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("sessions.json"))
}

fn read_sessions(app: &tauri::AppHandle) -> Vec<RecentSession> {
    let Ok(path) = sessions_path(app) else {
        return vec![];
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    serde_json::from_str::<SessionStore>(&content)
        .map(|s| s.sessions)
        .unwrap_or_default()
}

fn write_sessions(app: &tauri::AppHandle, sessions: Vec<RecentSession>) -> Result<(), String> {
    let path = sessions_path(app)?;
    let content =
        serde_json::to_string_pretty(&SessionStore { sessions }).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn workspace_file_name(name: &str) -> String {
    let mut file_name = name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    while file_name.contains("--") {
        file_name = file_name.replace("--", "-");
    }

    let file_name = file_name.trim_matches('-');
    if file_name.is_empty() {
        "workspace.dpane".to_string()
    } else {
        format!("{file_name}.dpane")
    }
}

fn layout_from_config(node: &devpane::config::LayoutNode) -> WorkspaceLayoutNode {
    match node {
        devpane::config::LayoutNode::Pane { pane, size } => WorkspaceLayoutNode::Pane {
            pane: pane.clone(),
            size: *size,
        },
        devpane::config::LayoutNode::Split {
            direction,
            size,
            children,
        } => WorkspaceLayoutNode::Split {
            direction: match direction {
                devpane::config::SplitDirection::Horizontal => LayoutDirection::Horizontal,
                devpane::config::SplitDirection::Vertical => LayoutDirection::Vertical,
            },
            size: *size,
            children: children.iter().map(layout_from_config).collect(),
        },
    }
}

fn layout_to_config(node: &WorkspaceLayoutNode) -> devpane::config::LayoutNode {
    match node {
        WorkspaceLayoutNode::Pane { pane, size } => devpane::config::LayoutNode::Pane {
            pane: pane.clone(),
            size: *size,
        },
        WorkspaceLayoutNode::Split {
            direction,
            size,
            children,
        } => devpane::config::LayoutNode::Split {
            direction: match direction {
                LayoutDirection::Horizontal => devpane::config::SplitDirection::Horizontal,
                LayoutDirection::Vertical => devpane::config::SplitDirection::Vertical,
            },
            size: *size,
            children: children.iter().map(layout_to_config).collect(),
        },
    }
}

#[tauri::command]
fn list_recent_sessions(app: tauri::AppHandle) -> Vec<RecentSession> {
    read_sessions(&app)
}

#[tauri::command]
fn add_recent_session(app: tauri::AppHandle, path: String, name: String) -> Result<(), String> {
    let mut sessions = read_sessions(&app);
    sessions.retain(|s| s.path != path);
    sessions.insert(
        0,
        RecentSession {
            path,
            name,
            last_opened: now_secs(),
        },
    );
    sessions.truncate(20);
    write_sessions(&app, sessions)
}

#[tauri::command]
fn suggest_workspace_path(app: tauri::AppHandle, name: Option<String>) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("workspaces");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let file_name = workspace_file_name(name.as_deref().unwrap_or("workspace"));
    Ok(dir.join(file_name).display().to_string())
}

#[tauri::command]
fn delete_workspace(app: tauri::AppHandle, request: DeleteWorkspaceRequest) -> Result<(), String> {
    let path = Path::new(&request.path);

    if path.exists() {
        if path.is_dir() {
            return Err("workspace path is a directory, not a .dpane file".to_string());
        }

        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    let mut sessions = read_sessions(&app);
    sessions.retain(|session| session.path != request.path);
    write_sessions(&app, sessions)
}

#[tauri::command]
fn load_workspace(path: String) -> Result<WorkspaceSummary, String> {
    let config_path = Path::new(&path);
    let config = DevPaneConfig::load_from_file(config_path).map_err(|e| e.to_string())?;
    let workspace = build_workspace(config_path, &config).map_err(|e| e.to_string())?;

    Ok(WorkspaceSummary {
        name: workspace.name,
        root: workspace.root.display().to_string(),
        layout: layout_from_config(&workspace.layout),
        panes: workspace
            .panes
            .iter()
            .map(|p| PaneSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                auto_start: p.auto_start,
                command: p.command.clone(),
                cwd: p.cwd.display().to_string(),
                shell: p.shell.clone(),
            })
            .collect(),
        scrollback: workspace.scrollback,
    })
}

/// Returns the lowercase program name of a shell path, without extension.
fn shell_program_name(shell: &str) -> String {
    Path::new(shell)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_lowercase())
        .unwrap_or_else(|| shell.to_lowercase())
}

#[cfg(windows)]
fn shell_command(shell: &str, command: Option<&str>, pane_name: &str) -> CommandBuilder {
    let mut builder = CommandBuilder::new(shell);
    builder.env("DEVPANE_PANE_NAME", pane_name);
    let command = command.filter(|value| !value.trim().is_empty());

    match shell_program_name(shell).as_str() {
        "powershell" | "pwsh" => {
            builder.args(["-NoLogo", "-NoExit"]);

            let mut startup = r#"function global:prompt { $location = $executionContext.SessionState.Path.CurrentLocation.Path; if ($location.StartsWith('\\?\')) { $location = $location.Substring(4) }; "[$env:DEVPANE_PANE_NAME] PS $location> " }"#.to_string();
            if let Some(command) = command {
                startup.push_str("; ");
                startup.push_str(command);
            }

            builder.args(["-Command", &startup]);
        }
        "cmd" => {
            if let Some(command) = command {
                builder.args(["/K", command]);
            }
        }
        _ => {
            if let Some(command) = command {
                builder.args(["-lc", command]);
            }
        }
    }

    builder
}

#[cfg(not(windows))]
fn shell_command(shell: &str, command: Option<&str>, pane_name: &str) -> CommandBuilder {
    let mut builder = CommandBuilder::new(shell);
    builder.env("DEVPANE_PANE_NAME", pane_name);

    if let Some(command) = command.filter(|value| !value.trim().is_empty()) {
        builder.args(["-lc", command]);
    }

    builder
}

#[cfg(windows)]
fn terminal_cwd(cwd: &str) -> String {
    cwd.strip_prefix(r"\\?\").unwrap_or(cwd).to_string()
}

#[cfg(not(windows))]
fn terminal_cwd(cwd: &str) -> String {
    cwd.to_string()
}

/// Resolves the default shell for terminals started without an explicit one.
fn default_shell() -> String {
    #[cfg(not(windows))]
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.trim().is_empty() {
            return shell;
        }
    }

    DEFAULT_SHELL.to_string()
}

/// Resolves the user's home directory for terminals started without a cwd.
fn home_dir() -> Option<String> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var(var)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

#[tauri::command]
fn start_terminal(
    app: tauri::AppHandle,
    state: tauri::State<'_, TerminalStore>,
    request: StartTerminalRequest,
) -> Result<(), String> {
    stop_terminal(state.clone(), request.pane_id.clone())?;

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: request.rows.max(1),
            cols: request.cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let shell = request
        .shell
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(default_shell);

    let mut command = shell_command(&shell, request.command.as_deref(), &request.pane_name);
    let cwd = request
        .cwd
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(terminal_cwd)
        .or_else(home_dir);
    if let Some(cwd) = cwd {
        command.cwd(cwd);
    }

    let mut child = pair
        .slave
        .spawn_command(command)
        .map_err(|e| e.to_string())?;
    drop(pair.slave);

    let killer = child.clone_killer();
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
    let generation = state.next_generation.fetch_add(1, Ordering::SeqCst);
    let pane_id = request.pane_id.clone();
    let event_app = app.clone();

    // Register the terminal before the reader thread starts so a fast-exiting
    // process cannot race the store cleanup at the end of the thread.
    state
        .terminals
        .lock()
        .map_err(|_| "terminal store lock poisoned".to_string())?
        .insert(
            request.pane_id.clone(),
            RunningTerminal {
                writer,
                killer,
                master: pair.master,
                generation,
            },
        );

    // ConPTY readers do not see EOF when the child exits; the PTY must be
    // closed first. Wait for the child here and drop the store entry (and
    // with it the master) so the reader thread unblocks.
    let waiter_app = app.clone();
    let waiter_pane = request.pane_id.clone();
    std::thread::spawn(move || {
        let _ = child.wait();

        let store = waiter_app.state::<TerminalStore>();
        if let Ok(mut terminals) = store.terminals.lock() {
            if terminals
                .get(&waiter_pane)
                .is_some_and(|terminal| terminal.generation == generation)
            {
                terminals.remove(&waiter_pane);
            }
        }
    });

    std::thread::spawn(move || {
        let output_event = format!("terminal-output-{pane_id}");
        let mut buffer = [0_u8; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(size) => {
                    let data = String::from_utf8_lossy(&buffer[..size]).to_string();
                    let _ = event_app.emit(
                        &output_event,
                        TerminalOutput {
                            pane_id: pane_id.clone(),
                            data,
                        },
                    );
                }
                Err(error) => {
                    let _ = event_app.emit(
                        &output_event,
                        TerminalOutput {
                            pane_id: pane_id.clone(),
                            data: format!("\r\n[terminal read error: {error}]\r\n"),
                        },
                    );
                    break;
                }
            }
        }

        let store = event_app.state::<TerminalStore>();

        // Drop the store entry only if it still belongs to this terminal; a
        // restart may already have replaced it with a newer generation.
        if let Ok(mut terminals) = store.terminals.lock() {
            if terminals
                .get(&pane_id)
                .is_some_and(|terminal| terminal.generation == generation)
            {
                terminals.remove(&pane_id);
            }
        }

        // An intentional stop already removed the entry; skip the exit event
        // so the UI does not treat the kill as a process exit.
        let intentional = store
            .stopping
            .lock()
            .map(|mut stopping| stopping.remove(&pane_id))
            .unwrap_or(false);

        if !intentional {
            let _ = event_app.emit(
                &format!("terminal-exited-{pane_id}"),
                TerminalExit {
                    pane_id: pane_id.clone(),
                },
            );
        }
    });

    Ok(())
}

#[tauri::command]
fn write_terminal(
    state: tauri::State<'_, TerminalStore>,
    request: TerminalInputRequest,
) -> Result<(), String> {
    let mut terminals = state
        .terminals
        .lock()
        .map_err(|_| "terminal store lock poisoned".to_string())?;
    let terminal = terminals
        .get_mut(&request.pane_id)
        .ok_or_else(|| format!("terminal '{}' is not running", request.pane_id))?;

    terminal
        .writer
        .write_all(request.data.as_bytes())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn resize_terminal(
    state: tauri::State<'_, TerminalStore>,
    request: ResizeTerminalRequest,
) -> Result<(), String> {
    let terminals = state
        .terminals
        .lock()
        .map_err(|_| "terminal store lock poisoned".to_string())?;
    let terminal = terminals
        .get(&request.pane_id)
        .ok_or_else(|| format!("terminal '{}' is not running", request.pane_id))?;

    terminal
        .master
        .resize(PtySize {
            rows: request.rows.max(1),
            cols: request.cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_terminal(state: tauri::State<'_, TerminalStore>, pane_id: String) -> Result<(), String> {
    let terminal = state
        .terminals
        .lock()
        .map_err(|_| "terminal store lock poisoned".to_string())?
        .remove(&pane_id);

    if let Some(mut terminal) = terminal {
        // The reader thread only consumes this flag after the entry is gone
        // from the store, so flagging before the kill is race-free.
        if let Ok(mut stopping) = state.stopping.lock() {
            stopping.insert(pane_id);
        }

        let _ = terminal.killer.kill();
        // Dropping `terminal` closes the PTY master, which unblocks the
        // reader thread on Windows ConPTY.
    }

    Ok(())
}

#[tauri::command]
fn save_workspace(request: SaveWorkspaceRequest) -> Result<WorkspaceSummary, String> {
    if request.name.trim().is_empty() {
        return Err("workspace name cannot be empty".to_string());
    }

    if request.panes.is_empty() {
        return Err("add at least one pane before saving".to_string());
    }

    let config_path = Path::new(&request.path);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    // Preserve fields the UI does not edit (root, settings, per-pane
    // cwd/shell/auto_start) when overwriting an existing workspace file.
    let existing: Option<DevPaneConfig> = if config_path.exists() {
        Some(DevPaneConfig::load_from_file(config_path).map_err(|e| e.to_string())?)
    } else {
        None
    };
    let existing_pane = |id: &str| -> Option<&PaneConfig> {
        existing.as_ref().and_then(|config| config.panes.get(id))
    };

    let panes = request
        .panes
        .iter()
        .map(|pane| {
            let original = existing_pane(&pane.id);

            (
                pane.id.as_str(),
                DpanePane {
                    name: pane.name.as_str(),
                    cwd: original.and_then(|p| p.cwd.as_ref()),
                    shell: original.and_then(|p| p.shell.as_deref()),
                    command: pane
                        .command
                        .as_deref()
                        .filter(|command| !command.trim().is_empty()),
                    auto_start: original.and_then(|p| p.auto_start),
                },
            )
        })
        .collect();

    let file = DpaneFile {
        version: 1,
        name: request.name.trim(),
        root: existing.as_ref().and_then(|config| config.root.as_ref()),
        settings: existing
            .as_ref()
            .and_then(|config| config.settings.as_ref()),
        layout: layout_to_config(&request.layout),
        panes,
    };

    let content = serde_yaml::to_string(&file).map_err(|e| e.to_string())?;
    std::fs::write(config_path, content).map_err(|e| e.to_string())?;

    load_workspace(request.path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Runs the Tauri desktop application.
pub fn run() {
    tauri::Builder::default()
        .manage(TerminalStore::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_recent_sessions,
            add_recent_session,
            suggest_workspace_path,
            delete_workspace,
            load_workspace,
            save_workspace,
            start_terminal,
            write_terminal,
            resize_terminal,
            stop_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
