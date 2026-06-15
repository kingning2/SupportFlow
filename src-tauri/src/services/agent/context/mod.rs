//! System prompt assembly — tools, skills, memory, knowledge, workspace context.

mod skills_format;
mod system_prompt;
mod types;
mod workspace;

pub use skills_format::format_skills_for_prompt;
pub use system_prompt::{build_agent_system_prompt, PromptBuilder};
pub use types::ContextFile;
pub use workspace::load_context_files;
