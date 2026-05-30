//! `agent/tools/mcp/mcp_client.py` — JSON-RPC 2.0 over stdio / SSE / streamable HTTP.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
use reqwest::Client;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, warn};

use super::config::{normalize_transport, McpServerConfig};

const PROTOCOL_VERSION: &str = "2024-11-05";
const READ_TIMEOUT: Duration = Duration::from_secs(30);
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum McpClientError {
    #[error("{0}")]
    Msg(String),
}

pub type McpResult<T> = Result<T, McpClientError>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

enum Transport {
    Stdio {
        child: Child,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    },
    Sse {
        post_url: String,
        http: Client,
    },
    StreamableHttp {
        url: String,
        headers: HashMap<String, String>,
        session_id: Option<String>,
        http: Client,
    },
}

struct Inner {
    transport: Transport,
    initialized: bool,
    next_id: AtomicU64,
}

/// Single MCP server client.
pub struct McpClient {
    pub name: String,
    inner: Arc<Mutex<Inner>>,
}

impl McpClient {
    /// Connect and perform MCP handshake.
    pub async fn initialize(config: &McpServerConfig) -> McpResult<Self> {
        let name = config.name.clone();
        let transport_key = normalize_transport(&config.transport);

        let transport = match transport_key.as_str() {
            "stdio" => Self::init_stdio(config).await?,
            "sse" => Self::init_sse(config).await?,
            "streamable-http" => Self::init_streamable_http(config).await?,
            other => {
                return Err(McpClientError::Msg(format!(
                    "Unknown transport type: {other}"
                )));
            }
        };

        let client = Self {
            name: name.clone(),
            inner: Arc::new(Mutex::new(Inner {
                transport,
                initialized: false,
                next_id: AtomicU64::new(1),
            })),
        };

        if !client.handshake().await? {
            return Err(McpClientError::Msg(format!(
                "Handshake failed for MCP server '{name}'"
            )));
        }

        Ok(client)
    }

