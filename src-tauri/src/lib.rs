use devpane::config::DevPaneConfig;
use devpane::workspace::build_workspace;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct RecentSession {
    pub path: String,
    pub name: String,
    pub last_opened: u64,
}

#[derive(Serialize, Deserialize)]
struct SessionStore {
    sessions: Vec<RecentSession>,
}

#[derive(Serialize)]
pub struct PaneSummary {
    pub id: String,
    pub name: String,
    pub auto_start: bool,
    pub command: Option<String>,
}

#[derive(Serialize)]
pub struct WorkspaceSummary {
    pub name: String,
    pub root: String,
    pub panes: Vec<PaneSummary>,
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

#[tauri::command]
fn list_recent_sessions(app: tauri::AppHandle) -> Vec<RecentSession> {
    read_sessions(&app)
}

#[tauri::command]
fn add_recent_session(app: tauri::AppHandle, path: String, name: String) -> Result<(), String> {
    let mut sessions = read_sessions(&app);
    sessions.retain(|s| s.path != path);
    sessions.insert(0, RecentSession { path, name, last_opened: now_secs() });
    sessions.truncate(20);
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
        panes: workspace
            .panes
            .iter()
            .map(|p| PaneSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                auto_start: p.auto_start,
                command: p.command.clone(),
            })
            .collect(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            load_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
