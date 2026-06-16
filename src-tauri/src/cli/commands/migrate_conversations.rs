//! `sf migrate-conversations` — move legacy sessions/messages into the conversation DB.

use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

use crate::cli::paths;
use crate::services::agent::memory::migrate_conversations_for_workspace;

#[derive(Args)]
pub struct MigrateConversationsArgs {
    /// Workspace directory (default: `SUPPORT_FLOW_WORKSPACE` or OS data dir)
    #[arg(long)]
    pub workspace: Option<PathBuf>,
}

pub fn run(args: MigrateConversationsArgs) -> Result<()> {
    let workspace = args
        .workspace
        .map(Ok)
        .unwrap_or_else(paths::resolve_workspace)?;

    if !workspace.is_dir() {
        anyhow::bail!("workspace not found: {}", workspace.display());
    }

    let report = migrate_conversations_for_workspace(&workspace)
        .map_err(|e| anyhow::anyhow!("migrate conversations: {e}"))?;

    if !report.migrated {
        println!("No legacy conversation data to migrate.");
    } else {
        println!(
            "Migrated {} session(s) and {} message(s).",
            report.sessions_copied, report.messages_copied
        );
    }
    println!("  Workspace: {}", workspace.display());
    println!(
        "  Conversation DB: {}",
        crate::services::agent::memory::MemoryConfig::new(&workspace)
            .conversation_db_path()
            .display()
    );

    Ok(())
}
