//! Shared context types for system prompt assembly.

#[derive(Debug, Clone)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
}
