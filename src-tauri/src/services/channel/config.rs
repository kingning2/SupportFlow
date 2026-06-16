//! Rust-owned channel config persistence and action orchestration.

use std::collections::{HashMap, HashSet};

use serde_json::{json, Map, Value};

use super::registry::{channel_field_type_map, channel_restart_keys, is_known_channel};

fn root_object(path: &std::path::Path) -> Result<Map<String, Value>, String> {
    let root = crate::config::provider_catalog::read_config_root(path)?;
    root.as_object()
        .cloned()
        .ok_or_else(|| "config root must be a JSON object".to_string())
}

fn write_root(path: &std::path::Path, root: Map<String, Value>) -> Result<(), String> {
    crate::config::provider_catalog::write_config_root(path, &Value::Object(root))
}

fn parse_channel_type(root: &Map<String, Value>) -> Vec<String> {
    crate::utils::channel::parse_desktop_channel_types(root.get("channel_type"))
}

fn bool_value(value: &Value) -> bool {
    value.as_bool().unwrap_or_else(|| {
        value
            .as_i64()
            .map(|v| v != 0)
            .or_else(|| value.as_str().map(|s| s == "true" || s == "1"))
            .unwrap_or(false)
    })
}

fn normalized_updates(
    channel: &str,
    config: &HashMap<String, Value>,
) -> Result<Map<String, Value>, String> {
    if !is_known_channel(channel) {
        return Err(format!(
            "{}: {channel}",
            super::contract::error_code::UNKNOWN_CHANNEL
        ));
    }
    let field_map = channel_field_type_map(channel).ok_or_else(|| {
        format!(
            "{}: {channel}",
            super::contract::error_code::UNKNOWN_CHANNEL
        )
    })?;
    let mut updates = Map::new();

    for (key, value) in config {
        let Some(field_type) = field_map.get(key.as_str()) else {
            continue;
        };
        let normalized = match *field_type {
            "bool" | "checkbox" => Value::Bool(bool_value(value)),
            "number" => Value::Number(serde_json::Number::from(value.as_i64().unwrap_or_default())),
            _ => value.clone(),
        };
        updates.insert(key.clone(), normalized);
    }

    if channel == "wework" {
        updates
            .entry("wework_version")
            .or_insert_with(|| Value::String("4.0.8.6027".into()));
        updates
            .entry("wework_init_wait_seconds")
            .or_insert_with(|| Value::Number(serde_json::Number::from(60)));
        updates
            .entry("wework_smart")
            .or_insert_with(|| Value::Bool(true));
    }

    Ok(updates)
}

/// Persist channel config updates from Rust and return applied keys.
///
/// # Arguments
///
/// * `config_path` - Shared config file path
/// * `channel` - Channel id
/// * `config` - Frontend submitted config values
///
/// # Returns
///
/// * `Vec<String>` - Keys written to config
pub fn persist_channel_config(
    config_path: &std::path::Path,
    channel: &str,
    config: &HashMap<String, Value>,
) -> Result<Vec<String>, String> {
    let updates = normalized_updates(channel, config)?;
    if updates.is_empty() {
        return Err(format!(
            "{}: no valid fields to update",
            super::contract::error_code::CONFIG_INVALID
        ));
    }
    let mut root = root_object(config_path)?;
    for (key, value) in &updates {
        root.insert(key.clone(), value.clone());
    }
    write_root(config_path, root)?;
    Ok(updates.keys().cloned().collect())
}

/// Persist channel connect state from Rust and return new configured channel list.
///
/// # Arguments
///
/// * `config_path` - Shared config file path
/// * `channel` - Channel id
/// * `config` - Frontend submitted config values
///
/// # Returns
///
/// * `(String, Vec<String>)` - New `channel_type` string and applied config keys
pub fn connect_channel(
    config_path: &std::path::Path,
    channel: &str,
    config: &HashMap<String, Value>,
) -> Result<(String, Vec<String>), String> {
    let updates = normalized_updates(channel, config)?;
    let mut root = root_object(config_path)?;
    for (key, value) in &updates {
        root.insert(key.clone(), value.clone());
    }
    let mut channel_names = parse_channel_type(&root);
    if !channel_names.iter().any(|name| name == channel) {
        channel_names.push(channel.to_string());
    }
    let channel_type = channel_names.join(",");
    root.insert("channel_type".into(), Value::String(channel_type.clone()));
    write_root(config_path, root)?;
    Ok((channel_type, updates.keys().cloned().collect()))
}

/// Persist channel disconnect state from Rust and return new configured channel list.
///
/// # Arguments
///
/// * `config_path` - Shared config file path
/// * `channel` - Channel id
///
/// # Returns
///
/// * `String` - New `channel_type` string
pub fn disconnect_channel(config_path: &std::path::Path, channel: &str) -> Result<String, String> {
    if !is_known_channel(channel) {
        return Err(format!(
            "{}: {channel}",
            super::contract::error_code::UNKNOWN_CHANNEL
        ));
    }
    let mut root = root_object(config_path)?;
    let channel_names = parse_channel_type(&root)
        .into_iter()
        .filter(|name| name != channel)
        .collect::<Vec<_>>();
    let channel_type = channel_names.join(",");
    root.insert("channel_type".into(), Value::String(channel_type.clone()));
    write_root(config_path, root)?;
    Ok(channel_type)
}

/// Return whether a saved config patch should trigger channel sidecar restart.
///
/// # Arguments
///
/// * `channel` - Channel id
/// * `applied_keys` - Keys written to config
///
/// # Returns
///
/// * `bool` - True when runtime restart is required
pub fn should_restart_channel(channel: &str, applied_keys: &[String]) -> bool {
    let Some(restart_keys) = channel_restart_keys(channel) else {
        return true;
    };
    let applied = applied_keys
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    restart_keys.iter().any(|key| applied.contains(key))
}

/// Build a JSON response matching the frontend action contract.
///
/// # Arguments
///
/// * `channel_type` - Updated channel_type string
/// * `restarted` - Whether runtime restart was requested
/// * `applied` - Keys applied to config
///
/// # Returns
///
/// * `Value` - JSON response payload
pub fn action_response(channel_type: String, restarted: bool, applied: Vec<String>) -> Value {
    json!({
        "status": "success",
        "channel_type": channel_type,
        "restarted": restarted,
        "applied": applied,
    })
}
