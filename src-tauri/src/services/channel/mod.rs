//! 渠道领域服务：配置持久化、动作编排响应与无状态规则。

mod config;
mod contract;
mod registry;

pub use config::{
    action_response, connect_channel, disconnect_channel, persist_channel_config,
    should_restart_channel,
};
pub use contract::{
    error_code, event, inbound_rpc, phase, sidecar_rpc, ChannelAdapterCapability, ChannelTypeId,
};
pub use registry::{
    all_channel_defs, channel_def, channel_field_type_map, channel_restart_keys, config_schema_for,
    is_known_channel, ChannelCapability, ChannelDef, ChannelFieldDef,
};
