//! Rust → 前端：统一 emit / emit_to 出口。

use tauri::{AppHandle, Emitter, Manager};

use super::names::{self, MAIN_WINDOW_LABEL};
use super::payloads::{AppSession, ChannelStatusChangedPayload, ModalLifecyclePayload};

fn map_emit_err(e: tauri::Error) -> String {
    e.to_string()
}

/// 向所有 Webview 广播会话快照（跨窗 IPC）。
pub fn session_changed_all(app: &AppHandle, session: &AppSession) -> Result<(), String> {
    for (label, _) in app.webview_windows() {
        session_changed_to(app, &label, session)?;
    }
    Ok(())
}

/// 向单个 Webview 推送会话（如新开的 modal 窗）。
pub fn session_changed_to(
    app: &AppHandle,
    webview_label: &str,
    session: &AppSession,
) -> Result<(), String> {
    app.emit_to(webview_label, names::SESSION_CHANGED, session)
        .map_err(map_emit_err)
}

/// 通知主窗：modal 已打开（驱动蒙层）。
pub fn modal_opened(app: &AppHandle, label: impl Into<String>) -> Result<(), String> {
    let label = label.into();
    app.emit_to(
        MAIN_WINDOW_LABEL,
        names::MODAL_OPENED,
        ModalLifecyclePayload { label },
    )
    .map_err(map_emit_err)
}

/// Broadcast channel lifecycle updates to every Webview (Python sidecar → UI).
pub fn channel_status_changed_all(
    app: &AppHandle,
    payload: &ChannelStatusChangedPayload,
) -> Result<(), String> {
    for (label, _) in app.webview_windows() {
        app.emit_to(label, names::CHANNEL_STATUS_CHANGED, payload)
            .map_err(map_emit_err)?;
    }
    Ok(())
}

/// 通知主窗：modal 已关闭。
pub fn modal_closed(app: &AppHandle, label: impl Into<String>) -> Result<(), String> {
    let label = label.into();
    app.emit_to(
        MAIN_WINDOW_LABEL,
        names::MODAL_CLOSED,
        ModalLifecyclePayload { label },
    )
    .map_err(map_emit_err)
}
