//! Profile store CRUD tests.

use serde_json::json;
use tauri_app_lib::services::agent::profile::ProfileStore;

#[test]
fn profile_traits_persist_across_reads() {
    let dir = std::env::temp_dir().join(format!("sf-profile-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = ProfileStore::open(&dir.join("profiles/index.db")).expect("open profile db");

    let mut patch = serde_json::Map::new();
    patch.insert("tone".into(), json!("concise"));
    store
        .update_traits("user-1", "wework", patch)
        .expect("update");

    let traits = store.get_traits("user-1", "wework").expect("get");
    assert_eq!(traits.get("tone").and_then(|v| v.as_str()), Some("concise"));

    let mut patch2 = serde_json::Map::new();
    patch2.insert("lang".into(), json!("zh"));
    store
        .update_traits("user-1", "wework", patch2)
        .expect("merge");

    let merged = store.get_traits("user-1", "wework").expect("get merged");
    assert_eq!(merged.get("tone").and_then(|v| v.as_str()), Some("concise"));
    assert_eq!(merged.get("lang").and_then(|v| v.as_str()), Some("zh"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn profile_isolated_per_channel() {
    let dir = std::env::temp_dir().join(format!("sf-profile-ch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let store = ProfileStore::open(&dir.join("profiles/index.db")).expect("open");

    let mut a = serde_json::Map::new();
    a.insert("tier".into(), json!("vip"));
    store.update_traits("u1", "wework", a).expect("wework");

    let mut b = serde_json::Map::new();
    b.insert("tier".into(), json!("standard"));
    store.update_traits("u1", "web", b).expect("web");

    assert_eq!(
        store
            .get_traits("u1", "wework")
            .unwrap()
            .get("tier")
            .and_then(|v| v.as_str()),
        Some("vip")
    );
    assert_eq!(
        store
            .get_traits("u1", "web")
            .unwrap()
            .get("tier")
            .and_then(|v| v.as_str()),
        Some("standard")
    );

    let _ = std::fs::remove_dir_all(&dir);
}
