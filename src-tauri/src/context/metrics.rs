//! Simple local metrics counters (JSON file under workspace `.supportflow/`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetricsSnapshot {
    #[serde(default)]
    counters: HashMap<String, u64>,
    #[serde(default)]
    latency_totals_ms: HashMap<String, u64>,
    #[serde(default)]
    latency_samples: HashMap<String, u64>,
}

pub struct MetricsStore {
    path: PathBuf,
    inner: Mutex<MetricsSnapshot>,
}

impl MetricsStore {
    pub fn for_workspace(workspace: &Path) -> Result<Self, String> {
        let path = workspace.join(".supportflow").join("metrics.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let snapshot = if path.is_file() {
            let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            MetricsSnapshot::default()
        };
        Ok(Self {
            path,
            inner: Mutex::new(snapshot),
        })
    }

    pub fn increment(&self, name: &str) -> Result<u64, String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        let entry = guard.counters.entry(name.to_string()).or_insert(0);
        *entry += 1;
        let value = *entry;
        self.flush(&guard)?;
        Ok(value)
    }

    pub fn record_latency_ms(&self, name: &str, ms: u128) -> Result<(), String> {
        let mut guard = self.inner.lock().map_err(|e| e.to_string())?;
        *guard.latency_totals_ms.entry(name.to_string()).or_insert(0) += ms as u64;
        *guard.latency_samples.entry(name.to_string()).or_insert(0) += 1;
        self.flush(&guard)
    }

    pub fn snapshot(&self) -> Result<MetricsSnapshot, String> {
        let guard = self.inner.lock().map_err(|e| e.to_string())?;
        Ok(guard.clone())
    }

    fn flush(&self, guard: &MetricsSnapshot) -> Result<(), String> {
        let json = serde_json::to_string_pretty(guard).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, json).map_err(|e| e.to_string())
    }
}
