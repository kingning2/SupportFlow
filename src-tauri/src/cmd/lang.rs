use tauri::{path::BaseDirectory, Manager};

use crate::context::session;
use crate::utils::log::trace_result_fn;

fn normalize_lang_code(language: &str) -> &'static str {
    match language.trim() {
        "en" => "en",
        "cn" => "cn",
        _ => "cn",
    }
}

#[tauri::command]
pub async fn get_lang(app: tauri::AppHandle) -> String {
    let code = session::get_session(&app)
        .map(|s| s.current_language)
        .unwrap_or_else(|_| session::read_stored_lang());
    crate::log_info!("cmd.lang.get_lang ok lang={code}");
    code
}

#[tauri::command]
pub async fn set_lang(app: tauri::AppHandle, lang: String) -> Result<(), String> {
    trace_result_fn("cmd.lang", "set_lang", || {
        session::set_current_language(&app, lang)
    })
}

#[tauri::command]
pub async fn get_language_resource_bundle(
    handle: tauri::AppHandle,
    language: String,
) -> Result<serde_json::Value, String> {
    trace_result_fn("cmd.lang", "get_language_resource_bundle", || {
        let code = normalize_lang_code(&language);
        let resource_path = handle
            .path()
            .resolve(
                format!("resources/languages/{code}.json"),
                BaseDirectory::Resource,
            )
            .map_err(|e| e.to_string())?;

        let mut content = crate::utils::fs::read_to_string(&resource_path)?;
        // Windows editors / PowerShell may write UTF-8 BOM; serde_json rejects it.
        if content.starts_with('\u{FEFF}') {
            content = content.trim_start_matches('\u{FEFF}').to_string();
        }
        let bundle: serde_json::Value = crate::utils::json::from_str(&content)?;
        Ok(bundle)
    })
}
