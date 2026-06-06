//! 渠道领域服务：配置持久化、动作编排响应与无状态规则。

mod config;

pub use config::{
    action_response, connect_channel, disconnect_channel, persist_channel_config,
    should_restart_channel,
};
