use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelRuntimeContext {
    pub channel_type: String,
    pub is_group: bool,
    pub content: String,
    pub actual_user_nickname: Option<String>,
    pub no_need_at: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelRuntimeConfig {
    pub group_chat_prefix: Vec<String>,
    pub group_chat_keyword: Vec<String>,
    pub single_chat_prefix: Vec<String>,
    pub group_chat_reply_prefix: String,
    pub group_chat_reply_suffix: String,
    pub single_chat_reply_prefix: String,
    pub single_chat_reply_suffix: String,
    pub image_create_prefix: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRuntimeResult {
    pub should_handle: bool,
    pub normalized_content: String,
    pub reply_prefix: String,
    pub reply_suffix: String,
    pub mention_prefix: String,
}

impl Default for ChannelRuntimeResult {
    fn default() -> Self {
        Self {
            should_handle: false,
            normalized_content: String::new(),
            reply_prefix: String::new(),
            reply_suffix: String::new(),
            mention_prefix: String::new(),
        }
    }
}

pub fn process_message(
    ctx: &ChannelRuntimeContext,
    cfg: &ChannelRuntimeConfig,
) -> ChannelRuntimeResult {
    let mut out = ChannelRuntimeResult::default();
    let mut content = ctx.content.trim().to_string();
    if content.is_empty() {
        return out;
    }

    if ctx.is_group {
        if !should_handle_group_message(&content, cfg) {
            return out;
        }
        content = strip_prefix_if_needed(content, &cfg.group_chat_prefix);
        out.reply_prefix = cfg.group_chat_reply_prefix.clone();
        out.reply_suffix = cfg.group_chat_reply_suffix.clone();
        out.mention_prefix = build_mention_prefix(ctx);
    } else {
        if !prefix_matches_all(&cfg.single_chat_prefix)
            && find_prefix(&content, &cfg.single_chat_prefix).is_none()
        {
            return out;
        }
        content = strip_prefix_if_needed(content, &cfg.single_chat_prefix);
        out.reply_prefix = cfg.single_chat_reply_prefix.clone();
        out.reply_suffix = cfg.single_chat_reply_suffix.clone();
    }

    if let Some(prefix) = find_prefix(&content, &cfg.image_create_prefix) {
        content = content.replacen(prefix, "", 1).trim().to_string();
    }

    out.should_handle = !content.is_empty();
    out.normalized_content = content;
    out
}

pub fn extract_media_urls(text: &str, limit: usize) -> Vec<(String, String)> {
    static IMAGE_RE: OnceLock<Regex> = OnceLock::new();
    static VIDEO_RE: OnceLock<Regex> = OnceLock::new();
    static MD_IMAGE_RE: OnceLock<Regex> = OnceLock::new();
    static IMG_TAG_RE: OnceLock<Regex> = OnceLock::new();

    let image_re = IMAGE_RE
        .get_or_init(|| Regex::new(r#"https?://[^\s]+\.(?:jpg|jpeg|png|gif|webp)"#).unwrap());
    let video_re =
        VIDEO_RE.get_or_init(|| Regex::new(r#"https?://[^\s]+\.(?:mp4|avi|mov|wmv|flv)"#).unwrap());
    let md_image_re = MD_IMAGE_RE.get_or_init(|| Regex::new(r#"!\[.*?\]\(([^)]+)\)"#).unwrap());
    let img_tag_re =
        IMG_TAG_RE.get_or_init(|| Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).unwrap());

    let mut out: Vec<(String, String)> = Vec::new();
    for m in image_re.find_iter(text) {
        push_unique(&mut out, (m.as_str().to_string(), "image".into()), limit);
    }
    for m in video_re.find_iter(text) {
        push_unique(&mut out, (m.as_str().to_string(), "video".into()), limit);
    }
    for cap in md_image_re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            push_unique(&mut out, (m.as_str().to_string(), "image".into()), limit);
        }
    }
    for cap in img_tag_re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            push_unique(&mut out, (m.as_str().to_string(), "image".into()), limit);
        }
    }
    out
}

pub fn decorate_text(raw: &str, meta: &ChannelRuntimeResult) -> String {
    format!(
        "{}{}{}{}",
        meta.reply_prefix, meta.mention_prefix, raw, meta.reply_suffix
    )
}

fn push_unique(out: &mut Vec<(String, String)>, item: (String, String), limit: usize) {
    if out.len() >= limit {
        return;
    }
    if out.iter().any(|(u, _)| u == &item.0) {
        return;
    }
    out.push(item);
}

/// `prefixes` contains `""` — match every message (SupportFlow Agent-style config).
fn prefix_matches_all(prefixes: &[String]) -> bool {
    prefixes.iter().any(|p| p.is_empty())
}

fn should_handle_group_message(content: &str, cfg: &ChannelRuntimeConfig) -> bool {
    if prefix_matches_all(&cfg.group_chat_prefix) {
        return true;
    }

    if find_prefix(content, &cfg.group_chat_prefix).is_some() {
        return true;
    }

    contains_keyword(content, &cfg.group_chat_keyword)
}

fn strip_prefix_if_needed(content: String, prefixes: &[String]) -> String {
    if prefix_matches_all(prefixes) {
        return content;
    }

    let Some(prefix) = find_prefix(&content, prefixes) else {
        return content;
    };

    content.replacen(prefix, "", 1).trim().to_string()
}

fn build_mention_prefix(ctx: &ChannelRuntimeContext) -> String {
    if ctx.no_need_at {
        return String::new();
    }

    let Some(name) = &ctx.actual_user_nickname else {
        return String::new();
    };
    if name.is_empty() {
        return String::new();
    }

    format!("@{}\n", name)
}

fn find_prefix<'a>(content: &'a str, prefixes: &'a [String]) -> Option<&'a str> {
    prefixes
        .iter()
        .filter(|p| !p.is_empty())
        .find(|p| content.starts_with(p.as_str()))
        .map(|p| p.as_str())
}

fn contains_keyword(content: &str, keywords: &[String]) -> bool {
    keywords
        .iter()
        .filter(|k| !k.is_empty())
        .any(|k| content.contains(k))
}
