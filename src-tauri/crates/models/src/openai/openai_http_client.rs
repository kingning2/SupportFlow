//! `models/openai/openai_http_client.py` — OpenAI-compatible HTTP + SSE.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::Stream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Map, Value};
use tracing::debug;

use super::openai_compat::OpenAiHttpError;

pub const DEFAULT_API_BASE: &str = "https://api.openai.com/v1";
pub const DEFAULT_TIMEOUT_SECS: u64 = 600;

const APP_TITLE: &str = "SupportFlow";
const APP_REFERER: &str = "https://github.com/kingning2/SupportFlow";

/// Minimal HTTP client for OpenAI-compatible chat completions.
#[derive(Debug, Clone)]
pub struct OpenAiHttpClient {
    client: reqwest::Client,
    api_key: Option<String>,
    api_base: String,
    timeout_secs: u64,
    extra_headers: HeaderMap,
}

impl Default for OpenAiHttpClient {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl OpenAiHttpClient {
    pub fn new(api_key: Option<String>, api_base: Option<String>) -> Self {
        let timeout = std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("reqwest client");
        Self {
            client,
            api_key,
            api_base: api_base
                .unwrap_or_else(|| DEFAULT_API_BASE.to_string())
                .trim_end_matches('/')
                .to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            extra_headers: HeaderMap::new(),
        }
    }

    pub fn with_timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_extra_headers(mut self, headers: HeaderMap) -> Self {
        self.extra_headers = headers;
        self
    }

    /// `POST /chat/completions` — non-streaming.
    pub async fn chat_completions(
        &self,
        payload: Map<String, Value>,
        api_key: Option<&str>,
        api_base: Option<&str>,
        timeout_secs: Option<u64>,
    ) -> Result<Value, OpenAiHttpError> {
        let mut body = payload;
        body.insert("stream".into(), Value::Bool(false));
        self.post_json(
            "/chat/completions",
            body,
            api_key,
            api_base,
            timeout_secs,
            false,
        )
        .await
    }

    /// `POST /chat/completions` — streaming SSE.
    pub async fn chat_completions_stream(
        &self,
        payload: Map<String, Value>,
        api_key: Option<&str>,
        api_base: Option<&str>,
        timeout_secs: Option<u64>,
    ) -> Result<SseChunkStream, OpenAiHttpError> {
        let mut body = payload;
        body.insert("stream".into(), Value::Bool(true));
        let url = self.url("/chat/completions", api_base);
        let headers = self.build_headers(api_key, &url)?;
        let timeout = std::time::Duration::from_secs(timeout_secs.unwrap_or(self.timeout_secs));

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|e| connection_error(e.to_string()))?;

        let status = resp.status().as_u16();
        if status >= 400 {
            let body = resp.json().await.unwrap_or(Value::Null);
            return Err(OpenAiHttpError::from_response(status, body));
        }

        Ok(SseChunkStream::new(resp.bytes_stream()))
    }

    async fn post_json(
        &self,
        path: &str,
        payload: Map<String, Value>,
        api_key: Option<&str>,
        api_base: Option<&str>,
        timeout_secs: Option<u64>,
        _stream: bool,
    ) -> Result<Value, OpenAiHttpError> {
        let url = self.url(path, api_base);
        let headers = self.build_headers(api_key, &url)?;
        let timeout = std::time::Duration::from_secs(timeout_secs.unwrap_or(self.timeout_secs));
        let clean: Map<String, Value> = payload.into_iter().filter(|(_, v)| !v.is_null()).collect();

        let resp = self
            .client
            .post(&url)
            .headers(headers)
            .timeout(timeout)
            .json(&clean)
            .send()
            .await
            .map_err(|e| connection_error(e.to_string()))?;

        let status = resp.status().as_u16();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        if status >= 400 {
            return Err(OpenAiHttpError::from_response(status, body));
        }
        Ok(body)
    }

    fn url(&self, path: &str, api_base: Option<&str>) -> String {
        let base = api_base
            .map(|s| s.trim_end_matches('/'))
            .unwrap_or(self.api_base.as_str());
        if path.starts_with('/') {
            format!("{base}{path}")
        } else {
            format!("{base}/{path}")
        }
    }

    fn build_headers(
        &self,
        api_key: Option<&str>,
        url: &str,
    ) -> Result<HeaderMap, OpenAiHttpError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let key = api_key.map(str::to_string).or_else(|| self.api_key.clone());
        if let Some(k) = key.filter(|s| !s.is_empty()) {
            let auth = format!("Bearer {k}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth).map_err(|e| connection_error(e.to_string()))?,
            );
        }
        for (k, v) in attribution_headers(url) {
            headers.insert(k, v);
        }
        headers.extend(self.extra_headers.clone());
        Ok(headers)
    }
}

