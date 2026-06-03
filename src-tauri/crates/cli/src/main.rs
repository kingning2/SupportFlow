//! SupportFlow CLI (`sf`) — Rust port of SupportFlow Agent `cli/cli.py`.

mod commands;
mod paths;
mod runtime;
mod skills_config;

use anyhow::Result;
use clap::{Parser, Subcommand};

const HELP_TEXT: &str = r#"Usage: sf COMMAND [ARGS]...

  SupportFlow CLI - Manage your SupportFlow instance.

Commands:
  help              Show this message.
  version           Show the version.
  start             Start SupportFlow desktop app.
  stop              Stop SupportFlow.
  restart           Restart SupportFlow.
  update            Update hints.
  status            Show running status.
  logs              View SupportFlow logs.
  agent             Headless agent (chat / repl).
  skill             Manage skills.
  knowledge         Manage knowledge base.
  config            Show paths and model config.
  context           Conversation context (clear session DB).
  install-browser   Browser tool setup.

Tip: Memory index management lives in chat — send /memory status or
/memory rebuild-index to the running agent."#;

#[derive(Parser)]
#[command(
    name = "sf",
    version,
    about = "SupportFlow CLI",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<TopCommand>,
}

#[derive(Subcommand)]
enum TopCommand {
    /// Show version
    Version,
    /// Show usage
    Help,
    /// Start desktop app
    Start(commands::process::StartArgs),
    /// Stop desktop app
    Stop,
    /// Restart desktop app
    Restart,
    /// Update hints
    Update,
    /// Running status
    Status,
    /// Tail log file
    Logs {
        #[arg(short, long, default_value_t = 100)]
        lines: usize,
    },
    /// Headless agent (chat / repl)
    Agent {
        #[command(subcommand)]
        command: commands::agent::AgentCommand,
    },
    /// Manage skills
    Skill {
        #[command(subcommand)]
        command: commands::skill::SkillCommand,
    },
    /// Manage knowledge base
    Knowledge {
        #[command(subcommand)]
        command: Option<commands::knowledge::KnowledgeCommand>,
    },
    /// Show configuration
    Config {
        #[command(subcommand)]
        command: Option<commands::config_cmd::ConfigCommand>,
    },
    /// Conversation context
    Context {
        #[command(subcommand)]
        command: Option<commands::context::ContextCommand>,
    },
    /// Browser tool setup
    InstallBrowser(commands::install_browser::InstallBrowserArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        None => println!("{HELP_TEXT}"),
        Some(TopCommand::Version) => println!("sf {}", env!("CARGO_PKG_VERSION")),
        Some(TopCommand::Help) => println!("{HELP_TEXT}"),
        Some(TopCommand::Start(a)) => commands::process::cmd_start(a)?,
        Some(TopCommand::Stop) => commands::process::cmd_stop()?,
        Some(TopCommand::Restart) => commands::process::cmd_restart()?,
        Some(TopCommand::Update) => commands::process::cmd_update()?,
        Some(TopCommand::Status) => commands::process::cmd_status()?,
        Some(TopCommand::Logs { lines }) => commands::process::cmd_logs(lines)?,
        Some(TopCommand::Agent { command }) => {
            commands::agent::run_command(command).await?
        }
        Some(TopCommand::Skill { command }) => commands::skill::run_command(command)?,
        Some(TopCommand::Knowledge { command }) => {
            commands::knowledge::run(command)?
        }
        Some(TopCommand::Config { command }) => commands::config_cmd::run(command)?,
        Some(TopCommand::Context { command }) => commands::context::run(command)?,
        Some(TopCommand::InstallBrowser(a)) => commands::install_browser::run(a)?,
    }
    Ok(())
}
