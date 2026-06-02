//! Shared utilities (`common/` helpers in SupportFlow Agent).

pub mod http_proxy;

pub use http_proxy::{build_reqwest_client, log_http_proxy_settings, HttpProxySettings};
