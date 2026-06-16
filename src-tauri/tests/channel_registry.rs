//! Channel registry unit tests.

#[test]
fn wework_is_registered_with_full_capabilities() {
    use tauri_app_lib::services::channel::{
        all_channel_defs, channel_def, config_schema_for, is_known_channel, ChannelCapability,
    };

    assert!(is_known_channel("wework"));
    assert!(!is_known_channel("telegram"));

    let def = channel_def("wework").expect("wework def");
    assert_eq!(def.name, "wework");
    assert!(def.capabilities.contains(&ChannelCapability::Connect));
    assert!(def.capabilities.contains(&ChannelCapability::Send));

    let schema = config_schema_for("wework").expect("schema");
    assert_eq!(schema["channel_type"], "wework");
    assert!(schema["fields"].as_array().unwrap().len() >= 1);

    assert_eq!(all_channel_defs().len(), 1);
}

#[test]
fn unknown_channel_config_returns_error_code() {
    use std::collections::HashMap;
    use tauri_app_lib::services::channel::persist_channel_config;
    use tempfile::TempDir;

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.json");
    std::fs::write(&path, r#"{"channel_type":""}"#).unwrap();

    let err = persist_channel_config(&path, "unknown", &HashMap::new()).unwrap_err();
    assert!(err.contains("channel.unknown"));
}
