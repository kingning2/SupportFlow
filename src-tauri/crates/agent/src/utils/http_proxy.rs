//! Re-export `common/http_proxy.py` from the models crate (shared with LLM HTTP).

pub use models::http_proxy::{build_reqwest_client, log_http_proxy_settings, HttpProxySettings};
