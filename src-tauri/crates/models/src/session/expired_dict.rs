//! `common/expired_dict.py`

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ExpiredDict<V> {
    expires_in: Duration,
    inner: HashMap<String, (V, Instant)>,
}

impl<V> ExpiredDict<V> {
    pub fn new(expires_in_seconds: u64) -> Self {
        Self {
            expires_in: Duration::from_secs(expires_in_seconds),
            inner: HashMap::new(),
        }
    }

    fn refresh_expiry(&self, at: Instant) -> Instant {
        at + self.expires_in
    }

    pub fn insert(&mut self, key: String, value: V) {
        let expiry = self.refresh_expiry(Instant::now());
        self.inner.insert(key, (value, expiry));
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut V> {
        let now = Instant::now();
        if let Some((_, expiry)) = self.inner.get(key) {
            if now > *expiry {
                self.inner.remove(key);
                return None;
            }
        } else {
            return None;
        }
        let new_expiry = self.refresh_expiry(now);
        let entry = self.inner.get_mut(key)?;
        entry.1 = new_expiry;
        Some(&mut entry.0)
    }

    pub fn contains_key(&mut self, key: &str) -> bool {
        self.get_mut(key).is_some()
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        self.inner.remove(key).map(|(v, _)| v)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
