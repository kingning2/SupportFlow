// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::too_many_lines)]

fn main() {
    #[cfg(feature = "desktop")]
    tauri_app_lib::run();
    #[cfg(not(feature = "desktop"))]
    compile_error!("tauri-app binary requires the `desktop` feature");
}
