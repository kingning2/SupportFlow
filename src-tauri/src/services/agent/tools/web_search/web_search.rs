//! `agent/tools/web_search/web_search.py`

use std::sync::Arc;
use std::time::Duration;

use crate::config::ModelsConfig;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Local};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::info;

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::web_search::config::{default_timeout_secs, WebSearchSettings};
use crate::utils::http_proxy::{build_reqwest_client, HttpProxySettings};

pub struct WebSearchTool {
    settings: WebSearchSettings,
    client: Client,
}

impl WebSearchTool {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let settings = WebSearchSettings::from_models(&config);
        let proxy = HttpProxySettings::from_config(&config);
        Self {
            settings,
            client: build_reqwest_client(&proxy, Duration::from_secs(default_timeout_secs()), None),
        }
    }

    pub fn from_settings(settings: WebSearchSettings) -> Self {
        Self {
            settings,
            client: build_reqwest_client(
                &HttpProxySettings::default(),
                Duration::from_secs(default_timeout_secs()),
                None,
            ),
        }
    }

    pub fn is_available(config: &ModelsConfig) -> bool {
        WebSearchSettings::from_models(config).is_available()
    }

    fn build_input_schema(&self) -> Value {
        let mut properties = json!({
            "query": { "type": "string", "description": "Search query string" },
            "count": { "type": "integer", "description": "Number of results to return (1-50, default: 10)" },
            "freshness": {
                "type": "string",
                "description": "Time range: 'noLimit' (default), 'oneDay', 'oneWeek', 'oneMonth', 'oneYear', or date range"
            },
            "summary": { "type": "boolean", "description": "Include text summary per result (Bocha only, default: false)" }
        });

        if self.settings.strategy == "auto" {
            let available = self.settings.configured_providers();
            if available.len() >= 2 {
                properties["provider"] = json!({
                    "type": "string",
                    "enum": available,
                    "description": "Optional search backend when multiple providers are configured"
                });
            }
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": ["query"]
        })
    }

    async fn search_bocha(
        &self,
        query: &str,
        count: i32,
        freshness: &str,
        summary: bool,
    ) -> Result<Value, String> {
        let api_key = self.settings.bocha_api_key.clone();
        let url = "https://api.bochaai.com/v1/web-search";
        let payload = json!({
            "query": query,
            "count": count,
            "freshness": freshness,
            "summary": summary
        });

        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if status == 401 {
            return Err("Error: Invalid bocha API key.".into());
        }
        if status == 403 {
            return Err(
                "Error: bocha API — insufficient balance. Top up at https://open.bochaai.com"
                    .into(),
            );
        }
        if status == 429 {
            return Err("Error: bocha API rate limit reached.".into());
        }
        if status != 200 {
            return Err(format!("Error: bocha API returned HTTP {status}"));
        }

        let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if let Some(code) = data.get("code").and_then(|c| c.as_i64()) {
            if code != 200 {
                let msg = data
                    .get("msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");
                return Err(format!("Error: bocha API error (code={code}): {msg}"));
            }
        }

        let pages = data
            .pointer("/data/webPages/value")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for p in pages {
            let mut item = json!({
                "title": p.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                "url": p.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                "snippet": p.get("snippet").and_then(|v| v.as_str()).unwrap_or(""),
                "siteName": p.get("siteName").and_then(|v| v.as_str()).unwrap_or(""),
                "datePublished": p.get("datePublished")
                    .or_else(|| p.get("dateLastCrawled"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
            });
            if let Some(s) = p.get("summary") {
                item["summary"] = s.clone();
            }
            results.push(item);
        }

        let total = data
            .pointer("/data/webPages/totalEstimatedMatches")
            .and_then(|v| v.as_u64())
            .unwrap_or(results.len() as u64);

        Ok(json!({
            "query": query,
            "backend": "bocha",
            "total": total,
            "count": results.len(),
            "results": results
        }))
    }

    async fn search_zhipu(
        &self,
        query: &str,
        count: i32,
        freshness: &str,
    ) -> Result<Value, String> {
        let api_key = self.settings.zhipu_ai_api_key.clone();
        let base = self.settings.zhipu_ai_api_base.trim_end_matches('/');
        let url = format!("{base}/web_search");

        let trimmed_query: String = query.chars().take(70).collect();
        let mut engine = self.settings.zhipu_search_engine.clone();
        if !matches!(
            engine.as_str(),
            "search_std" | "search_pro" | "search_pro_sogou" | "search_pro_quark"
        ) {
            engine = "search_pro".into();
        }

        let recency = if matches!(
            freshness,
            "oneDay" | "oneWeek" | "oneMonth" | "oneYear" | "noLimit"
        ) {
            freshness
        } else {
            "noLimit"
        };

        let mut payload = json!({
            "search_engine": engine,
            "search_query": trimmed_query,
            "search_intent": false,
            "count": count.clamp(1, 50),
            "search_recency_filter": recency,
        });

        let size = &self.settings.zhipu_content_size;
        if size == "medium" || size == "high" {
            payload["content_size"] = json!(size);
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if status == 401 {
            return Err("Error: Invalid Zhipu API key.".into());
        }
        if status != 200 {
            return Err(format!(
                "Error: Zhipu API returned HTTP {status}: {}",
                &text[..text.len().min(200)]
            ));
        }

        let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if let Some(err) = data.get("error") {
            if err.is_object() {
                let code = err.get("code").map(|c| c.to_string()).unwrap_or_default();
                let message = err.get("message").and_then(|m| m.as_str()).unwrap_or("");
                return Err(format!("Error: Zhipu returned {code}: {message}"));
            }
        }

        let items = data
            .get("search_result")
            .or_else(|| data.pointer("/data/search_result"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let results: Vec<Value> = items
            .iter()
            .map(|it| {
                json!({
                    "title": it.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    "url": it.get("link").or_else(|| it.get("url")).and_then(|v| v.as_str()).unwrap_or(""),
                    "snippet": it.get("content").or_else(|| it.get("snippet")).and_then(|v| v.as_str()).unwrap_or(""),
                    "siteName": it.get("media").or_else(|| it.get("siteName")).and_then(|v| v.as_str()).unwrap_or(""),
                    "datePublished": it.get("publish_date").or_else(|| it.get("datePublished")).and_then(|v| v.as_str()).unwrap_or(""),
                })
            })
            .collect();

        Ok(json!({
            "query": query,
            "backend": "zhipu",
            "total": results.len(),
            "count": results.len(),
            "results": results
        }))
    }

    async fn search_qianfan(
        &self,
        query: &str,
        count: i32,
        freshness: &str,
    ) -> Result<Value, String> {
        let api_key = self.settings.qianfan_api_key.clone();
        let base = self.settings.qianfan_api_base.trim_end_matches('/');
        let url = format!("{base}/ai_search/web_search");

        let mut payload = json!({
            "messages": [{ "role": "user", "content": query }],
            "search_source": "baidu_search_v2",
            "resource_type_filter": [{ "type": "web", "top_k": count.clamp(1, 50) }],
        });

        if let Some(filter) = qianfan_freshness_filter(freshness) {
            payload["search_filter"] = filter;
        }

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .header("X-Appbuilder-From", "supportflow")
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if status == 401 {
            return Err("Error: Invalid Qianfan API key.".into());
        }
        if status != 200 {
            return Err(format!(
                "Error: Qianfan API returned HTTP {status}: {}",
                &text[..text.len().min(200)]
            ));
        }

        let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if let Some(code) = data.get("code") {
            if !code.is_null() && code.as_i64().unwrap_or(0) != 0 {
                let message = data.get("message").and_then(|m| m.as_str()).unwrap_or("");
                return Err(format!("Error: Qianfan returned {code}: {message}"));
            }
        }

        let refs = data
            .get("references")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let results: Vec<Value> = refs
            .iter()
            .map(|d| {
                let content = d.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let snippet: String = content.chars().take(200).collect();
                json!({
                    "title": d.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                    "url": d.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                    "snippet": snippet,
                    "siteName": d.get("web_anchor").or_else(|| d.get("website")).and_then(|v| v.as_str()).unwrap_or(""),
                    "datePublished": d.get("date").and_then(|v| v.as_str()).unwrap_or(""),
                })
            })
            .collect();

        Ok(json!({
            "query": query,
            "backend": "qianfan",
            "total": results.len(),
            "count": results.len(),
            "results": results
        }))
    }

    async fn search_linkai(
        &self,
        query: &str,
        count: i32,
        freshness: &str,
    ) -> Result<Value, String> {
        let api_key = self.settings.linkai_api_key.clone();
        let base = self.settings.linkai_api_base.trim_end_matches('/');
        let url = format!("{base}/v1/plugin/execute");

        let payload = json!({
            "code": "web-search",
            "args": { "query": query, "count": count, "freshness": freshness }
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if status == 401 {
            return Err("Error: Invalid LinkAI API key.".into());
        }
        if status != 200 {
            return Err(format!("Error: LinkAI API returned HTTP {status}"));
        }

        let data: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(format!("Error: LinkAI search failed: {msg}"));
        }

        let raw = data.get("data").cloned().unwrap_or(Value::Null);
        let parsed = match raw {
            Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
            other => other,
        };

        if let Some(pages) = parsed.pointer("/webPages/value").and_then(|v| v.as_array()) {
            let results: Vec<Value> = pages
                .iter()
                .map(|p| {
                    let mut item = json!({
                        "title": p.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "url": p.get("url").and_then(|v| v.as_str()).unwrap_or(""),
                        "snippet": p.get("snippet").and_then(|v| v.as_str()).unwrap_or(""),
                        "siteName": p.get("siteName").and_then(|v| v.as_str()).unwrap_or(""),
                        "datePublished": p.get("datePublished").or_else(|| p.get("dateLastCrawled")).and_then(|v| v.as_str()).unwrap_or(""),
                    });
                    if let Some(s) = p.get("summary") {
                        item["summary"] = s.clone();
                    }
                    item
                })
                .collect();
            let total = parsed
                .pointer("/webPages/totalEstimatedMatches")
                .and_then(|v| v.as_u64())
                .unwrap_or(results.len() as u64);
            return Ok(json!({
                "query": query,
                "backend": "linkai",
                "total": total,
                "count": results.len(),
                "results": results
            }));
        }

        Ok(json!({
            "query": query,
            "backend": "linkai",
            "total": 1,
            "count": 1,
            "results": [{ "content": parsed.to_string() }]
        }))
    }
}

fn qianfan_freshness_filter(freshness: &str) -> Option<Value> {
    if freshness.is_empty() || freshness == "noLimit" {
        return None;
    }
    let delta_days = match freshness {
        "oneDay" => 1,
        "oneWeek" => 7,
        "oneMonth" => 30,
        "oneYear" => 365,
        _ => return None,
    };
    let now = Local::now();
    let end_date = (now + ChronoDuration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let start_date = (now - ChronoDuration::days(delta_days))
        .format("%Y-%m-%d")
        .to_string();
    Some(json!({
        "range": {
            "page_time": {
                "gte": start_date,
                "lt": end_date
            }
        }
    }))
}

fn map_reqwest_error(e: reqwest::Error) -> String {
    if e.is_timeout() {
        format!(
            "Error: Search request timed out after {}s",
            default_timeout_secs()
        )
    } else if e.is_connect() {
        "Error: Failed to connect to search API".into()
    } else {
        format!("Error: Search failed - {e}")
    }
}

#[async_trait]
impl AgentTool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for real-time information. Returns titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        self.build_input_schema()
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if query.is_empty() {
            return ToolRunResult::error("Error: 'query' parameter is required");
        }

        let mut count = params.get("count").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
        if !(1..=50).contains(&count) {
            count = 10;
        }
        let freshness = params
            .get("freshness")
            .and_then(|v| v.as_str())
            .unwrap_or("noLimit");
        let summary = params
            .get("summary")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let requested = params.get("provider").and_then(|v| v.as_str());

        let provider = match self.settings.resolve_provider(requested) {
            Some(p) => p,
            None => {
                return ToolRunResult::error(
                    "Error: No search provider configured. \
                     Configure one of BOCHA_API_KEY / zhipu_ai_api_key / qianfan_api_key / linkai_api_key.",
                );
            }
        };

        let available = self.settings.configured_providers();
        let q_preview = if query.len() > 60 {
            format!("{}...", &query[..57])
        } else {
            query.to_string()
        };
        info!(
            provider = %provider,
            available = ?available,
            query = %q_preview,
            count,
            freshness,
            "WebSearch"
        );

        let result = match provider.as_str() {
            "bocha" => self.search_bocha(query, count, freshness, summary).await,
            "zhipu" => self.search_zhipu(query, count, freshness).await,
            "qianfan" => self.search_qianfan(query, count, freshness).await,
            "linkai" => self.search_linkai(query, count, freshness).await,
            other => Err(format!("Error: Unknown provider '{other}'")),
        };

        match result {
            Ok(value) => ToolRunResult::success(value),
            Err(msg) => ToolRunResult::error(msg),
        }
    }
}
