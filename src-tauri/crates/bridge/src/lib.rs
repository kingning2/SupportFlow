//! SupportFlow Agent `bridge/` package — bot routing, `AgentBridge`, initializer, events.
//!
//! Replaces Python `bridge/bridge.py`, `agent_bridge.py`, `agent_initializer.py`,
//! `agent_event_handler.py` for the Tauri desktop runtime.

mod agent_bridge;
mod agent_event_handler;
mod agent_initializer;
mod bot_router;
mod bridge;
mod config_sync;
mod context_params;

pub use agent_bridge::AgentBridge;
pub use agent_event_handler::AgentEventHandler;
pub use agent_initializer::{AgentInitOptions, AgentInitializer};
pub use bot_router::{auto_pick_voice_to_text, resolve_bot_type};
pub use bridge::Bridge;
pub use config_sync::{load_dotenv_into_process, sync_config_to_dotenv};
pub use context_params::context_from_reply_params;

use std::path::PathBuf;
use std::sync::Arc;

use agent::McpToolLoader;
use models::ModelsConfig;

/// Shared bridge stack for one runtime workspace + config.
pub struct BridgeRuntime {
    pub bridge: Arc<Bridge>,
    pub agent_bridge: Arc<AgentBridge>,
}

impl BridgeRuntime {
    pub fn new(
        workspace: PathBuf,
        config: Arc<ModelsConfig>,
        mcp_loader: Arc<McpToolLoader>,
    ) -> Self {
        let bridge = Arc::new(Bridge::new(config.clone()));
        let agent_bridge = AgentBridge::new(bridge.clone(), workspace, config, mcp_loader);
        bridge.attach_agent_bridge(agent_bridge.clone());
        Self {
            bridge,
            agent_bridge,
        }
    }

    pub async fn reply(
        &self,
        query: &str,
        context: Option<models::Context>,
        use_agent: bool,
        clear_history: bool,
        on_event: Option<agent::AgentEventCallback>,
    ) -> models::Reply {
        if use_agent || self.bridge.config.agent_enabled() {
            self.agent_bridge
                .agent_reply(query, context, on_event, clear_history)
                .await
        } else {
            self.bridge
                .fetch_reply_content(query, context.as_ref())
                .await
        }
    }
}
