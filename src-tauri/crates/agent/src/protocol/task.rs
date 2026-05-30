//! `agent/protocol/task.py`

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Text,
    Image,
    Video,
    Audio,
    File,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Init,
    Processing,
    Completed,
    Failed,
}

/// Task processed by an agent (`agent.protocol.task.Task`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub content: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub created_at: f64,
    pub updated_at: f64,
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub videos: Vec<String>,
    #[serde(default)]
    pub audios: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
}

impl Default for Task {
    fn default() -> Self {
        Self::new("")
    }
}

impl Task {
    pub fn new(content: impl Into<String>) -> Self {
        let now = now_secs();
        Self {
            id: Uuid::new_v4().to_string(),
            content: content.into(),
            task_type: TaskType::Text,
            status: TaskStatus::Init,
            created_at: now,
            updated_at: now,
            metadata: serde_json::Map::new(),
            images: Vec::new(),
            videos: Vec::new(),
            audios: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn get_text(&self) -> &str {
        &self.content
    }

    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = now_secs();
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}
