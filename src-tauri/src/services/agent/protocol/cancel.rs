//! `agent/protocol/cancel.py` — in-process cancel registry for agent runs.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use thiserror::Error;

/// Raised inside the agent loop when a stop has been requested.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("agent cancelled")]
pub struct AgentCancelledError;

/// Poll at safe checkpoints (`is_cancelled()`), same as Python `threading.Event.is_set()`.
#[derive(Debug, Clone)]
pub struct CancelHandle {
    flag: Arc<AtomicBool>,
}

impl CancelHandle {
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn trigger(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

struct CancelEntry {
    handle: CancelHandle,
    session_id: Option<String>,
}

/// In-process registry: `request_id` → cancel flag; `session_id` → in-flight request ids.
#[derive(Default)]
pub struct CancelTokenRegistry {
    inner: Mutex<RegistryInner>,
}

#[derive(Default)]
struct RegistryInner {
    by_request: HashMap<String, CancelEntry>,
    by_session: HashMap<String, HashSet<String>>,
}

impl CancelTokenRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create (or return existing) cancel handle for a request.
    pub fn register(&self, request_id: &str, session_id: Option<&str>) -> CancelHandle {
        if request_id.is_empty() {
            return CancelHandle {
                flag: Arc::new(AtomicBool::new(false)),
            };
        }
        let mut guard = self.inner.lock().expect("cancel registry lock");
        if let Some(entry) = guard.by_request.get(request_id) {
            return entry.handle.clone();
        }
        let handle = CancelHandle {
            flag: Arc::new(AtomicBool::new(false)),
        };
        let sid = session_id.map(str::to_string);
        guard.by_request.insert(
            request_id.to_string(),
            CancelEntry {
                handle: handle.clone(),
                session_id: sid.clone(),
            },
        );
        if let Some(s) = sid {
            guard
                .by_session
                .entry(s)
                .or_default()
                .insert(request_id.to_string());
        }
        handle
    }

    pub fn get_handle(&self, request_id: &str) -> Option<CancelHandle> {
        if request_id.is_empty() {
            return None;
        }
        let guard = self.inner.lock().expect("cancel registry lock");
        guard.by_request.get(request_id).map(|e| e.handle.clone())
    }

    /// Trigger cancel for a specific request. Returns `true` when matched.
    pub fn cancel_request(&self, request_id: &str) -> bool {
        if request_id.is_empty() {
            return false;
        }
        let handle = {
            let guard = self.inner.lock().expect("cancel registry lock");
            guard.by_request.get(request_id).map(|e| e.handle.clone())
        };
        if let Some(h) = handle {
            h.trigger();
            true
        } else {
            false
        }
    }

    /// Cancel every in-flight request for a session. Returns count cancelled.
    pub fn cancel_session(&self, session_id: &str) -> usize {
        if session_id.is_empty() {
            return 0;
        }
        let handles: Vec<CancelHandle> = {
            let guard = self.inner.lock().expect("cancel registry lock");
            guard
                .by_session
                .get(session_id)
                .map(|ids| {
                    ids.iter()
                        .filter_map(|rid| guard.by_request.get(rid).map(|e| e.handle.clone()))
                        .collect()
                })
                .unwrap_or_default()
        };
        let n = handles.len();
        for h in handles {
            h.trigger();
        }
        n
    }

    /// Remove an entry once the agent run is done. Safe to call twice.
    pub fn unregister(&self, request_id: &str) {
        if request_id.is_empty() {
            return;
        }
        let mut guard = self.inner.lock().expect("cancel registry lock");
        let Some(entry) = guard.by_request.remove(request_id) else {
            return;
        };
        if let Some(sid) = entry.session_id {
            if let Some(bucket) = guard.by_session.get_mut(&sid) {
                bucket.remove(request_id);
                if bucket.is_empty() {
                    guard.by_session.remove(&sid);
                }
            }
        }
    }

    pub fn has_active(&self, session_id: &str) -> bool {
        if session_id.is_empty() {
            return false;
        }
        let guard = self.inner.lock().expect("cancel registry lock");
        guard
            .by_session
            .get(session_id)
            .is_some_and(|b| !b.is_empty())
    }
}

static REGISTRY: OnceLock<CancelTokenRegistry> = OnceLock::new();

pub fn get_cancel_registry() -> &'static CancelTokenRegistry {
    REGISTRY.get_or_init(CancelTokenRegistry::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_cancel_unregister() {
        let reg = CancelTokenRegistry::new();
        let h = reg.register("req-1", Some("sess-a"));
        assert!(!h.is_cancelled());
        assert!(reg.has_active("sess-a"));
        assert!(reg.cancel_request("req-1"));
        assert!(h.is_cancelled());
        reg.unregister("req-1");
        assert!(!reg.has_active("sess-a"));
    }

    #[test]
    fn cancel_session_cancels_all() {
        let reg = CancelTokenRegistry::new();
        let h1 = reg.register("r1", Some("s1"));
        let h2 = reg.register("r2", Some("s1"));
        assert_eq!(reg.cancel_session("s1"), 2);
        assert!(h1.is_cancelled());
        assert!(h2.is_cancelled());
    }
}