    pub async fn list_tools(&self) -> Vec<McpToolSchema> {
        match self.send_request("tools/list", json!({})).await {
            Ok(resp) => resp
                .get("result")
                .and_then(|r| r.get("tools"))
                .and_then(|t| t.as_array())
                .map(|tools| {
                    tools
                        .iter()
                        .filter_map(|t| {
                            Some(McpToolSchema {
                                name: t.get("name")?.as_str()?.to_string(),
                                description: t
                                    .get("description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                input_schema: t
                                    .get("inputSchema")
                                    .cloned()
                                    .unwrap_or(json!({"type": "object"})),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Err(e) => {
                warn!(server = %self.name, %e, "list_tools failed");
                Vec::new()
            }
        }
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> String {
        let params = json!({ "name": tool_name, "arguments": arguments });
        match self.send_request("tools/call", params).await {
            Ok(resp) => {
                let content = resp
                    .get("result")
                    .and_then(|r| r.get("content"))
                    .and_then(|c| c.as_array());
                let Some(items) = content else {
                    return resp.to_string();
                };
                items
                    .iter()
                    .filter(|item| item.get("type").and_then(|t| t.as_str()) == Some("text"))
                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            Err(e) => format!("Error: {e}"),
        }
    }

    pub async fn shutdown(&self) {
        let mut guard = self.inner.lock().await;
        match &mut guard.transport {
            Transport::Stdio { child, .. } => {
                let _ = child.start_kill();
                let _ = timeout(Duration::from_secs(5), child.wait()).await;
                debug!(server = %self.name, "stdio process terminated");
            }
            Transport::StreamableHttp {
                url,
                headers,
                session_id,
                http,
            } => {
                if let Some(sid) = session_id.take() {
                    let delete_url = url.clone();
                    let mut hdrs = HeaderMap::new();
                    let _ = hdrs.insert(
                        HeaderName::from_static("mcp-session-id"),
                        HeaderValue::from_str(&sid).unwrap_or(HeaderValue::from_static("")),
                    );
                    for (k, v) in headers.iter() {
                        if let (Ok(name), Ok(val)) = (
                            HeaderName::from_bytes(k.as_bytes()),
                            HeaderValue::from_str(v),
                        ) {
                            let _ = hdrs.insert(name, val);
                        }
                    }
                    let _ = http.delete(delete_url).headers(hdrs).send().await;
                }
            }
            Transport::Sse { .. } => {}
        }
        guard.initialized = false;
    }

    async fn init_stdio(config: &McpServerConfig) -> McpResult<Transport> {
        let command = config.command.as_ref().ok_or_else(|| {
            McpClientError::Msg(format!(
                "[MCP:{}] stdio config missing 'command'",
                config.name
            ))
        })?;

        let mut cmd = Command::new(command);
        cmd.args(&config.args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        if !config.env.is_empty() {
            cmd.envs(&config.env);
        }

        let mut child = cmd.spawn().map_err(|e| {
            McpClientError::Msg(format!("[MCP:{}] failed to spawn: {e}", config.name))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpClientError::Msg(format!("[MCP:{}] no stdin", config.name)))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpClientError::Msg(format!("[MCP:{}] no stdout", config.name)))?;

        let stderr = child.stderr.take();
        if let Some(stderr) = stderr {
            let server = config.name.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        debug!(server = %server, %trimmed, "mcp stderr");
                    }
                    line.clear();
                }
            });
        }

        debug!(server = %config.name, "stdio process started");

        Ok(Transport::Stdio {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    async fn init_sse(config: &McpServerConfig) -> McpResult<Transport> {
        let url = config.url.as_ref().ok_or_else(|| {
            McpClientError::Msg(format!("[MCP:{}] SSE config missing 'url'", config.name))
        })?;

        let http = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| McpClientError::Msg(e.to_string()))?;

        let post_url = Self::sse_discover_endpoint(&http, url, &config.name).await?;

        Ok(Transport::Sse { post_url, http })
    }

    async fn sse_discover_endpoint(
        http: &Client,
        sse_url: &str,
        server_name: &str,
    ) -> McpResult<String> {
        let mut resp = http
            .get(sse_url)
            .header(ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|e| McpClientError::Msg(e.to_string()))?;

        let mut buf = String::new();
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| McpClientError::Msg(e.to_string()))?
        {
            buf.push_str(&String::from_utf8_lossy(&chunk));
            for line in buf.lines() {
                let line = line.trim_end_matches(['\r', '\n']);
                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if data.starts_with('{') {
                        let parsed: Value = serde_json::from_str(data)
                            .map_err(|e| McpClientError::Msg(e.to_string()))?;
                        if let Some(uri) = parsed
                            .get("uri")
                            .or_else(|| parsed.get("url"))
                            .or_else(|| parsed.get("endpoint"))
                            .and_then(|v| v.as_str())
                        {
                            return Ok(uri.to_string());
                        }
                    } else if data.starts_with("http") {
                        return Ok(data.to_string());
                    } else {
                        return Ok(reqwest::Url::parse(sse_url)
                            .ok()
                            .and_then(|base| base.join(data).ok())
                            .map(|u| u.to_string())
                            .unwrap_or_else(|| data.to_string()));
                    }
                }
            }
        }

        Err(McpClientError::Msg(format!(
            "[MCP:{server_name}] No endpoint event received from SSE stream"
        )))
    }

    async fn init_streamable_http(config: &McpServerConfig) -> McpResult<Transport> {
        let url = config.url.as_ref().ok_or_else(|| {
            McpClientError::Msg(format!(
                "[MCP:{}] streamable-http config missing 'url'",
                config.name
            ))
        })?;

        let http = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .map_err(|e| McpClientError::Msg(e.to_string()))?;

        Ok(Transport::StreamableHttp {
            url: url.clone(),
            headers: config.headers.clone(),
            session_id: None,
            http,
        })
    }

    async fn handshake(&self) -> McpResult<bool> {
        {
            let mut guard = self.inner.lock().await;
            guard.initialized = true;
        }

        let init_params = json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "CowAgent", "version": "1.0" }
        });

        let resp = match self.send_request("initialize", init_params).await {
            Ok(r) => r,
            Err(e) => {
                let mut guard = self.inner.lock().await;
                guard.initialized = false;
                warn!(server = %self.name, %e, "Handshake initialize failed");
                return Ok(false);
            }
        };

        if resp.get("error").is_some() {
            let mut guard = self.inner.lock().await;
            guard.initialized = false;
            warn!(server = %self.name, ?resp, "Handshake error");
            return Ok(false);
        }

        self.send_notification("notifications/initialized", json!({}))
            .await;
        debug!(server = %self.name, "Handshake complete");
        Ok(true)
    }

    async fn send_request(&self, method: &str, params: Value) -> McpResult<Value> {
        let message = {
            let guard = self.inner.lock().await;
            if !guard.initialized && method != "initialize" {
                return Err(McpClientError::Msg(format!(
                    "[MCP:{}] Client not initialized",
                    self.name
                )));
            }
            Self::build_request(&guard.next_id, method, params)
        };

        let mut guard = self.inner.lock().await;
        match &mut guard.transport {
            Transport::Stdio { stdin, stdout, .. } => {
                Self::stdio_send(stdin, stdout, &self.name, message).await
            }
            Transport::Sse { post_url, http } => {
                Self::http_post_json(http, post_url, &HashMap::new(), None, message, true).await
            }
            Transport::StreamableHttp {
                url,
                headers,
                session_id,
                http,
            } => {
                let sid = session_id.clone();
                let mut resp =
                    Self::http_post_json(http, url, headers, sid.as_deref(), message, true).await?;
                if session_id.is_none() {
                    if let Some(new_sid) = resp
                        .get("_session_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        *session_id = Some(new_sid);
                    }
                }
                if let Value::Object(ref mut map) = resp {
                    map.remove("_session_id");
                }
                Ok(resp)
            }
        }
    }

    async fn send_notification(&self, method: &str, params: Value) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        let guard = self.inner.lock().await;
        let _ = match &guard.transport {
            Transport::Stdio { stdin, .. } => {
                drop(guard);
                let mut guard = self.inner.lock().await;
                if let Transport::Stdio { stdin, .. } = &mut guard.transport {
                    let raw = format!("{notification}\n");
                    let _ = stdin.write_all(raw.as_bytes()).await;
                    let _ = stdin.flush().await;
                }
                Ok::<(), McpClientError>(())
            }
            Transport::Sse { post_url, http } => {
                Self::http_post_json(http, post_url, &HashMap::new(), None, notification, false)
                    .await
                    .map(|_| ())
            }
            Transport::StreamableHttp {
                url,
                headers,
                session_id,
                http,
            } => Self::http_post_json(
                http,
                url,
                headers,
                session_id.as_deref(),
                notification,
                false,
            )
            .await
            .map(|_| ()),
        };
    }

    fn build_request(next_id: &AtomicU64, method: &str, params: Value) -> Value {
        let id = next_id.fetch_add(1, Ordering::SeqCst);
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        })
    }

