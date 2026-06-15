//! `agent/tools/memory/memory_search.py`

use async_trait::async_trait;
use serde_json::Value;

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::memory::traits::MemoryManager;

pub struct MemorySearchTool {
    memory: std::sync::Arc<dyn MemoryManager>,
    user_id: Option<String>,
    description: String,
}

impl MemorySearchTool {
    pub fn new(
        memory: std::sync::Arc<dyn MemoryManager>,
        user_id: Option<String>,
        enable_knowledge: bool,
    ) -> Self {
        let description = if enable_knowledge {
            "在长期记忆与知识库中进行语义/关键词检索，用于召回历史对话、偏好与知识页。"
        } else {
            "在长期记忆中进行语义/关键词检索，用于召回历史对话与偏好。"
        }
        .to_string();
        Self {
            memory,
            user_id,
            description,
        }
    }
}

#[async_trait]
impl AgentTool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "max_results": { "type": "integer", "description": "Max results (default 10)" },
                "min_score": { "type": "number", "description": "Min relevance 0-1 (default 0.1)" }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;
        let min_score = params
            .get("min_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.1);

        if query.is_empty() {
            return ToolRunResult::error("Error: query parameter is required");
        }

        match self
            .memory
            .search(
                query,
                self.user_id.as_deref(),
                max_results,
                min_score,
            )
            .await
        {
            Ok(results) if results.is_empty() => ToolRunResult::success_text(format!(
                "No memories found for '{query}'. This is normal if no memories have been stored yet. You can store new memories by writing to MEMORY.md or memory/YYYY-MM-DD.md files."
            )),
            Ok(results) => {
                let mut output = vec![format!("Found {} relevant memories:\n", results.len())];
                for (i, r) in results.iter().enumerate() {
                    output.push(format!(
                        "\n{}. {} (lines {}-{})\n   Score: {:.3}\n   Snippet: {}",
                        i + 1,
                        r.path,
                        r.start_line,
                        r.end_line,
                        r.score,
                        r.snippet
                    ));
                }
                ToolRunResult::success_text(output.join(""))
            }
            Err(e) => ToolRunResult::error(format!("Error searching memory: {e}")),
        }
    }
}
