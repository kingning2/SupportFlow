//! Simple YAML frontmatter parser (`agent/skills/frontmatter.py` subset).

use std::collections::HashMap;

use serde_json::Value;

pub fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Some(body) = content.strip_prefix("---") else {
        return map;
    };
    let Some(rest) = body
        .strip_prefix('\n')
        .or_else(|| body.strip_prefix("\r\n"))
    else {
        return map;
    };
    let end = rest.find("\n---").or_else(|| rest.find("\r\n---"));
    let Some(fm_text) = end.map(|i| &rest[..i]) else {
        return map;
    };

    for line in fm_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }
    map
}

pub fn body_after_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    if let Some(idx) = content[3..].find("\n---") {
        let start = 3 + idx + 4;
        return content.get(start..).unwrap_or("").trim_start();
    }
    content
}

/// Parse optional JSON Schema parameters from frontmatter `parameters` field.
pub fn parse_parameters(fm: &HashMap<String, String>) -> Result<Option<Value>, String> {
    let Some(raw) = fm.get("parameters").filter(|s| !s.trim().is_empty()) else {
        return Ok(None);
    };
    let value: Value =
        serde_json::from_str(raw).map_err(|e| format!("invalid parameters JSON: {e}"))?;
    if !value.is_object() && !value.is_array() {
        return Err("parameters must be a JSON object or array schema".into());
    }
    Ok(Some(value))
}

/// Resolve skill version from frontmatter (default `1.0.0`).
pub fn parse_version(fm: &HashMap<String, String>) -> String {
    fm.get("version")
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "1.0.0".to_string())
}

/// Split `name@version` spec into parts.
pub fn parse_skill_ref(spec: &str) -> (&str, Option<&str>) {
    if let Some((name, ver)) = spec.rsplit_once('@') {
        if !ver.is_empty() {
            return (name, Some(ver));
        }
    }
    (spec, None)
}
