//! Channel adapter catalog: capabilities, config schema, and restart policy.
//!
//! New desktop channels register here only — `ProcessHub` and sidecar RPC stay generic.

use std::collections::HashMap;

use serde_json::{json, Value};

/// Adapter capability matrix (see `docs/channel-adapter-contract.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelCapability {
    Connect,
    Disconnect,
    ListConversations,
    Send,
    OnMessage,
    Health,
}

#[derive(Debug, Clone)]
pub struct ChannelFieldDef {
    pub key: &'static str,
    pub label_zh: &'static str,
    pub label_en: &'static str,
    pub field_type: &'static str,
    pub default_value: Value,
    pub placeholder_zh: Option<&'static str>,
    pub placeholder_en: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct ChannelDef {
    pub name: &'static str,
    pub label_zh: &'static str,
    pub label_en: &'static str,
    pub icon: &'static str,
    pub color: &'static str,
    pub hint_zh: &'static str,
    pub hint_en: &'static str,
    pub fields: &'static [ChannelFieldDef],
    pub restart_keys: &'static [&'static str],
    pub capabilities: &'static [ChannelCapability],
}

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

#[cfg(feature = "channel-wework")]
const WEWORK_RESTART_KEYS: &[&str] = &[
    "wework_exe_path",
    "wework_version",
    "wework_smart",
    "wework_init_wait_seconds",
];

const DESKTOP_CAPABILITIES: &[ChannelCapability] = &[
    ChannelCapability::Connect,
    ChannelCapability::Disconnect,
    ChannelCapability::ListConversations,
    ChannelCapability::Send,
    ChannelCapability::OnMessage,
    ChannelCapability::Health,
];

const CHANNEL_DEFS: &[ChannelDef] = &[
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
        restart_keys: WEWORK_RESTART_KEYS,
        capabilities: DESKTOP_CAPABILITIES,
    },
];

/// All registered desktop channel definitions.
pub fn all_channel_defs() -> &'static [ChannelDef] {
    CHANNEL_DEFS
}

/// Lookup one channel definition by id (`wework`, …).
pub fn channel_def(name: &str) -> Option<&'static ChannelDef> {
    CHANNEL_DEFS.iter().find(|def| def.name == name)
}

/// Whether the id is registered in the desktop catalog.
pub fn is_known_channel(name: &str) -> bool {
    channel_def(name).is_some()
}

/// Config field types for persistence validation (`key` → `text` | `bool` | `number`).
pub fn channel_field_type_map(name: &str) -> Option<HashMap<&'static str, &'static str>> {
    let def = channel_def(name)?;
    Some(
        def.fields
            .iter()
            .map(|field| (field.key, field.field_type))
            .collect(),
    )
}

/// Keys that require a sidecar restart when changed during `save`.
pub fn channel_restart_keys(name: &str) -> Option<&'static [&'static str]> {
    channel_def(name).map(|def| def.restart_keys)
}

/// JSON Schema-like object for frontend tooling (MVP: field list + capabilities).
pub fn config_schema_for(name: &str) -> Option<Value> {
    let def = channel_def(name)?;
    let fields = def
        .fields
        .iter()
        .map(|field| {
            let mut row = json!({
                "key": field.key,
                "type": field.field_type,
                "default": field.default_value,
                "label": { "zh": field.label_zh, "en": field.label_en },
            });
            if let (Some(zh), Some(en)) = (field.placeholder_zh, field.placeholder_en) {
                row["placeholder"] = json!({ "zh": zh, "en": en });
            }
            row
        })
        .collect::<Vec<_>>();
    let capabilities = def
        .capabilities
        .iter()
        .map(capability_name)
        .collect::<Vec<_>>();
    Some(json!({
        "channel_type": def.name,
        "capabilities": capabilities,
        "fields": fields,
    }))
}

fn capability_name(cap: &ChannelCapability) -> &'static str {
    match cap {
        ChannelCapability::Connect => "connect",
        ChannelCapability::Disconnect => "disconnect",
        ChannelCapability::ListConversations => "list_conversations",
        ChannelCapability::Send => "send",
        ChannelCapability::OnMessage => "on_message",
        ChannelCapability::Health => "health",
    }
}
