//! Rerank providers for hybrid memory search (`query + candidates → reordered scores`).

use std::collections::HashSet;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::config::ModelsConfig;
use crate::utils::{build_reqwest_client, HttpProxySettings};

use super::storage::SearchResult;

const RERANK_HTTP_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone)]
struct RerankVendorMeta {
    default_base_url: &'static str,
    default_model: &'static str,
}

fn rerank_vendor_meta(provider: &str) -> Option<RerankVendorMeta> {
    Some(match provider {
        "jina" => RerankVendorMeta {
            default_base_url: "https://api.jina.ai/v1",
            default_model: "jina-reranker-v2-base-multilingual",
        },
        "cohere" => RerankVendorMeta {
            default_base_url: "https://api.cohere.ai/v1",
            default_model: "rerank-multilingual-v3.0",
        },
        "siliconflow" => RerankVendorMeta {
            default_base_url: "https://api.siliconflow.cn/v1",
            default_model: "BAAI/bge-reranker-v2-m3",
        },
        "dashscope" => RerankVendorMeta {
            default_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            default_model: "gte-rerank-v2",
        },
        "openai" | "linkai" => RerankVendorMeta {
            default_base_url: "https://api.openai.com/v1",
            default_model: "rerank-english-v3.0",
        },
        _ => return None,
    })
}

/// Cross-encoder / API rerank：输入 query 与候选 chunk，输出重排后的列表。
#[async_trait]
pub trait RerankProvider: Send + Sync {
    async fn rerank(
        &self,
        query: &str,
        candidates: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, String>;
    fn provider_name(&self) -> &str;
    fn model_name(&self) -> &str;
}

/// 本地轻量 rerank：按 query 与 snippet 的词重叠打分（无需 API）。
pub struct LexicalRerankProvider;

impl LexicalRerankProvider {
    fn overlap_score(query: &str, document: &str) -> f64 {
        let q: HashSet<_> = tokenize(query).into_iter().collect();
        if q.is_empty() {
            return 0.0;
        }
        let d: HashSet<_> = tokenize(document).into_iter().collect();
        let inter = q.intersection(&d).count() as f64;
        inter / q.len() as f64
    }
}

#[async_trait]
impl RerankProvider for LexicalRerankProvider {
    async fn rerank(
        &self,
        query: &str,
        candidates: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, String> {
        let mut out: Vec<SearchResult> = candidates
            .into_iter()
            .map(|mut r| {
                let doc = format!("{} {}", r.path, r.snippet);
                r.score = Self::overlap_score(query, &doc);
                r
            })
            .collect();
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(out)
    }

    fn provider_name(&self) -> &str {
        "local"
    }

    fn model_name(&self) -> &str {
        "lexical-overlap"
    }
}

/// OpenAI-compatible `/v1/rerank` HTTP provider（Jina / Cohere / SiliconFlow 等）。
pub struct HttpRerankProvider {
    client: reqwest::Client,
    api_base: String,
    api_key: String,
    model: String,
    provider_name: String,
}

impl HttpRerankProvider {
    pub fn new(
        provider_name: impl Into<String>,
        model: String,
        api_key: String,
        api_base: String,
        proxy: &HttpProxySettings,
    ) -> Result<Self, String> {
        let api_key = api_key.trim().to_string();
        if api_key.is_empty() || api_key == "YOUR API KEY" || api_key == "YOUR_API_KEY" {
            return Err("Rerank API key is not configured".into());
        }
        let client = build_reqwest_client(
            proxy,
            std::time::Duration::from_secs(RERANK_HTTP_TIMEOUT_SECS),
            None,
        );
        Ok(Self {
            client,
            api_base: api_base.trim_end_matches('/').to_string(),
            api_key,
            model,
            provider_name: provider_name.into(),
        })
    }

    async fn call_api(
        &self,
        query: &str,
        documents: &[String],
    ) -> Result<Vec<(usize, f64)>, String> {
        let url = format!("{}/rerank", self.api_base);
        let body = json!({
            "model": self.model,
            "query": query,
            "documents": documents,
            "top_n": documents.len(),
        });
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("rerank request: {e}"))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| format!("rerank body: {e}"))?;
        if !status.is_success() {
            return Err(format!("rerank HTTP {status}: {text}"));
        }
        parse_rerank_response(&text)
    }
}

