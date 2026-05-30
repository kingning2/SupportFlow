//! `agent/tools/utils/truncate.py`

use serde_json::{json, Value};

pub const DEFAULT_MAX_LINES: usize = 2000;
pub const DEFAULT_MAX_BYTES: usize = 50 * 1024;
pub const GREP_MAX_LINE_LENGTH: usize = 500;

#[derive(Debug, Clone)]
pub struct TruncationResult {
    pub content: String,
    pub truncated: bool,
    pub truncated_by: Option<String>,
    pub total_lines: usize,
    pub total_bytes: usize,
    pub output_lines: usize,
    pub output_bytes: usize,
    pub last_line_partial: bool,
    pub first_line_exceeds_limit: bool,
    pub max_lines: usize,
    pub max_bytes: usize,
}

impl TruncationResult {
    pub fn to_value(&self) -> Value {
        json!({
            "content": self.content,
            "truncated": self.truncated,
            "truncated_by": self.truncated_by,
            "total_lines": self.total_lines,
            "total_bytes": self.total_bytes,
            "output_lines": self.output_lines,
            "output_bytes": self.output_bytes,
            "last_line_partial": self.last_line_partial,
            "first_line_exceeds_limit": self.first_line_exceeds_limit,
            "max_lines": self.max_lines,
            "max_bytes": self.max_bytes,
        })
    }
}

pub fn format_size(bytes_count: usize) -> String {
    if bytes_count < 1024 {
        format!("{bytes_count}B")
    } else if bytes_count < 1024 * 1024 {
        format!("{:.1}KB", bytes_count as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes_count as f64 / (1024.0 * 1024.0))
    }
}

pub fn truncate_head(
    content: &str,
    max_lines: Option<usize>,
    max_bytes: Option<usize>,
) -> TruncationResult {
    let max_lines = max_lines.unwrap_or(DEFAULT_MAX_LINES);
    let max_bytes = max_bytes.unwrap_or(DEFAULT_MAX_BYTES);

    let total_bytes = content.len();
    let lines: Vec<&str> = content.split('\n').collect();
    let total_lines = lines.len();

    if total_lines <= max_lines && total_bytes <= max_bytes {
        return TruncationResult {
            content: content.to_string(),
            truncated: false,
            truncated_by: None,
            total_lines,
            total_bytes,
            output_lines: total_lines,
            output_bytes: total_bytes,
            last_line_partial: false,
            first_line_exceeds_limit: false,
            max_lines,
            max_bytes,
        };
    }

    if !lines.is_empty() && lines[0].len() > max_bytes {
        return TruncationResult {
            content: String::new(),
            truncated: true,
            truncated_by: Some("bytes".into()),
            total_lines,
            total_bytes,
            output_lines: 0,
            output_bytes: 0,
            last_line_partial: false,
            first_line_exceeds_limit: true,
            max_lines,
            max_bytes,
        };
    }

    let mut output_lines_arr: Vec<&str> = Vec::new();
    let mut output_bytes_count = 0usize;
    let mut truncated_by = "lines".to_string();

    for (i, line) in lines.iter().enumerate() {
        if i >= max_lines {
            break;
        }
        let line_bytes = line.len() + if i > 0 { 1 } else { 0 };
        if output_bytes_count + line_bytes > max_bytes {
            truncated_by = "bytes".into();
            break;
        }
        output_lines_arr.push(line);
        output_bytes_count += line_bytes;
    }

    if output_lines_arr.len() >= max_lines && output_bytes_count <= max_bytes {
        truncated_by = "lines".into();
    }

    let output_content = output_lines_arr.join("\n");
    let final_output_bytes = output_content.len();

    TruncationResult {
        content: output_content,
        truncated: true,
        truncated_by: Some(truncated_by),
        total_lines,
        total_bytes,
        output_lines: output_lines_arr.len(),
        output_bytes: final_output_bytes,
        last_line_partial: false,
        first_line_exceeds_limit: false,
        max_lines,
        max_bytes,
    }
}

pub fn truncate_tail(
    content: &str,
    max_lines: Option<usize>,
    max_bytes: Option<usize>,
) -> TruncationResult {
    let max_lines = max_lines.unwrap_or(DEFAULT_MAX_LINES);
    let max_bytes = max_bytes.unwrap_or(DEFAULT_MAX_BYTES);

    let total_bytes = content.len();
    let lines: Vec<&str> = content.split('\n').collect();
    let total_lines = lines.len();

    if total_lines <= max_lines && total_bytes <= max_bytes {
        return TruncationResult {
            content: content.to_string(),
            truncated: false,
            truncated_by: None,
            total_lines,
            total_bytes,
            output_lines: total_lines,
            output_bytes: total_bytes,
            last_line_partial: false,
            first_line_exceeds_limit: false,
            max_lines,
            max_bytes,
        };
    }

    let mut output_lines_arr: Vec<&str> = Vec::new();
    let mut output_bytes_count = 0usize;
    let mut truncated_by = "lines".to_string();
    let mut last_line_partial = false;

    for i in (0..lines.len()).rev() {
        if output_lines_arr.len() >= max_lines {
            break;
        }
        let line = lines[i];
        let line_bytes = line.len() + if !output_lines_arr.is_empty() { 1 } else { 0 };

        if output_bytes_count + line_bytes > max_bytes {
            truncated_by = "bytes".into();
            if output_lines_arr.is_empty() {
                let truncated_line = truncate_string_to_bytes_from_end(line, max_bytes);
                output_bytes_count = truncated_line.len();
                output_lines_arr.insert(0, "");
                last_line_partial = true;
                return TruncationResult {
                    content: truncated_line,
                    truncated: true,
                    truncated_by: Some(truncated_by),
                    total_lines,
                    total_bytes,
                    output_lines: 1,
                    output_bytes: output_bytes_count,
                    last_line_partial,
                    first_line_exceeds_limit: false,
                    max_lines,
                    max_bytes,
                };
            }
            break;
        }

        output_lines_arr.insert(0, line);
        output_bytes_count += line_bytes;
    }

    if output_lines_arr.len() >= max_lines && output_bytes_count <= max_bytes {
        truncated_by = "lines".into();
    }

    let output_content = output_lines_arr.join("\n");
    let final_output_bytes = output_content.len();

    TruncationResult {
        content: output_content,
        truncated: true,
        truncated_by: Some(truncated_by),
        total_lines,
        total_bytes,
        output_lines: output_lines_arr.len(),
        output_bytes: final_output_bytes,
        last_line_partial,
        first_line_exceeds_limit: false,
        max_lines,
        max_bytes,
    }
}

fn truncate_string_to_bytes_from_end(text: &str, max_bytes: usize) -> String {
    let encoded = text.as_bytes();
    if encoded.len() <= max_bytes {
        return text.to_string();
    }
    let mut start = encoded.len().saturating_sub(max_bytes);
    while start < encoded.len() && encoded[start] & 0xC0 == 0x80 {
        start += 1;
    }
    String::from_utf8_lossy(&encoded[start..]).into_owned()
}

pub fn truncate_line(line: &str, max_chars: usize) -> (String, bool) {
    if line.chars().count() <= max_chars {
        return (line.to_string(), false);
    }
    let truncated: String = line.chars().take(max_chars).collect();
    (format!("{truncated}... [truncated]"), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_tail_keeps_end() {
        let content = (0..100)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let r = truncate_tail(&content, Some(5), None);
        assert!(r.truncated);
        assert!(r.content.contains("line99"));
    }
}
