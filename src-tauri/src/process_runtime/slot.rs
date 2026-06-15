//! 懒加载进程 slot：get-or-spawn 模式，供 `context::ProcessHub` 复用。

use std::sync::Arc;

use tokio::sync::Mutex;

pub struct ProcessSlot<T> {
    handle: Mutex<Option<Arc<T>>>,
}

impl<T> ProcessSlot<T> {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
        }
    }

    pub async fn get(&self) -> Option<Arc<T>> {
        self.handle.lock().await.clone()
    }

    pub async fn set(&self, value: Arc<T>) {
        *self.handle.lock().await = Some(value);
    }

    /// 若 slot 为空则调用 `spawn` 拉起并缓存；已存在则直接返回。
    pub async fn ensure<F, Fut>(&self, spawn: F) -> Result<Arc<T>, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Arc<T>, String>>,
    {
        if let Some(existing) = self.get().await {
            return Ok(existing);
        }
        let created = spawn().await?;
        self.set(created.clone()).await;
        Ok(created)
    }

    pub async fn clear(&self) {
        *self.handle.lock().await = None;
    }
}

impl<T> Default for ProcessSlot<T> {
    fn default() -> Self {
        Self::new()
    }
}
