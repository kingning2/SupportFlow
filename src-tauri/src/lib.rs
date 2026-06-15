pub mod io;
pub use io as fs_io;

pub mod channel_runtime;
pub mod config;
pub mod process_runtime;

pub mod cli;
pub mod python;
pub mod services;

#[cfg(feature = "desktop")]
mod cmd;
#[cfg(feature = "desktop")]
mod context;
#[cfg(feature = "desktop")]
pub mod contracts;
#[cfg(feature = "desktop")]
mod events;
pub mod utils;

pub use services::agent;
pub use services::bridge;

#[cfg(feature = "desktop")]
use tauri::Manager;

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = utils::log::init_log() {
        eprintln!("failed to init log: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(context::channel::ChannelStatusStore::default())
        .setup(|app| {
            let runtime = std::sync::Arc::new(context::agent_runtime::AgentRuntime::initialize(
                app.handle(),
            )?);
            let runtime_bg = runtime.clone();
            let license_store = tauri::async_runtime::block_on(
                context::license_store::LicenseStore::initialize_async(app.handle()),
            );
            tauri::async_runtime::spawn(async move {
                runtime_bg.start_sidecar_deferred().await;
            });
            app.manage(runtime);
            app.manage(license_store);
            app.manage(context::channel::ChannelInboxStore::open(app.handle())?);
            #[cfg(feature = "channel-wework")]
            app.manage(context::channel::WeworkAccountsStore::open(app.handle())?);
            events::setup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd::license::license_get_status,
            cmd::license::license_apply_activation,
            cmd::license::license_pick_and_apply_activation_key,
            cmd::log::log_fe,
            cmd::log::log_fe_req,
            cmd::window::open_modal_window,
            cmd::window::close_modal_window,
            cmd::window::modal_window_ready,
            cmd::window::preload_modal_window,
            cmd::agent_ipc::agent_get_console_state,
            cmd::agent_ipc::agent_send_message,
            cmd::agent_ipc::agent_cancel,
            cmd::agent_ipc::agent_clear_context,
            cmd::agent_ipc::agent_new_session,
            cmd::agent_ipc::agent_refresh_skills,
            cmd::agent_ipc::agent_get_skill_detail,
            cmd::agent_ipc::agent_install_skill,
            cmd::agent_ipc::agent_update_provider,
            cmd::agent_ipc::agent_clear_provider,
            cmd::agent_ipc::agent_set_chat_model,
            cmd::agent_ipc::agent_list_sessions,
            cmd::agent_ipc::agent_list_memory,
            cmd::agent_ipc::agent_read_memory,
            cmd::agent_ipc::agent_list_knowledge,
            cmd::agent_ipc::agent_read_knowledge,
            cmd::agent_ipc::agent_get_knowledge_graph,
            cmd::agent_ipc::agent_upload_knowledge,
            cmd::agent_ipc::agent_pick_and_upload_knowledge,
            cmd::agent_ipc::agent_remove_knowledge_file,
            cmd::agent_ipc::agent_list_channels,
            cmd::agent_ipc::agent_get_channel_catalog,
            cmd::agent_ipc::agent_channel_action,
            cmd::agent_ipc::agent_channel_console_api,
            cmd::agent_ipc::agent_list_tasks,
            cmd::agent_ipc::agent_get_logs_status,
            cmd::agent_ipc::agent_read_logs,
            cmd::agent_ipc::agent_start_log_stream,
            cmd::agent_ipc::agent_stop_log_stream,
            cmd::channel_inbox::channel_get_inbox,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_list_accounts,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_upsert_account,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_delete_account,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_get_active_account_id,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_set_active_account_id,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_mark_contacts_synced,
            #[cfg(feature = "channel-wework")]
            cmd::wework_accounts::wework_contacts_synced,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
