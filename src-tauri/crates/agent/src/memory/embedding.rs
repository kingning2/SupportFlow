//! OpenAI-compatible embedding providers (`agent/memory/embedding/provider.py`).

use async_trait::async_trait;
use models::{build_reqwest_client, HttpProxySettings, ModelsConfig};
use serde_json::json;

const EMBEDDING_HTTP_TIMEOUT_SECS: u64 = 90;

#[derive(Debug, Clone)]
struct VendorMeta {
    default_base_url: &'static str,
    default_model: &'static str,
    default_dimensions: u32,
    max_batch_size: usize,
}

fn vendor_meta(provider: &str) -> Option<VendorMeta> {
    Some(match provider {
        "openai" => VendorMeta {
            default_base_url: "https://api.openai.com/v1",
            default_model: "text-embedding-3-small",
            default_dimensions: 1536,
            max_batch_size: 64,
        },
        "linkai" => VendorMeta {
            default_base_url: "https://api.link-ai.tech/v1",
            default_model: "text-embedding-3-small",
            default_dimensions: 1536,
            max_batch_size: 64,
        },
        "dashscope" => VendorMeta {
            default_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1",
            default_model: "text-embedding-v4",
            default_dimensions: 1024,
            max_batch_size: 10,
        },
        "zhipu" => VendorMeta {
            default_base_url: "https://open.bigmodel.cn/api/paas/v4",
            default_model: "embedding-3",
            default_dimensions: 1024,
            max_batch_size: 64,
        },
        _ => return None,
    })
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>, String>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String>;
    fn dimensions(&self) -> usize;
    fn provider_name(&self) -> &str;
    fn model_name(&self) -> &str;
}

pub struct OpenAiEmbeddingProvider {
    client: reqwest::Client,
    api_base: String,
    api_key: String,
    model: String,
    dimensions: u32,
    max_batch_size: usize,
    provider_name: String,
}

impl OpenAiEmbeddingProvider {
    pub fn new(
        provider_name: impl Into<String>,
        model: String,
        api_key: String,
        api_base: String,
        dimensions: u32,
        max_batch_size: usize,
        proxy: &HttpProxySettings,
    ) -> Result<Self, String> {
        let api_key = api_key.trim().to_string();
        if api_key.is_empty() || api_key == "YOUR API KEY" || api_key == "YOUR_API_KEY" {
            return Err("Embedding API key is not configured".into());
        }
        let client = build_reqwest_client(
            proxy,
            std::time::Duration::from_secs(EMBEDDING_HTTP_TIMEOUT_SECS),
            None,
        );
        Ok(Self {
            client,
            api_base: api_base.trim_end_matches('/').to_string(),
            api_key,
            model,
            dimensions,
            max_batch_size: max_batch_size.max(1),
            provider_name: provider_name.into(),
        })
    }

    async fn call_api(&self, input: serde_json::Value) -> Result<Vec<Vec<f32>>, String> {
        let url = format!("{}/embeddings", self.api_base);
        let body = json!({
            "input": input,
            "model": self.model,
            "dimensions": self.dimensions,
        });
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("embedding request failed: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("embedding API {status}: {text}"));
        }
        let payload: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let data = payload
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "invalid embedding response: missing data".to_string())?;
        let mut out = Vec::with_capacity(data.len());
        for item in data {
            let emb = item
                .get("embedding")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "invalid embedding item".to_string())?;
            out.push(
                emb.iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect(),
            );
        }
        Ok(out)
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    async fn embed_query(&self, text: &str) -> Result<Vec<f32>, String> {
        let batch = self.call_api(json!(text)).await?;
        batch
            .into_iter()
            .next()
            .ok_or_else(|| "empty embedding response".to_string())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let mut out = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(self.max_batch_size) {
            let part = self.call_api(json!(chunk)).await?;
            out.extend(part);
        }
        Ok(out)
    }

    fn dimensions(&self) -> usize {
        self.dimensions as usize
    }

    fn provider_name(&self) -> &str {
        &self.provider_name
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Build embedding provider from SupportFlow config (legacy OpenAI → LinkAI, or explicit vendor).
pub fn create_embedding_provider(
    config: &ModelsConfig,
) -> Result<Option<std::sync::Arc<dyn EmbeddingProvider>>, String> {
    let proxy = HttpProxySettings::from_models(config);
    let explicit = config
        .embedding_provider
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_lowercase());

    if let Some(provider) = explicit {
        return create_explicit_provider(config, &provider, &proxy);
    }

    create_legacy_provider(config, &proxy)
}

fn create_legacy_provider(
    config: &ModelsConfig,
    proxy: &HttpProxySettings,
) -> Result<Option<std::sync::Arc<dyn EmbeddingProvider>>, String> {
    if let Some(key) = config.open_ai_api_key.as_deref().filter(|k| valid_key(k)) {
        let base = config
            .open_ai_api_base
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("https://api.openai.com/v1");
        if let Ok(p) = OpenAiEmbeddingProvider::new(
            "openai",
            "text-embedding-3-small".into(),
            key.to_string(),
            base.to_string(),
            1536,
            64,
            proxy,
        ) {
            return Ok(Some(std::sync::Arc::new(p)));
        }
    }
    if let Some(key) = config.linkai_api_key.as_deref().filter(|k| valid_key(k)) {
        if let Ok(p) = OpenAiEmbeddingProvider::new(
            "linkai",
            "text-embedding-3-small".into(),
            key.to_string(),
            "https://api.link-ai.tech/v1".into(),
            1536,
            64,
            proxy,
        ) {
            return Ok(Some(std::sync::Arc::new(p)));
        }
    }
    Ok(None)
}

fn create_explicit_provider(
    config: &ModelsConfig,
    provider: &str,
    proxy: &HttpProxySettings,
) -> Result<Option<std::sync::Arc<dyn EmbeddingProvider>>, String> {
    let meta = match vendor_meta(provider) {
        Some(m) => m,
        None => return Ok(None),
    };
    let (api_key, api_base) =
        resolve_embedding_credentials(config, provider, meta.default_base_url)?;
    let Some(api_key) = api_key else {
        return Ok(None);
    };
    let model = config
        .embedding_model
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(meta.default_model)
        .to_string();
    let dim = config
        .embedding_dimensions
        .filter(|d| *d > 0)
        .unwrap_or(meta.default_dimensions);
    let p = OpenAiEmbeddingProvider::new(
        provider,
        model,
        api_key,
        api_base,
        dim,
        meta.max_batch_size,
        proxy,
    )?;
    Ok(Some(std::sync::Arc::new(p)))
}

fn resolve_embedding_credentials(
    config: &ModelsConfig,
    provider: &str,
    default_base: &str,
) -> Result<(Option<String>, String), String> {
    let key = match provider {
        "openai" => config.open_ai_api_key.clone(),
        "linkai" => config.linkai_api_key.clone(),
        "dashscope" => config.dashscope_api_key.clone(),
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
        "dashscope" => default_base.to_string(),
        "zhipu" => default_base.to_string(),
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
