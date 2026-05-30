//! 前端经 `emit("fe/log" | "fe/log-req", …)` 写入 Rust 日志。

use tauri::{AppHandle, Listener};

use crate::events::names::{FE_LOG, FE_LOG_REQ};
use crate::events::payloads::{FeLogLevel, FeLogPayload};

pub fn register(app: &AppHandle) {
    app.listen_any(FE_LOG, |event| {
        dispatch("[fe]", &event);
    });
    app.listen_any(FE_LOG_REQ, |event| {
        dispatch("[fe_req]", &event);
    });
}

fn dispatch(prefix: &str, event: &tauri::Event) {
    let raw = event.payload();
    let payload: FeLogPayload = match serde_json::from_str(raw) {
        Ok(p) => p,
        Err(e) => {
            crate::log_warn!("{prefix} invalid payload: {e} (raw={raw})");
            return;
        }
    };

    match payload.level {
        FeLogLevel::Info => crate::log_info!("{prefix} {}", payload.msg),
        FeLogLevel::Warn => crate::log_warn!("{prefix} {}", payload.msg),
        FeLogLevel::Error => crate::log_error!("{prefix} {}", payload.msg),
    }
}