#[async_trait]
impl RerankProvider for HttpRerankProvider {
    async fn rerank(
        &self,
        query: &str,
        candidates: Vec<SearchResult>,
    ) -> Result<Vec<SearchResult>, String> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }
        let documents: Vec<String> = candidates
            .iter()
            .map(|c| format!("{} {}", c.path, c.snippet))
            .collect();
        let ranked = self.call_api(query, &documents).await?;
        let mut out = Vec::with_capacity(ranked.len());
        for (idx, score) in ranked {
            if let Some(mut item) = candidates.get(idx).cloned() {
                item.score = score;
                out.push(item);
            }
        }
        Ok(out)
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

fn parse_rerank_response(text: &str) -> Result<Vec<(usize, f64)>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("rerank json: {e}"))?;
    let items = v
        .get("results")
        .or_else(|| v.get("data"))
        .and_then(|a| a.as_array())
        .ok_or_else(|| "rerank response missing results/data".to_string())?;

    let mut ranked = Vec::new();
    for item in items {
        let idx = item
            .get("index")
            .and_then(|i| i.as_u64())
            .ok_or_else(|| "rerank item missing index".to_string())? as usize;
        let score = item
            .get("relevance_score")
            .or_else(|| item.get("score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(0.0);
        ranked.push((idx, score));
    }
    Ok(ranked)
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// 从 `config.json` 构建 rerank provider；未配置时返回 `None`（保持现网行为）。
pub fn create_rerank_provider(
    config: &ModelsConfig,
) -> Result<Option<std::sync::Arc<dyn RerankProvider>>, String> {
    let explicit = config
        .rerank_provider
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());

    let Some(provider) = explicit else {
        return Ok(None);
    };

    if provider == "local" || provider == "lexical" {
        return Ok(Some(std::sync::Arc::new(LexicalRerankProvider)));
    }

    let proxy = HttpProxySettings::from_config(config);
    let meta = rerank_vendor_meta(&provider)
        .ok_or_else(|| format!("unsupported rerank provider: {provider}"))?;
    let (api_key, api_base) = resolve_rerank_credentials(config, &provider, meta.default_base_url)?;
    let Some(api_key) = api_key else {
        tracing::warn!(
            "[Rerank] provider={provider} configured but API key missing; skipping rerank"
        );
        return Ok(None);
    };
    let model = config
        .rerank_model
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(meta.default_model)
        .to_string();
    let p = HttpRerankProvider::new(provider.clone(), model, api_key, api_base, &proxy)?;
    Ok(Some(std::sync::Arc::new(p)))
}

fn resolve_rerank_credentials(
    config: &ModelsConfig,
    provider: &str,
    default_base: &str,
) -> Result<(Option<String>, String), String> {
    let key = match provider {
        "openai" => config.open_ai_api_key.clone(),
        "linkai" => config.linkai_api_key.clone(),
        "jina" => config.custom_api_key.clone(),
        "cohere" => config.custom_api_key.clone(),
        "dashscope" => config.dashscope_api_key.clone(),
        "siliconflow" => config
            .custom_api_key
            .clone()
            .or_else(|| config.open_ai_api_key.clone()),
        "zhipu" => config.zhipu_ai_api_key.clone(),
        "doubao" => config.ark_api_key.clone(),
        _ => None,
    };
    let key = key.filter(|k| valid_key(k.as_str()));
    let base = match provider {
        "openai" => config
            .open_ai_api_base
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_base.to_string()),
        "linkai" => "https://api.link-ai.tech/v1".into(),
        "jina" => default_base.to_string(),
        "cohere" => default_base.to_string(),
        "siliconflow" => default_base.to_string(),
        "dashscope" => default_base.to_string(),
        "zhipu" => config
            .zhipu_ai_api_base
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_base.to_string()),
        "doubao" => config
            .ark_base_url
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| default_base.to_string()),
        _ => default_base.to_string(),
    };
    Ok((key, base))
}

fn valid_key(key: &str) -> bool {
    let k = key.trim();
    !k.is_empty() && k != "YOUR API KEY" && k != "YOUR_API_KEY"
}