fn connection_error(message: String) -> OpenAiHttpError {
    OpenAiHttpError {
        status_code: 0,
        body: Value::Null,
        message,
    }
}

fn attribution_headers(url: &str) -> Vec<(reqwest::header::HeaderName, HeaderValue)> {
    let host = url
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("")
        .to_lowercase();
    let mut out = Vec::new();
    if host == "openrouter.ai" || host.ends_with(".openrouter.ai") {
        push_header(&mut out, "http-referer", APP_REFERER);
        push_header(&mut out, "x-title", APP_TITLE);
    }
    if host == "ai-gateway.vercel.sh" || host.ends_with(".ai-gateway.vercel.sh") {
        push_header(&mut out, "http-referer", APP_REFERER);
        push_header(&mut out, "x-title", APP_TITLE);
    }
    out
}

fn push_header(out: &mut Vec<(reqwest::header::HeaderName, HeaderValue)>, name: &str, value: &str) {
    if let (Ok(n), Ok(v)) = (
        reqwest::header::HeaderName::from_bytes(name.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        out.push((n, v));
    }
}

/// Stream of parsed SSE JSON chunks (OpenAI chat completion stream shape).
pub struct SseChunkStream {
    byte_stream: Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>,
    buf: Vec<u8>,
    finished: bool,
}

impl SseChunkStream {
    fn new(
        byte_stream: impl Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
    ) -> Self {
        Self {
            byte_stream: Box::pin(byte_stream),
            buf: Vec::new(),
            finished: false,
        }
    }

    fn poll_next_event(&mut self, cx: &mut Context<'_>) -> Poll<Option<Value>> {
        loop {
            if let Some(event) = take_sse_event(&mut self.buf) {
                if event == "[DONE]" {
                    return Poll::Ready(None);
                }
                if event.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(&event) {
                    Ok(chunk) => return Poll::Ready(Some(chunk)),
                    Err(e) => {
                        debug!(event = %event.chars().take(200).collect::<String>(), err = %e, "skip malformed SSE chunk");
                        continue;
                    }
                }
            }

            if self.finished {
                if let Some(event) = flush_sse_remainder(&mut self.buf) {
                    if event == "[DONE]" {
                        return Poll::Ready(None);
                    }
                    if let Ok(chunk) = serde_json::from_str(&event) {
                        return Poll::Ready(Some(chunk));
                    }
                }
                return Poll::Ready(None);
            }

            match self.byte_stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => self.buf.extend_from_slice(&bytes),
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(error_chunk(0, format!("Stream error: {e}"))));
                }
                Poll::Ready(None) => self.finished = true,
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl Stream for SseChunkStream {
    type Item = Value;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_event(cx)
    }
}

fn take_sse_event(buf: &mut Vec<u8>) -> Option<String> {
    let (idx, term_len) = find_event_terminator(buf)?;
    let event_bytes = buf.drain(..idx).collect::<Vec<_>>();
    buf.drain(..term_len.min(buf.len()));
    decode_sse_data_payload(&event_bytes)
}

fn flush_sse_remainder(buf: &mut Vec<u8>) -> Option<String> {
    if buf.is_empty() {
        return None;
    }
    let rest = std::mem::take(buf);
    decode_sse_data_payload(&rest)
}

fn find_event_terminator(buf: &[u8]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    for (idx, len) in [
        (buf.windows(4).position(|w| w == b"\r\n\r\n"), 4),
        (buf.windows(2).position(|w| w == b"\n\n"), 2),
        (buf.windows(2).position(|w| w == b"\r\r"), 2),
    ] {
        if let Some(i) = idx {
            best = match best {
                Some((b, _)) if i < b => best,
                _ => Some((i, len)),
            };
        }
    }
    best
}

fn decode_sse_data_payload(event_bytes: &[u8]) -> Option<String> {
    let event_text = String::from_utf8_lossy(event_bytes).into_owned();
    let mut data_lines = Vec::new();
    for line in event_text.lines() {
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.trim_start()),
            None => continue,
        };
        if field == "data" {
            data_lines.push(value.to_string());
        }
    }
    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}

fn error_chunk(status_code: u16, message: String) -> Value {
    json!({
        "error": true,
        "message": message,
        "status_code": status_code
    })
}
