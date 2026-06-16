//! User profile persistence (`user_profiles` SQLite table).

mod store;

pub use store::{profile_db_path, profile_store_for_path, ProfileStore};

use std::sync::{Arc, Mutex};

/// Mutable scope for the current channel reply (user + channel binding).
#[derive(Debug, Clone, Default)]
pub struct ProfileScope {
    pub user_id: Option<String>,
    pub channel: String,
}

pub type SharedProfileScope = Arc<Mutex<ProfileScope>>;

pub fn new_profile_scope(channel: impl Into<String>) -> SharedProfileScope {
    Arc::new(Mutex::new(ProfileScope {
        user_id: None,
        channel: channel.into(),
    }))
}
