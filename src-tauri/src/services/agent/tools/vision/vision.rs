//! `agent/tools/vision/vision.py`

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::config::ModelsConfig;
use crate::utils::{build_reqwest_client, HttpProxySettings};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::vision::config::{
    default_vision_model, resolve_providers, user_vision_model, VisionBackend, VisionProvider,
};

const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_TOKENS: u32 = 1000;

const SUPPORTED_EXT: &[(&str, &str)] = &[
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
];

pub struct VisionTool {
    config: Arc<ModelsConfig>,
    proxy: HttpProxySettings,
}

impl VisionTool {
    pub fn new(config: Arc<ModelsConfig>) -> Self {
        let proxy = HttpProxySettings::from_config(&config);
        Self { config, proxy }
    }

    pub fn is_available(config: &ModelsConfig) -> bool {
        !resolve_providers(Arc::new(config.clone())).is_empty()
    }

    fn local_image_block(path: &str) -> Result<Value, String> {
        let path = Path::new(path);
        if !path.is_file() {
            return Err(format!("Image file not found: {}", path.display()));
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let mime = SUPPORTED_EXT
            .iter()
            .find(|(e, _)| *e == ext)
            .map(|(_, m)| *m)
            .ok_or_else(|| {
                format!("Unsupported image format '.{ext}'. Supported: jpg, jpeg, png, gif, webp")
            })?;
        let bytes = std::fs::read(path).map_err(|e| format!("read image: {e}"))?;
        let b64 = STANDARD.encode(bytes);
        Ok(json!({
            "type": "image_url",
            "image_url": { "url": format!("data:{mime};base64,{b64}") }
        }))
    }

    async fn download_to_data_url(&self, url: &str) -> Result<Value, String> {
        let client =
            build_reqwest_client(&self.proxy, Duration::from_secs(DEFAULT_TIMEOUT_SECS), None);
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("download image: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!(
                "Failed to download image: HTTP {}",
                resp.status().as_u16()
            ));
        }
        let content_type = resp
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .split(';')
            .next()
            .unwrap_or("image/jpeg")
            .trim()
            .to_string();
        let mime = if content_type.starts_with("image/") {
            content_type
        } else {
            "image/jpeg".into()
        };
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        let b64 = STANDARD.encode(bytes);
        Ok(json!({
            "type": "image_url",
            "image_url": { "url": format!("data:{mime};base64,{b64}") }
        }))
    }

    async fn build_image_content(&self, image: &str) -> Result<Value, String> {
        if image.starts_with("http://") || image.starts_with("https://") {
            self.download_to_data_url(image).await
        } else {
            Self::local_image_block(image)
        }
    }

    async fn call_provider(
        &self,
        provider: &VisionProvider,
        model: &str,
        question: &str,
        image_block: &Value,
    ) -> Result<Value, String> {
        match &provider.backend {
            VisionBackend::OpenAi { api_key, api_base }
            | VisionBackend::LinkAi { api_key, api_base } => {
                self.call_raw_http(
                    api_key,
                    api_base,
                    model,
                    question,
                    image_block,
                    &provider.name,
                )
                .await
            }
        }
    }

    async fn call_raw_http(
        &self,
        api_key: &str,
        api_base: &str,
        model: &str,
        question: &str,
        image_block: &Value,
        provider_name: &str,
    ) -> Result<Value, String> {
        let client =
            build_reqwest_client(&self.proxy, Duration::from_secs(DEFAULT_TIMEOUT_SECS), None);
        let url = format!("{}/chat/completions", api_base.trim_end_matches('/'));
        let body = json!({
            "model": model,
            "max_tokens": MAX_TOKENS,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": question },
                    image_block,
                ],
            }],
        });
        let resp = client
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {api_key}"))
            .header(CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("vision request: {e}"))?;
        let status = resp.status();
        let data: Value = resp.json().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            let snippet = data.to_string();
            let snippet = if snippet.len() > 200 {
                &snippet[..200]
            } else {
                &snippet
            };
            return Err(format!("HTTP {}: {}", status.as_u16(), snippet));
        }
        if let Some(msg) = data
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return Err(format!("API error - {msg}"));
        }
        let content = data
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        let usage = data.get("usage").cloned().unwrap_or(json!({}));
        Ok(json!({
            "model": model,
            "provider": provider_name,
            "content": content,
            "usage": {
                "prompt_tokens": usage.get("prompt_tokens").unwrap_or(&json!(0)),
                "completion_tokens": usage.get("completion_tokens").unwrap_or(&json!(0)),
                "total_tokens": usage.get("total_tokens").unwrap_or(&json!(0)),
            }
        }))
    }

    async fn call_with_fallback(
        &self,
        providers: Vec<VisionProvider>,
        question: &str,
        image_block: &Value,
    ) -> ToolRunResult {
        let default_model =
            user_vision_model(&self.config).unwrap_or_else(|| default_vision_model().to_string());
        let mut errors = Vec::new();
        for (i, provider) in providers.iter().enumerate() {
            let model = provider
                .model_override
                .as_deref()
                .unwrap_or(default_model.as_str());
            info!(
                provider = %provider.name,
                model = %model,
                attempt = i + 1,
                total = providers.len(),
                "Vision trying provider"
            );
            match self
                .call_provider(provider, model, question, image_block)
                .await
            {
                Ok(result) => {
                    return ToolRunResult::success(result);
                }
                Err(e) => {
                    warn!(provider = %provider.name, error = %e, "Vision provider failed");
                    errors.push(format!("[{}/{}] {}", provider.name, model, e));
                }
            }
        }
        ToolRunResult::error(format!(
            "Error: All Vision API providers failed.\n{}",
            errors
                .iter()
                .map(|e| format!("  - {e}"))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

#[async_trait]
impl AgentTool for VisionTool {
    fn name(&self) -> &str {
        "vision"
    }

    fn description(&self) -> &str {
        "分析本地图片或图片 URL（jpg/jpeg/png），可描述内容、提取文字、识别物体与颜色等。"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "image": {
                    "type": "string",
                    "description": "Local file path or HTTP(S) URL of the image to analyze"
                },
                "question": {
                    "type": "string",
                    "description": "Question to ask about the image"
                }
            },
            "required": ["image", "question"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let image = params
            .get("image")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let question = params
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if image.is_empty() {
            return ToolRunResult::error("Error: 'image' parameter is required");
        }
        if question.is_empty() {
            return ToolRunResult::error("Error: 'question' parameter is required");
        }

        let providers = resolve_providers(self.config.clone());
        if providers.is_empty() {
            return ToolRunResult::error(
                "Error: No model available for Vision.\n\
                 Configure OPENAI_API_KEY or LINKAI_API_KEY via env_config, or set another vendor API key in config.json.",
            );
        }

        let image_block = match self.build_image_content(image).await {
            Ok(v) => v,
            Err(e) => return ToolRunResult::error(format!("Error: {e}")),
        };

        self.call_with_fallback(providers, question, &image_block)
            .await
    }
}
