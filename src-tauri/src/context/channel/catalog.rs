//! Rust-owned channel catalog for frontend rendering and runtime aggregation.

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::services::channel::{all_channel_defs, is_known_channel, ChannelDef};

use super::bridge::ChannelBridge;
use super::status::ChannelStatusStore;
#[cfg(feature = "channel-wework")]
use super::wework_accounts::WeworkAccountsStore;

fn localized(zh: &str, en: &str) -> Value {
    json!({ "zh": zh, "en": en })
}

fn read_config_value(config_path: &std::path::Path, key: &str) -> Option<Value> {
    let raw = crate::utils::fs::read_to_string(config_path).ok()?;
    let root: Value = crate::utils::json::from_str(&raw).ok()?;
    root.get(key).cloned()
}

fn active_channel_names(config_path: &std::path::Path) -> Result<Vec<String>, String> {
    let raw = crate::utils::fs::read_to_string(config_path)?;
    let root: Value = crate::utils::json::from_str(&raw)?;
    Ok(crate::utils::channel::parse_desktop_channel_types(
        root.get("channel_type"),
    ))
}

fn phase_to_login_status(phase: &str) -> Option<&'static str> {
    ChannelBridge::login_status_for_phase(phase)
}

fn phase_to_active(phase: &str) -> bool {
    ChannelBridge::is_active_phase(phase)
}

#[cfg(feature = "channel-wework")]
fn catalog_row(
    def: &ChannelDef,
    in_config: bool,
    runtime: Option<&super::status::ChannelRuntimeStatus>,
    config_path: &std::path::Path,
    wework_accounts: &WeworkAccountsStore,
) -> Value {
    let fields = def
        .fields
        .iter()
        .map(|field| {
            let value = read_config_value(config_path, field.key)
                .unwrap_or_else(|| field.default_value.clone());
            let mut row = json!({
                "key": field.key,
                "label": localized(field.label_zh, field.label_en),
                "type": field.field_type,
                "value": value,
                "default": field.default_value,
            });
            if let (Some(zh), Some(en)) = (field.placeholder_zh, field.placeholder_en) {
                row["placeholder"] = localized(zh, en);
            }
            row
        })
        .collect::<Vec<_>>();

    let mut row = json!({
        "name": def.name,
        "label": localized(def.label_zh, def.label_en),
        "active": runtime
            .map(|status| phase_to_active(&status.phase))
            .unwrap_or(false),
        "fields": fields,
        "hint": localized(def.hint_zh, def.hint_en),
        "icon": def.icon,
        "color": def.color,
    });

    let login_status = runtime
        .and_then(|status| phase_to_login_status(&status.phase).map(str::to_string))
        .or_else(|| {
            if in_config {
                Some("unknown".to_string())
            } else {
                None
            }
        });
    if let Some(login_status) = login_status {
        row["login_status"] = Value::String(login_status.clone());
        row["loginStatus"] = Value::String(login_status);
    }

    if def.name == "wework" {
        if let Some(status) = runtime {
            if let Some(user_id) = status.user_id.as_deref() {
                row["login_profile"] = json!({
                    "user_id": user_id,
                    "display_name": status.display_name.clone().unwrap_or_default(),
                });
                if let Ok(true) = wework_accounts.contacts_synced(user_id) {
                    row["contacts_synced"] = Value::Bool(true);
                }
            }
        }
    }

    row
}

#[cfg(not(feature = "channel-wework"))]
fn catalog_row(
    def: &ChannelDef,
    in_config: bool,
    runtime: Option<&super::status::ChannelRuntimeStatus>,
    config_path: &std::path::Path,
) -> Value {
    let fields = def
        .fields
        .iter()
        .map(|field| {
            let value = read_config_value(config_path, field.key)
                .unwrap_or_else(|| field.default_value.clone());
            let mut row = json!({
                "key": field.key,
                "label": localized(field.label_zh, field.label_en),
                "type": field.field_type,
                "value": value,
                "default": field.default_value,
            });
            if let (Some(zh), Some(en)) = (field.placeholder_zh, field.placeholder_en) {
                row["placeholder"] = localized(zh, en);
            }
            row
        })
        .collect::<Vec<_>>();

    let mut row = json!({
        "name": def.name,
        "label": localized(def.label_zh, def.label_en),
        "active": runtime
            .map(|status| phase_to_active(&status.phase))
            .unwrap_or(false),
        "fields": fields,
        "hint": localized(def.hint_zh, def.hint_en),
        "icon": def.icon,
        "color": def.color,
    });

    let login_status = runtime
        .and_then(|status| phase_to_login_status(&status.phase).map(str::to_string))
        .or_else(|| {
            if in_config {
                Some("unknown".to_string())
            } else {
                None
            }
        });
    if let Some(login_status) = login_status {
        row["login_status"] = Value::String(login_status.clone());
        row["loginStatus"] = Value::String(login_status);
    }

    row
}

/// Build the frontend channel catalog using Rust-owned definitions and runtime status.
///
/// # Arguments
///
/// * `app` - Tauri app handle used to read managed stores
/// * `config_path` - Shared bundled config path
///
/// # Returns
///
/// * `Value` - JSON object matching the frontend `ChannelCatalogEntry[]` contract
pub fn build_catalog(app: &AppHandle, config_path: &std::path::Path) -> Result<Value, String> {
    let active = active_channel_names(config_path)?;
    let status_store = app.state::<ChannelStatusStore>();
    #[cfg(feature = "channel-wework")]
    let wework_accounts = app.state::<WeworkAccountsStore>();

    let channels = all_channel_defs()
        .iter()
        .map(|def| {
            let in_config = active.iter().any(|name| name == def.name);
            let runtime = status_store.get(def.name).ok().flatten();
            #[cfg(feature = "channel-wework")]
            {
                catalog_row(
                    def,
                    in_config,
                    runtime.as_ref(),
                    config_path,
                    &wework_accounts,
                )
            }
            #[cfg(not(feature = "channel-wework"))]
            {
                catalog_row(def, in_config, runtime.as_ref(), config_path)
            }
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "status": "success",
        "channels": channels,
    }))
}

/// Validate a channel id against the desktop registry.
pub fn validate_channel_id(name: &str) -> Result<(), String> {
    if is_known_channel(name) {
        Ok(())
    } else {
        Err(format!(
            "{}: {name}",
            crate::services::channel::error_code::UNKNOWN_CHANNEL
        ))
    }
}
