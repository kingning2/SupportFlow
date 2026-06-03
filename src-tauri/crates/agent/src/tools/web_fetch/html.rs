//! HTML title/text extraction and charset detection.

use regex::Regex;

const META_CHARSET: &str = r#"(?i)<meta[^>]+charset\s*=\s*["']?\s*([\w-]+)"#;
const META_HTTP_EQUIV: &str = r#"(?i)<meta[^>]+http-equiv\s*=\s*["']?Content-Type["']?[^>]+content\s*=\s*["'][^"']*charset=([\w-]+)"#;
const TITLE: &str = r"(?is)<title[^>]*>(.*?)</title>";

pub fn extract_charset_from_content_type(content_type: &str) -> Option<String> {
    let re = Regex::new(r#"(?i)charset\s*=\s*["']?\s*([\w-]+)"#).ok()?;
    re.captures(content_type)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn extract_charset_from_html_meta(raw: &[u8]) -> Option<String> {
    let head = String::from_utf8_lossy(&raw[..raw.len().min(4096)]);
    let re = Regex::new(META_CHARSET).ok()?;
    if let Some(caps) = re.captures(&head) {
        return caps.get(1).map(|m| m.as_str().to_string());
    }
    let re = Regex::new(META_HTTP_EQUIV).ok()?;
    re.captures(&head)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Priority: Content-Type charset → HTML meta → chardetng → utf-8 (`web_fetch._detect_encoding`).
pub fn detect_encoding(bytes: &[u8], content_type: &str) -> String {
    if let Some(charset) = extract_charset_from_content_type(content_type) {
        return charset;
    }
    if let Some(charset) = extract_charset_from_html_meta(bytes) {
        return charset;
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    let name = encoding.name().to_lowercase();
    let trusted = [
        "utf",
        "gb",
        "big5",
        "euc",
        "shift_jis",
        "iso-2022",
        "windows",
        "ascii",
    ];
    if trusted.iter().any(|p| name.starts_with(p)) {
        return encoding.name().to_string();
    }
    "utf-8".into()
}

pub fn decode_bytes(bytes: &[u8], charset: Option<&str>) -> String {
    if let Some(label) = charset {
        if let Some(encoding) = encoding_rs::Encoding::for_label(label.as_bytes()) {
            let (decoded, _, _) = encoding.decode(bytes);
            return decoded.into_owned();
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn extract_title(html: &str) -> String {
    let re = match Regex::new(TITLE) {
        Ok(r) => r,
        Err(_) => return "Untitled".into(),
    };
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Untitled".into())
}

pub fn extract_text(html: &str) -> String {
    let script = Regex::new(r"(?is)<script[^>]*>.*?</script>").ok();
    let style = Regex::new(r"(?is)<style[^>]*>.*?</style>").ok();
    let tags = Regex::new(r"<[^>]+>").ok();

    let mut text = html.to_string();
    if let Some(re) = script {
        text = re.replace_all(&text, "").into_owned();
    }
    if let Some(re) = style {
        text = re.replace_all(&text, "").into_owned();
    }
    if let Some(re) = tags {
        text = re.replace_all(&text, "").into_owned();
    }

    text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    if let Ok(re) = Regex::new(r"[^\S\n]+") {
        text = re.replace_all(&text, " ").into_owned();
    }
    if let Ok(re) = Regex::new(r"\n{3,}") {
        text = re.replace_all(&text, "\n\n").into_owned();
    }

    text.lines()
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_title_and_strips_tags() {
        let html = "<html><head><title>Hello</title></head><body><p>World</p></body></html>";
        assert_eq!(extract_title(html), "Hello");
        assert!(extract_text(html).contains("World"));
    }
}
