use std::io::{self, Write};

use anyhow::Result;
use models::{Context as AgentContext, ContextType};

use crate::runtime::CliRuntime;

#[derive(clap::Subcommand)]
pub enum AgentCommand {
    /// Send one message and print the reply
    Chat {
        message: String,
        /// Session id for persistence
        #[arg(short, long, default_value = "cli-default")]
        session: String,
        /// Clear conversation history before this turn
        #[arg(long)]
        clear: bool,
        /// Stream tool/log events to stderr
        #[arg(long)]
        verbose: bool,
    },
    /// Interactive REPL (type `exit` or Ctrl+C to quit)
    Repl {
        #[arg(short, long, default_value = "cli-default")]
        session: String,
        #[arg(long)]
        clear: bool,
        #[arg(long)]
        verbose: bool,
    },
}

pub async fn run_command(command: AgentCommand) -> Result<()> {
    match command {
        AgentCommand::Chat {
            message,
            session,
            clear,
            verbose,
        } => {
            let rt = CliRuntime::load()?;
            let reply = chat_once(&rt, &message, &session, clear, verbose).await?;
            println!("{}", format_reply(&reply));
        }
        AgentCommand::Repl {
            session,
            clear,
            verbose,
        } => {
            let rt = CliRuntime::load()?;
            println!("SupportFlow agent REPL (session: {session}). Type 'exit' to quit.\n");
            let mut first = clear;
            loop {
                print!("> ");
                io::stdout().flush()?;
                let mut line = String::new();
                if io::stdin().read_line(&mut line)? == 0 {
                    break;
                }
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line.eq_ignore_ascii_case("exit") || line.eq_ignore_ascii_case("quit") {
                    break;
                }
                let reply = chat_once(&rt, line, &session, first, verbose).await?;
                println!("\n{}\n", format_reply(&reply));
                first = false;
            }
        }
    }
    Ok(())
}

async fn chat_once(
    rt: &CliRuntime,
    message: &str,
    session: &str,
    clear_history: bool,
    verbose: bool,
) -> Result<models::Reply> {
    let mut ctx = AgentContext::new(ContextType::Text, message);
    ctx.kwargs.insert("session_id".into(), session.to_string());
    ctx.kwargs.insert("channel_type".into(), "cli".to_string());

    let on_event = if verbose {
        Some(std::sync::Arc::new(move |ev: agent::AgentEvent| {
            eprintln!("[agent] {ev:?}");
        }) as agent::AgentEventCallback)
    } else {
        None
    };

    let reply = rt
        .stack
        .reply(message, Some(ctx), true, clear_history, on_event)
        .await;
    Ok(reply)
}

fn format_reply(reply: &models::Reply) -> String {
    reply
        .text_content
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| reply.content.clone())
}
