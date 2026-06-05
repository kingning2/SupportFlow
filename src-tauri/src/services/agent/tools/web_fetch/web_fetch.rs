//! `agent/tools/web_fetch/web_fetch.py`

use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::StreamExt;
use models::ModelsConfig;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{json, Value};
use tracing::info;
use url::Url;

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::web_fetch::document::{
    cleanup_file, format_document_result, is_binary_content_type, is_document_url,
    is_supported_doc_suffix, parse_document_file, rewrite_url_with_suffix, safe_filename,
    suffix_from_content_type, url_suffix, MAX_FILE_SIZE,
};
use crate::services::agent::tools::web_fetch::html::{
    decode_bytes, detect_encoding, extract_text, extract_title,
};
use crate::services::agent::utils::{
    build_reqwest_client, log_http_proxy_settings, HttpProxySettings,
};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const USER_AGENT_STR: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct WebFetchTool {
    cwd: PathBuf,
    client: reqwest::Client,
}

impl WebFetchTool {
    pub fn new(cwd: PathBuf) -> Self {
        Self::with_proxy(cwd, &HttpProxySettings::default())
    }

    pub fn with_models_config(cwd: PathBuf, config: &ModelsConfig) -> Self {
        let proxy = HttpProxySettings::from_models(config);
        log_http_proxy_settings(&proxy);
        Self::with_proxy(cwd, &proxy)
    }

    fn with_proxy(cwd: PathBuf, proxy: &HttpProxySettings) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STR));
        headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("*/*"));

        let client = build_reqwest_client(
            proxy,
            Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            Some(headers),
        );

        Self { cwd, client }
    }

    fn tmp_dir(&self) -> PathBuf {
        let dir = self.cwd.join("tmp");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    async fn fetch_webpage(&self, url: &str) -> Result<String, String> {
        let parsed = Url::parse(url).map_err(|_| "Invalid URL".to_string())?;
        let host = parsed.host_str().unwrap_or("").to_string();

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| map_reqwest_error(&e, &host, false))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Error: HTTP {} for URL: {url}", status.as_u16()));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        if is_binary_content_type(&content_type) && !is_document_url(url) {
            return self
                .handle_download_by_content_type(url, &content_type)
                .await;
        }

        let charset = detect_encoding(&bytes, &content_type);
        let html = decode_bytes(&bytes, Some(&charset));
        let title = extract_title(&html);
        let text = extract_text(&html);

        Ok(format!("Title: {title}\n\nContent:\n{text}"))
    }

    async fn handle_download_by_content_type(
        &self,
        url: &str,
        content_type: &str,
    ) -> Result<String, String> {
        let Some(detected_suffix) = suffix_from_content_type(content_type) else {
            return Err(format!(
                "Error: URL returned binary content ({content_type}), not a supported document type"
            ));
        };
        if !is_supported_doc_suffix(detected_suffix) {
            return Err(format!(
                "Error: URL returned binary content ({content_type}), not a supported document type"
            ));
        }
        let doc_url = if is_document_url(url) {
            url.to_string()
        } else {
            rewrite_url_with_suffix(url, detected_suffix)
        };
        self.fetch_document(&doc_url).await
    }

    async fn fetch_document(&self, url: &str) -> Result<String, String> {
        let suffix = url_suffix(url);
        let parsed = Url::parse(url).map_err(|_| "Invalid URL".to_string())?;
        let host = parsed.host_str().unwrap_or("").to_string();
        let filename = safe_filename(url);
        let local_path = self.tmp_dir().join(&filename);

        info!(%url, path = %local_path.display(), "WebFetch downloading document");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| map_reqwest_error(&e, &host, true))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Error: HTTP {} for URL: {url}", status.as_u16()));
        }

        if let Some(len) = response.content_length() {
            if len as usize > MAX_FILE_SIZE {
                return Err(format!(
                    "Error: File too large ({} > {})",
                    crate::services::agent::tools::utils::format_size(len as usize),
                    crate::services::agent::tools::utils::format_size(MAX_FILE_SIZE)
                ));
            }
        }

        let mut stream = response.bytes_stream();
        let mut downloaded: usize = 0;
        let mut file = tokio::fs::File::create(&local_path)
            .await
            .map_err(|e| format!("write file: {e}"))?;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("download stream: {e}"))?;
            downloaded += chunk.len();
            if downloaded > MAX_FILE_SIZE {
                drop(file);
                cleanup_file(&local_path);
                return Err(format!(
                    "Error: File too large (>{}), download aborted",
                    crate::services::agent::tools::utils::format_size(MAX_FILE_SIZE)
                ));
            }
            use tokio::io::AsyncWriteExt;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("write file: {e}"))?;
        }

        let text = match parse_document_file(&local_path, &suffix) {
            Ok(t) => t,
            Err(e) => {
                cleanup_file(&local_path);
                return Err(format!("Error: Failed to parse document: {e}"));
            }
        };

        if text.trim().is_empty() {
            return Ok(format_document_result(&filename, &local_path, ""));
        }

        Ok(format_document_result(&filename, &local_path, &text))
    }
}

fn map_reqwest_error(e: &reqwest::Error, host: &str, is_download: bool) -> String {
    if e.is_timeout() {
        if is_download {
            return format!("Error: Download timed out after {DEFAULT_TIMEOUT_SECS}s");
        }
        return format!("Error: Request timed out after {DEFAULT_TIMEOUT_SECS}s");
    }
    if e.is_connect() {
        return format!("Error: Failed to connect to {host}");
    }
    if is_download {
        return format!("Error: Failed to download file: {e}");
    }
    format!("Error: Failed to fetch URL: {e}")
}

#[async_trait]
impl AgentTool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a http/https URL. For web pages, extracts readable text. \
         For document files (PDF, Word, TXT, Markdown, Excel, PPT), downloads and parses the file content. \
         Supported file types: .pdf, .docx, .txt, .md, .csv, .xls, .xlsx, .ppt, .pptx"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The HTTP/HTTPS URL to fetch (web page or document file link)"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if url.is_empty() {
            return ToolRunResult::error("Error: 'url' parameter is required");
        }

        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => {
                return ToolRunResult::error(
                    "Error: Invalid URL (must start with http:// or https://)",
                );
            }
        };

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return ToolRunResult::error(
                "Error: Invalid URL (must start with http:// or https://)",
            );
        }

        let result = if is_document_url(url) {
            self.fetch_document(url).await
        } else {
            self.fetch_webpage(url).await
        };

        match result {
            Ok(text) => ToolRunResult::success_text(text),
            Err(msg) => ToolRunResult::error(msg),
        }
    }
}
