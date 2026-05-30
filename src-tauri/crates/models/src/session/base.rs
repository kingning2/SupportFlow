//! `models/session_manager.py` — `Session` base class.

use serde_json::{json, Value};

/// Base session (mirrors Python `Session`).
#[derive(Debug, Clone)]
pub struct BaseSession {
    pub session_id: String,
    pub system_prompt: String,
    pub messages: Vec<Value>,
}

impl BaseSession {
    pub fn new(
        session_id: impl Into<String>,
        system_prompt: Option<String>,
        default_desc: &str,
    ) -> Self {
        let system_prompt = system_prompt.unwrap_or_else(|| default_desc.to_string());
        let mut s = Self {
            session_id: session_id.into(),
            system_prompt: system_prompt.clone(),
            messages: Vec::new(),
        };
        s.reset();
        s
    }

    pub fn reset(&mut self) {
        self.messages = vec![json!({
            "role": "system",
            "content": self.system_prompt,
        })];
    }

    pub fn set_system_prompt(&mut self, system_prompt: impl Into<String>) {
        self.system_prompt = system_prompt.into();
        self.reset();
    }

    pub fn add_query(&mut self, query: &str) {
        self.messages
            .push(json!({ "role": "user", "content": query }));
    }

    pub fn add_reply(&mut self, reply: &str) {
        self.messages
            .push(json!({ "role": "assistant", "content": reply }));
    }
}

/// Build OpenAI legacy prompt string (`open_ai_session.__str__`).
pub fn openai_session_prompt(messages: &[Value]) -> String {
    let mut prompt = String::new();
    for item in messages {
        let role = item.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let content = item.get("content").and_then(|c| c.as_str()).unwrap_or("");
        match role {
            "system" => {
                prompt.push_str(content);
                prompt.push_str("<|endoftext|>\n\n\n");
            }
            "user" => {
                prompt.push_str("Q: ");
                prompt.push_str(content);
                prompt.push('\n');
            }
            "assistant" => {
                prompt.push_str("\n\nA: ");
                prompt.push_str(content);
                prompt.push_str("<|endoftext|>\n");
            }
            _ => {}
        }
    }
    if messages
        .last()
        .and_then(|m| m.get("role"))
        .and_then(|r| r.as_str())
        == Some("user")
    {
        prompt.push_str("A: ");
    }
    prompt
}
