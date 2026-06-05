use tauri::AppHandle;

use crate::context::session::{self, AppSession};

#[tauri::command]
pub fn get_app_session(app: AppHandle) -> Result<AppSession, String> {
    let result = session::get_session(&app);

    match &result {
        Ok(snapshot) => crate::log_cmd_ok!(
            "cmd.session.get_app_session",
            "lang={}",
            snapshot.current_language
        ),
        Err(err) => crate::log_cmd_err!("cmd.session.get_app_session", err),
    }

    result
}
