//! Simple YAML frontmatter parser (`agent/skills/frontmatter.py` subset).

use std::collections::HashMap;

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
