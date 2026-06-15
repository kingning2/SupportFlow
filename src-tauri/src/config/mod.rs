//! 应用配置与 IPC 契约类型（`config.json`、Provider 目录、Context/Reply）。

pub mod bridge;
pub mod catalog;
pub mod const_;
pub mod models_config;
pub mod provider_catalog;

pub use bridge::{Context, ContextType, Reply, ReplyType};
pub use catalog::{list_providers, provider_configured, ModelProviderDescriptor};
pub use const_::BotType;
pub use models_config::{BrowserConfig, ModelsConfig, ToolsConfig, VisionConfig, WebSearchConfig};
pub use provider_catalog::{
    build_provider_details, clear_provider_credentials, find_provider_meta, mask_api_key,
    patch_config_file, set_chat_model, update_provider_credentials, ProviderDetail, ProviderMeta,
    PROVIDER_METAS,
};
