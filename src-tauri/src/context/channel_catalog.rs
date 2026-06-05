//! Rust-owned channel catalog for frontend rendering and runtime aggregation.

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::context::channel_status::ChannelStatusStore;
#[cfg(feature = "channel-wework")]
use crate::context::wework_accounts::WeworkAccountsStore;

struct ChannelFieldDef {
    key: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    field_type: &'static str,
    default_value: Value,
    placeholder_zh: Option<&'static str>,
    placeholder_en: Option<&'static str>,
}

struct ChannelDef {
    name: &'static str,
    label_zh: &'static str,
    label_en: &'static str,
    icon: &'static str,
    color: &'static str,
    hint_zh: &'static str,
    hint_en: &'static str,
    fields: &'static [ChannelFieldDef],
}

const WX_FIELDS: &[ChannelFieldDef] = &[ChannelFieldDef {
    key: "hot_reload",
    label_zh: "热重载登录",
    label_en: "Hot reload login",
    field_type: "bool",
    default_value: Value::Bool(false),
    placeholder_zh: None,
    placeholder_en: None,
}];

#[cfg(feature = "channel-wework")]
const WEWORK_FIELDS: &[ChannelFieldDef] = &[ChannelFieldDef {
    key: "wework_exe_path",
    label_zh: "企微程序路径",
    label_en: "WeCom executable path",
    field_type: "text",
    default_value: Value::String(String::new()),
    placeholder_zh: Some(r"如 D:\WXWork\4.0.8.6027\WXWork.exe"),
    placeholder_en: Some(r"e.g. D:\WXWork\4.0.8.6027\WXWork.exe"),
}];

const CHANNEL_DEFS: &[ChannelDef] = &[
    ChannelDef {
        name: "wx",
        label_zh: "个人微信",
        label_en: "Personal WeChat",
        icon: "fa-brands fa-weixin",
        color: "green",
        hint_zh: "基于 itchat 网页协议，存在封号风险，仅建议测试。支持私聊与群聊；登录态保存在数据目录 itchat.pkl。",
        hint_en: "itchat web protocol; account risk; test use only. Supports DM and groups.",
        fields: WX_FIELDS,
    },
    #[cfg(feature = "channel-wework")]
    ChannelDef {
        name: "wework",
        label_zh: "企微个人号",
        label_en: "WeCom Desktop",
        icon: "fa-desktop",
        color: "emerald",
        hint_zh: "需 Windows 与企业微信 PC 4.0.8.6027；仅当本机有多个安装版本时再改路径。",
        hint_en: "Windows + WeCom 4.0.8.6027; change path only if multiple installs.",
        fields: WEWORK_FIELDS,
    },
];

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
    match phase {
        "starting" | "waiting_login" | "waiting_scan" => Some("waiting_scan"),
        "scanned" => Some("scanned"),
        "logged_in" | "syncing" | "ready" => Some("logged_in"),
        "error" | "stopped" => Some("unknown"),
        _ => None,
    }
}

fn phase_to_active(phase: &str) -> bool {
    matches!(phase, "ready")
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

    let channels = CHANNEL_DEFS
        .iter()
        .map(|def| {
            let in_config = active.iter().any(|name| name == def.name);
            let runtime = status_store.get(def.name).ok().flatten();

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
                    .as_ref()
                    .map(|status| phase_to_active(&status.phase))
                    .unwrap_or(false),
                "fields": fields,
                "hint": localized(def.hint_zh, def.hint_en),
                "icon": def.icon,
                "color": def.color,
            });

            let login_status = runtime
                .as_ref()
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

            #[cfg(feature = "channel-wework")]
            if def.name == "wework" {
                if let Some(status) = runtime.as_ref() {
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
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "status": "success",
        "channels": channels,
    }))
}
