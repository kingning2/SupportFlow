mod cmd;
mod context;
pub mod contracts;
mod events;
mod utils;

use tauri::Manager;

/// Agent runtime (Python `agent/` package, incremental port).
pub use agent;
/// LLM `models/` layer (Python `models/` package).
pub use models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = utils::log::init_log() {
        eprintln!("failed to init log: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(context::session::SessionStore::load_from_disk())
        .setup(|app| {
            let runtime = std::sync::Arc::new(context::agent_runtime::AgentRuntime::initialize(
                app.handle(),
            )?);
            let runtime_bg = runtime.clone();
            tauri::async_runtime::spawn(async move {
                runtime_bg.start_sidecar_deferred().await;
            });
            app.manage(runtime);
            events::setup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::lang::get_lang,
            cmd::lang::set_lang,
            cmd::session::get_app_session,
            cmd::lang::get_language_resource_bundle,
            cmd::log::log_fe,
            cmd::log::log_fe_req,
            cmd::window::open_modal_window,
            cmd::window::close_modal_window,
            cmd::window::modal_window_ready,
            cmd::window::preload_modal_window,
            cmd::agent::agent_get_console_state,
            cmd::agent::agent_send_message,
            cmd::agent::agent_cancel,
            cmd::agent::agent_clear_context,
            cmd::agent::agent_new_session,
            cmd::agent::agent_refresh_skills,
            cmd::agent::agent_update_provider,
            cmd::agent::agent_clear_provider,
            cmd::agent::agent_set_chat_model,
            cmd::agent::agent_list_sessions,
            cmd::agent::agent_list_memory,
            cmd::agent::agent_read_memory,
            cmd::agent::agent_list_knowledge,
            cmd::agent::agent_read_knowledge,
            cmd::agent::agent_get_knowledge_graph,
            cmd::agent::agent_list_channels,
            cmd::agent::agent_get_channel_catalog,
            cmd::agent::agent_channel_action,
            cmd::agent::agent_channel_console_api,
            cmd::agent::agent_list_tasks,
            cmd::agent::agent_get_logs_status,
            cmd::agent::agent_read_logs,
            cmd::agent::agent_start_log_stream,
            cmd::agent::agent_stop_log_stream,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
