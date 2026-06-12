//! Rust -> 前端：统一 emit / emit_to 出口。

use tauri::{AppHandle, Emitter, Manager};

use super::names::{self, MAIN_WINDOW_LABEL};
use super::payloads::{ChannelStatusChangedPayload, ModalLifecyclePayload};

fn map_emit_err(e: tauri::Error) -> String {
    e.to_string()
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

/// Broadcast channel lifecycle updates to every Webview (Python sidecar -> UI).
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
