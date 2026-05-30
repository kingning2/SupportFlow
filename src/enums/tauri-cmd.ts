/** Tauri invoke 命令名（与 Rust `generate_handler!` 注册名一致） */
export enum TauriCmd {
  GetLang = "get_lang",
  SetLang = "set_lang",
  GetLanguageResourceBundle = "get_language_resource_bundle",
  GetAppSession = "get_app_session",
  LogFe = "log_fe",
  LogFeReq = "log_fe_req",
  OpenModalWindow = "open_modal_window",
  CloseModalWindow = "close_modal_window",
  ModalWindowReady = "modal_window_ready",
  PreloadModalWindow = "preload_modal_window"
}
