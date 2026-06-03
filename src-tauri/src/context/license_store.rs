//! Shared subscription license status across all Webviews.
//!
//! Machine code is computed once at startup. Activation token is read from
//! app data (writable) with fallback to bundled resources.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use crate::utils::license::{compute_machine_code_windows, verify_license_token};

pub struct LicenseStore(pub Mutex<LicenseStatus>);

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatusDto {
    pub machine_code: String,
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LicenseStatus {
    pub valid: bool,
    pub reason: Option<String>,
    pub machine_code: String,
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn is_dev_license_bypass_enabled() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    // Enable bypass for local single-channel dev launcher by default.
    // Can be disabled with SUPPORT_FLOW_DEV_BYPASS_LICENSE=0/false/off.
    let has_dev_channel = std::env::var("DEV_CHANNEL")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    if !has_dev_channel {
        return false;
    }
    match std::env::var("SUPPORT_FLOW_DEV_BYPASS_LICENSE") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "off")
        }
        Err(_) => true,
    }
}

fn read_optional_resource_text(app: &AppHandle, filename: &str) -> Result<Option<String>, String> {
    let path = app
        .path()
        .resolve(filename, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Ok(None);
    }
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .map(Some)
}

/// 获取激活文件路径（固定一个位置，简单可靠）
fn license_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("SupportFlow").join("license.json"))
}

fn write_license_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let path = license_file_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let payload = serde_json::json!({ "token": token.trim() });
    fs::write(path, payload.to_string()).map_err(|e| e.to_string())
}

fn read_license_token_file(path: &PathBuf) -> Result<Option<String>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    extract_token_from_license_json(&raw).map(Some)
}

fn extract_token_from_license_json(raw: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    match v {
        // 兼容直接写字符串的最简格式： "xxxxx"
        serde_json::Value::String(s) => Ok(s.trim().to_string()),
        serde_json::Value::Object(map) => {
            // 兼容两种常见字段名
            if let Some(t) = map.get("token").and_then(|x| x.as_str()) {
                return Ok(t.trim().to_string());
            }
            if let Some(t) = map.get("activationToken").and_then(|x| x.as_str()) {
                return Ok(t.trim().to_string());
            }
            Err("license.json 缺少 token 字段".to_string())
        }
        _ => Err("license.json 格式错误".to_string()),
    }
}

fn read_public_key_pem(app: &AppHandle) -> Result<String, String> {
    if let Some(pem) = read_optional_resource_text(app, "public_key.pem")? {
        return Ok(pem);
    }

    // Dev fallback: Tauri dev may not bundle non-whitelisted resources into `BaseDirectory::Resource`.
    // Use the repo file relative to the `src-tauri` crate dir (stable even if CWD differs).
    let dev_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("public_key.pem");
    if dev_path.is_file() {
        return fs::read_to_string(dev_path).map_err(|e| e.to_string());
    }

    Err("missing resources/public_key.pem".to_string())
}

fn read_stored_activation_token(app: &AppHandle) -> Result<Option<String>, String> {
    // 1) 先读本地可写目录
    let path = license_file_path(app)?;
    if let Some(token) = read_license_token_file(&path)? {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(Some(token));
        }
    }

    // 2) 再读 resources 里的只读兜底文件
    if let Some(raw) = read_optional_resource_text(app, "license.json")? {
        let token = extract_token_from_license_json(&raw)?;
        if !token.trim().is_empty() {
            return Ok(Some(token));
        }
    }
    Ok(None)
}

impl LicenseStore {
    fn evaluate(
        machine_code: &str,
        public_key_pem: &str,
        token: Option<&str>,
    ) -> (bool, Option<String>) {
        match token {
            Some(t) if !t.trim().is_empty() => {
                let vr = verify_license_token(
                    t.trim(),
                    now_unix_seconds(),
                    machine_code,
                    public_key_pem,
                );
                (vr.valid, vr.reason)
            }
            _ => {
                if is_dev_license_bypass_enabled() {
                    return (
                        true,
                        Some("dev license bypass enabled (missing activation token)".to_string()),
                    );
                }
                (false, Some("missing activation token".to_string()))
            }
        }
    }

    fn build_status(
        machine_code: String,
        public_key_pem: &str,
        token: Option<String>,
    ) -> LicenseStatus {
        let (valid, reason) = Self::evaluate(&machine_code, public_key_pem, token.as_deref());
        LicenseStatus {
            valid,
            reason,
            machine_code,
        }
    }

    /// Startup: compute machine code once, then verify stored/bundled token if any.
    pub fn initialize(app: &AppHandle) -> Self {
        match Self::try_initialize(app) {
            Ok(store) => store,
            Err(e) => {
                let machine_code = compute_machine_code_windows().unwrap_or_default();
                Self(Mutex::new(LicenseStatus {
                    valid: false,
                    reason: Some(format!("license init failed: {e}")),
                    machine_code,
                }))
            }
        }
    }

    fn try_initialize(app: &AppHandle) -> Result<Self, String> {
        let machine_code = compute_machine_code_windows()?;
        let public_key_pem = read_public_key_pem(app)?;
        let token = read_stored_activation_token(app)?;
        Ok(Self(Mutex::new(Self::build_status(
            machine_code,
            &public_key_pem,
            token,
        ))))
    }

    pub async fn initialize_async(app: &AppHandle) -> Self {
        let handle = app.clone();
        match tokio::task::spawn_blocking(move || Self::initialize(&handle)).await {
            Ok(store) => store,
            Err(e) => Self(Mutex::new(LicenseStatus {
                valid: false,
                reason: Some(format!("license init task failed: {e}")),
                machine_code: String::new(),
            })),
        }
    }

    pub fn snapshot(&self) -> Result<LicenseStatusDto, String> {
        let guard = self.0.lock().map_err(|e| e.to_string())?;
        Ok(LicenseStatusDto {
            machine_code: guard.machine_code.clone(),
            valid: guard.valid,
            reason: guard.reason.clone(),
        })
    }

    /// Verify token, persist to app data, refresh in-memory status.
    pub fn apply_activation_token(
        &self,
        app: &AppHandle,
        token: &str,
    ) -> Result<LicenseStatusDto, String> {
        let token = token.trim();
        if token.is_empty() {
            return Err("activation token is empty".to_string());
        }

        let public_key_pem = read_public_key_pem(app)?;
        let machine_code = {
            let guard = self.0.lock().map_err(|e| e.to_string())?;
            guard.machine_code.clone()
        };

        if machine_code.is_empty() {
            return Err("machine code not ready".to_string());
        }

        let vr = verify_license_token(token, now_unix_seconds(), &machine_code, &public_key_pem);
        if !vr.valid {
            return Err(vr
                .reason
                .unwrap_or_else(|| "invalid activation token".to_string()));
        }

        write_license_token(app, token)?;

        let next = Self::build_status(machine_code, &public_key_pem, Some(token.to_string()));
        let dto = LicenseStatusDto {
            machine_code: next.machine_code.clone(),
            valid: next.valid,
            reason: next.reason.clone(),
        };
        *self.0.lock().map_err(|e| e.to_string())? = next;
        Ok(dto)
    }

    pub fn require_valid(&self) -> Result<(), String> {
        let guard = self.0.lock().map_err(|e| e.to_string())?;
        if guard.valid {
            Ok(())
        } else {
            Err(guard
                .reason
                .clone()
                .map(|r| format!("LICENSE_LOCKED: {r}"))
                .unwrap_or_else(|| "LICENSE_LOCKED".to_string()))
        }
    }
}
