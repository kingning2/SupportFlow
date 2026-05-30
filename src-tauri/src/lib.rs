mod cmd;
mod context;
pub mod contracts;
mod events;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = utils::log::init_log() {
        eprintln!("failed to init log: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(context::session::SessionStore::load_from_disk())
        .setup(|app| {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
