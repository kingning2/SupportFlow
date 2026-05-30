//! Shared prompt types.

#[derive(Debug, Clone)]
pub struct ContextFile {
    pub path: String,
    pub content: String,
}
