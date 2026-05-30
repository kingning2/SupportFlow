//! `agent/tools/utils/diff.py`

use regex::Regex;

#[derive(Debug, Clone)]
pub struct FuzzyMatchResult {
    pub found: bool,
    pub index: usize,
    pub match_length: usize,
    pub content_for_replacement: String,
}

pub fn strip_bom(text: &str) -> (String, String) {
    if let Some(stripped) = text.strip_prefix('\u{feff}') {
        ("\u{feff}".to_string(), stripped.to_string())
    } else {
        (String::new(), text.to_string())
    }
}

pub fn detect_line_ending(text: &str) -> String {
    if text.contains("\r\n") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

pub fn normalize_to_lf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn restore_line_endings(text: &str, original_ending: &str) -> String {
    if original_ending == "\r\n" {
        text.replace('\n', "\r\n")
    } else {
        text.to_string()
    }
}

pub fn normalize_for_fuzzy_match(text: &str) -> String {
    let re_spaces = Regex::new(r"[ \t]+").expect("regex");
    let re_trail = Regex::new(r" +\n").expect("regex");
    let mut text = re_spaces.replace_all(text, " ").to_string();
    text = re_trail.replace_all(&text, "\n").to_string();
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| {
            let stripped = line.trim_start();
            if stripped.is_empty() {
                String::new()
            } else {
                let indent_count = line.len() - stripped.len();
                format!("{}{}", " ".repeat(indent_count), stripped)
            }
        })
        .collect();
    lines.join("\n")
}

pub fn fuzzy_find_text(content: &str, old_text: &str) -> FuzzyMatchResult {
    if let Some(index) = content.find(old_text) {
        return FuzzyMatchResult {
            found: true,
            index,
            match_length: old_text.len(),
            content_for_replacement: content.to_string(),
        };
    }

    let fuzzy_content = normalize_for_fuzzy_match(content);
    let fuzzy_old = normalize_for_fuzzy_match(old_text);
    if let Some(index) = fuzzy_content.find(&fuzzy_old) {
        return FuzzyMatchResult {
            found: true,
            index,
            match_length: fuzzy_old.len(),
            content_for_replacement: fuzzy_content,
        };
    }

    FuzzyMatchResult {
        found: false,
        index: 0,
        match_length: 0,
        content_for_replacement: String::new(),
    }
}

#[derive(Debug, Clone)]
pub struct DiffOutput {
    pub diff: String,
    pub first_changed_line: Option<u32>,
}

pub fn generate_diff_string(old_content: &str, new_content: &str) -> DiffOutput {
    let diff = similar::TextDiff::from_lines(old_content, new_content);
    let diff_string = diff.unified_diff().to_string();

    let re = Regex::new(r"@@ -\d+,?\d* \+(\d+)").expect("regex");
    let first_changed_line = diff_string.lines().find_map(|line| {
        re.captures(line)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok())
    });

    DiffOutput {
        diff: diff_string,
        first_changed_line,
    }
}
