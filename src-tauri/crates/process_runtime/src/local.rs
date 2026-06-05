//! 单个子进程实例的本地可变状态。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::Value;
use tokio::process::{Child, ChildStdin};
use tokio::sync::{oneshot, Mutex};

use crate::stdin::StdinLineWriter;

pub struct ProcessLocalState {
    pub tokio_child: Mutex<Option<Child>>,
    pub stdin: Mutex<Option<ChildStdin>>,
    pub pending: Mutex<HashMap<u64, oneshot::Sender<Value>>>,
    /// 外部托管子进程（如 Tauri shell sidecar）存活标记。
    pub external_alive: AtomicBool,
    pub external_stdin: Mutex<Option<Arc<dyn StdinLineWriter>>>,
}

impl ProcessLocalState {
    pub fn new() -> Self {
        Self {
            tokio_child: Mutex::new(None),
            stdin: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            external_alive: AtomicBool::new(false),
            external_stdin: Mutex::new(None),
        }
    }

    pub async fn clear(&self) {
        self.external_alive.store(false, Ordering::SeqCst);
        *self.external_stdin.lock().await = None;
        *self.tokio_child.lock().await = None;
        *self.stdin.lock().await = None;
        self.pending.lock().await.clear();
    }
}

impl Default for ProcessLocalState {
    fn default() -> Self {
        Self::new()
    }
}