    async fn stdio_send(
        stdin: &mut ChildStdin,
        stdout: &mut BufReader<ChildStdout>,
        server_name: &str,
        message: Value,
    ) -> McpResult<Value> {
        let raw = format!("{message}\n");
        stdin
            .write_all(raw.as_bytes())
            .await
            .map_err(|e| McpClientError::Msg(e.to_string()))?;
        stdin
            .flush()
            .await
            .map_err(|e| McpClientError::Msg(e.to_string()))?;

        let expected_id = message.get("id").cloned();

        loop {
            let mut line = String::new();
            let read = timeout(READ_TIMEOUT, stdout.read_line(&mut line))
                .await
                .map_err(|_| {
                    McpClientError::Msg(format!(
                        "[MCP:{server_name}] stdio read timed out after {}s",
                        READ_TIMEOUT.as_secs()
                    ))
                })?
                .map_err(|e| McpClientError::Msg(e.to_string()))?;

            if read == 0 {
                return Err(McpClientError::Msg(format!(
                    "[MCP:{server_name}] stdio process closed unexpectedly"
                )));
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let data: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if data.get("id").is_none() {
                debug!(
                    server = %server_name,
                    method = data.get("method").and_then(|m| m.as_str()).unwrap_or("?"),
                    "notification skipped"
                );
                continue;
            }

            if expected_id.is_none() || data.get("id") == expected_id.as_ref() {
                return Ok(data);
            }
        }
    }

    async fn http_post_json(
        http: &Client,
        url: &str,
        extra_headers: &HashMap<String, String>,
        session_id: Option<&str>,
        message: Value,
        expect_response: bool,
    ) -> McpResult<Value> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/event-stream"),
        );
        if let Some(sid) = session_id {
            let _ = headers.insert(
                HeaderName::from_static("mcp-session-id"),
                HeaderValue::from_str(sid).unwrap_or(HeaderValue::from_static("")),
            );
        }
        for (k, v) in extra_headers {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                let _ = headers.insert(name, val);
            }
        }

        let resp = http
            .post(url)
            .headers(headers.clone())
            .json(&message)
            .send()
            .await
            .map_err(|e| McpClientError::Msg(e.to_string()))?;

        let status = resp.status();
        let resp_headers = resp.headers().clone();
        let session_from_server = resp_headers
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if !expect_response || status.as_u16() == 202 {
            let _ = resp.bytes().await;
            return Ok(session_from_server
                .map(|s| json!({ "_session_id": s }))
                .unwrap_or(json!({})));
        }

        let content_type = resp_headers
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_ascii_lowercase();

        let expected_id = message.get("id").cloned();

        if content_type.contains("text/event-stream") {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| McpClientError::Msg(e.to_string()))?;
            let text = String::from_utf8_lossy(&bytes);
            Self::parse_sse_json_response(&text, expected_id)
        } else {
            let text = resp
                .text()
                .await
                .map_err(|e| McpClientError::Msg(e.to_string()))?;
            if text.is_empty() {
                return Ok(session_from_server
                    .map(|s| json!({ "_session_id": s }))
                    .unwrap_or(json!({})));
            }
            let mut val: Value =
                serde_json::from_str(&text).map_err(|e| McpClientError::Msg(e.to_string()))?;
            if let Some(sid) = session_from_server {
                if let Value::Object(ref mut map) = val {
                    map.insert("_session_id".into(), Value::String(sid));
                }
            }
            Ok(val)
        }
    }

    fn parse_sse_json_response(text: &str, expected_id: Option<Value>) -> McpResult<Value> {
        let mut data_buf: Vec<String> = Vec::new();
        for line in text.lines() {
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                if data_buf.is_empty() {
                    continue;
                }
                let payload = data_buf.join("\n");
                data_buf.clear();
                let msg: Value = match serde_json::from_str(&payload) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if msg.get("id").is_none() {
                    continue;
                }
                if expected_id.is_none() || msg.get("id") == expected_id.as_ref() {
                    return Ok(msg);
                }
                continue;
            }
            if line.starts_with(':') {
                continue;
            }
            if let Some(data) = line.strip_prefix("data:") {
                data_buf.push(data.trim().to_string());
            }
        }
        Err(McpClientError::Msg(
            "streamable-http SSE stream closed before response".into(),
        ))
    }
}
