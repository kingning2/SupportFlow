use serde_json::Value;
use std::collections::HashSet;

const DESKTOP_EXCLUDED_CHANNELS: &[&str] = &["web", "terminal"];

/// 解析配置里的 channel_type，支持逗号字符串和数组两种格式。
pub fn parse_channel_types(value: Option<&Value>) -> Vec<String> {
    let items: Vec<String> = match value {
        Some(Value::String(raw)) => raw
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    unique_channel_names(items)
}

/// 解析桌面端可展示的渠道，过滤 Python 服务端专用渠道。
pub fn parse_desktop_channel_types(value: Option<&Value>) -> Vec<String> {
    parse_channel_types(value)
        .into_iter()
        .filter(|name| !DESKTOP_EXCLUDED_CHANNELS.contains(&name.as_str()))
        .collect()
}

fn unique_channel_names(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|name| seen.insert(name.clone()))
        .collect()
}
