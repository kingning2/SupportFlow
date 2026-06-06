//! Provider resolution for `web_search` (`agent/tools/web_search/web_search.py`).

use models::ModelsConfig;

use crate::services::agent::tools::env_config::read_env_file;
use crate::services::agent::tools::utils::path::supportflow_env_file;

pub const PROVIDER_ORDER: &[&str] = &["bocha", "qianfan", "zhipu", "linkai"];

const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub fn default_timeout_secs() -> u64 {
    DEFAULT_TIMEOUT_SECS
}

#[derive(Debug, Clone)]
pub struct WebSearchSettings {
    pub strategy: String,
    pub pinned_provider: String,
    pub bocha_api_key: String,
    pub zhipu_ai_api_key: String,
    pub zhipu_ai_api_base: String,
    pub zhipu_search_engine: String,
    pub zhipu_content_size: String,
    pub qianfan_api_key: String,
    pub qianfan_api_base: String,
    pub linkai_api_key: String,
    pub linkai_api_base: String,
}

impl WebSearchSettings {
    pub fn from_models(config: &ModelsConfig) -> Self {
        let dotenv = read_env_file(&supportflow_env_file());
        let ws = config.tools.as_ref().and_then(|t| t.web_search.as_ref());

        Self {
            strategy: ws
                .and_then(|w| w.strategy.clone())
                .unwrap_or_else(|| "auto".into())
                .trim()
                .to_lowercase(),
            pinned_provider: ws
                .and_then(|w| w.provider.clone())
                .unwrap_or_default()
                .trim()
                .to_lowercase(),
            bocha_api_key: resolve_key(
                ws.and_then(|w| w.bocha_api_key.as_deref()),
                &dotenv,
                "BOCHA_API_KEY",
            ),
            zhipu_ai_api_key: resolve_key(
                config.zhipu_ai_api_key.as_deref(),
                &dotenv,
                "ZHIPUAI_API_KEY",
            ),
            zhipu_ai_api_base: config
                .zhipu_ai_api_base
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "https://open.bigmodel.cn/api/paas/v4".into()),
            zhipu_search_engine: ws
                .and_then(|w| w.zhipu_search_engine.clone())
                .unwrap_or_else(|| "search_pro".into())
                .trim()
                .to_lowercase(),
            zhipu_content_size: ws
                .and_then(|w| w.zhipu_content_size.clone())
                .unwrap_or_default()
                .trim()
                .to_lowercase(),
            qianfan_api_key: resolve_key(
                config.qianfan_api_key.as_deref(),
                &dotenv,
                "QIANFAN_API_KEY",
            ),
            qianfan_api_base: config
                .qianfan_api_base
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "https://qianfan.baidubce.com/v2".into()),
            linkai_api_key: resolve_key(
                config.linkai_api_key.as_deref(),
                &dotenv,
                "LINKAI_API_KEY",
            ),
            linkai_api_base: config
                .linkai_api_base
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "https://api.link-ai.tech".into()),
        }
    }

    pub fn api_key_for(&self, provider: &str) -> &str {
        match provider {
            "bocha" => &self.bocha_api_key,
            "zhipu" => &self.zhipu_ai_api_key,
            "qianfan" => &self.qianfan_api_key,
            "linkai" => &self.linkai_api_key,
            _ => "",
        }
    }

    pub fn configured_providers(&self) -> Vec<String> {
        PROVIDER_ORDER
            .iter()
            .filter(|p| !self.api_key_for(p).is_empty())
            .map(|p| (*p).to_string())
            .collect()
    }

    pub fn is_available(&self) -> bool {
        !self.configured_providers().is_empty()
    }

    pub fn resolve_provider(&self, requested: Option<&str>) -> Option<String> {
        let available = self.configured_providers();
        if available.is_empty() {
            return None;
        }

        if let Some(req) = requested.map(str::trim).filter(|s| !s.is_empty()) {
            let req = req.to_lowercase();
            if available.iter().any(|p| p == &req) {
                return Some(req);
            }
            tracing::warn!(
                requested = %req,
                "WebSearch requested provider unavailable, falling back"
            );
        }

        if self.strategy == "fixed" {
            let pinned = self.pinned_provider.trim().to_lowercase();
            if !pinned.is_empty() && available.iter().any(|p| p == &pinned) {
                return Some(pinned);
            }
            if !pinned.is_empty() {
                tracing::warn!(
                    pinned = %pinned,
                    "WebSearch pinned provider unavailable, falling back to auto"
                );
            }
        }

        available.into_iter().next()
    }
}

fn resolve_key(
    config_val: Option<&str>,
    dotenv: &std::collections::HashMap<String, String>,
    env_name: &str,
) -> String {
    config_val
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| dotenv.get(env_name).cloned())
        .or_else(|| std::env::var(env_name).ok())
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_picks_first_configured_provider() {
        let cfg = ModelsConfig {
            linkai_api_key: Some("lk".into()),
            ..Default::default()
        };
        let settings = WebSearchSettings::from_models(&cfg);
        assert_eq!(settings.resolve_provider(None).as_deref(), Some("linkai"));
    }

    #[test]
    fn respects_explicit_provider_when_configured() {
        let cfg = ModelsConfig {
            linkai_api_key: Some("lk".into()),
            tools: Some(models::ToolsConfig {
                web_search: Some(models::WebSearchConfig {
                    bocha_api_key: Some("b".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let settings = WebSearchSettings::from_models(&cfg);
        assert_eq!(
            settings.resolve_provider(Some("linkai")).as_deref(),
            Some("linkai")
        );
    }
}
