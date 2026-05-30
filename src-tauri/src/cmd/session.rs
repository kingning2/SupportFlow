use tauri::AppHandle;

use crate::context::session::{self, AppSession};

#[tauri::command]
pub fn get_app_session(app: AppHandle) -> Result<AppSession, String> {
    let snapshot = session::get_session(&app)?;
    crate::log_info!(
        "cmd.session.get_app_session ok lang={}",
        snapshot.current_language
    );
    Ok(snapshot)
}
