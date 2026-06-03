use anyhow::Result;

use crate::paths;
use crate::runtime::CliRuntime;

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Show workspace, config path, and model summary
    Show,
}

pub fn run(command: Option<ConfigCommand>) -> Result<()> {
    match command.unwrap_or(ConfigCommand::Show) {
        ConfigCommand::Show => show(),
    }
}

fn show() -> Result<()> {
    let rt = CliRuntime::load()?;
    let workspace = &rt.workspace;
    println!("\n  SupportFlow configuration\n");
    println!("  Workspace:  {}", workspace.display());
    println!("  Config:     {}", rt.config_path.display());
    println!("  Skills:     {}", paths::skills_dir(workspace).display());
    println!("  Knowledge:  {}", paths::knowledge_dir(workspace).display());
    println!(
        "  Bot:        {} ({})",
        rt.config.bot_type,
        rt.config.model.as_deref().unwrap_or("default")
    );
    println!(
        "  Agent mode: {}",
        if rt.config.agent_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!(
        "  Persistence: {}",
        if rt.config.conversation_persistence.unwrap_or(true) {
            "on"
        } else {
            "off"
        }
    );
    println!();
    Ok(())
}
