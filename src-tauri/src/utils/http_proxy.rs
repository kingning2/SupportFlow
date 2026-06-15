//! 全局 HTTP 代理规则（`reqwest` 客户端构建）。

use std::time::Duration;

use reqwest::Client;
use tracing::info;

use crate::config::ModelsConfig;

#[derive(Debug, Clone, Default)]
pub struct HttpProxySettings {
    pub use_proxy: bool,
    pub proxy: Option<String>,
}

impl HttpProxySettings {
    pub fn from_config(config: &ModelsConfig) -> Self {
        Self {
            use_proxy: config.use_proxy.unwrap_or(false),
            proxy: config.proxy.clone().filter(|s| !s.trim().is_empty()),
        }
    }

    #[deprecated(note = "use from_config")]
    pub fn from_models(config: &ModelsConfig) -> Self {
        Self::from_config(config)
    }

    fn system_proxy_url() -> Option<String> {
        for key in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
            if let Ok(v) = std::env::var(key) {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
        None
    }

    fn resolved_proxy_url(&self) -> Option<String> {
        if !self.use_proxy {
            return None;
        }
        if let Some(ref url) = self.proxy {
            return Some(url.clone());
        }
        Self::system_proxy_url()
    }

    pub fn describe(&self) -> String {
        if !self.use_proxy {
            if let Some(ignored) = Self::system_proxy_url() {
                return format!("disabled (use_proxy=false); ignoring system proxy {ignored}");
            }
            return "disabled (use_proxy=false); direct connection".into();
        }
        if let Some(ref url) = self.proxy {
            return format!("enabled via config proxy={url}");
        }
        if let Some(env) = Self::system_proxy_url() {
            return format!("enabled via environment ({env})");
        }
        "enabled (use_proxy=true) but no proxy URL configured; direct connection".into()
    }
}

pub fn log_http_proxy_settings(settings: &HttpProxySettings) {
    info!("[HTTP] Proxy mode: {}", settings.describe());
}

pub fn build_reqwest_client(
    settings: &HttpProxySettings,
    timeout: Duration,
    extra_headers: Option<reqwest::header::HeaderMap>,
) -> Client {
    let mut builder = Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(10));

    if let Some(headers) = extra_headers {
        builder = builder.default_headers(headers);
    }

    if !settings.use_proxy {
        builder = builder.no_proxy();
    } else if let Some(url) = settings.resolved_proxy_url() {
        if let Ok(proxy) = reqwest::Proxy::all(&url) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build().unwrap_or_else(|_| Client::new())
}
