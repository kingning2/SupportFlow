use anyhow::Result;

use crate::services::agent::conversation_store_for_workspace;

use crate::cli::paths;
use crate::cli::runtime::CliRuntime;

#[derive(clap::Subcommand)]
pub enum ContextCommand {
    /// Clear persisted messages for a session
    Clear {
        #[arg(short, long, default_value = "cli-default")]
        session: String,
    },
}

pub fn run(command: Option<ContextCommand>) -> Result<()> {
    match command {
        None => {
            println!(
                "\n  Context commands:\n\n    sf context clear [--session ID]\n\n  In the desktop app, use /context or /context clear in chat.\n"
            );
        }
        Some(ContextCommand::Clear { session }) => {
            let rt = CliRuntime::load()?;
            rt.stack.agent_bridge.clear_session(&session);
            let store = conversation_store_for_workspace(&rt.workspace)
                .map_err(|e| anyhow::anyhow!("open conversation store: {e}"))?;
            let n = store
                .clear_context(&session)
                .map_err(|e| anyhow::anyhow!("clear context: {e}"))?;
            println!("Cleared session '{session}' ({n} messages removed).");
            println!("  Workspace: {}", paths::resolve_workspace()?.display());
        }
    }
    Ok(())
}
