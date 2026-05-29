use nova_core::session::WorkspaceSession;
use tauri::command;

#[command]
pub fn load_workspace_session(workspace_path: String) -> Result<WorkspaceSession, String> {
    WorkspaceSession::load(&workspace_path).map_err(|e| e.to_string())
}

#[command]
pub fn save_workspace_session(session: WorkspaceSession) -> Result<(), String> {
    // Optionally update the timestamp before saving
    let mut session_to_save = session;
    session_to_save.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    session_to_save.save().map_err(|e| e.to_string())
}
