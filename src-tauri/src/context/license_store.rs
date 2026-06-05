//! Shared subscription license status across all Webviews.
//!
//! Machine code and token verification are delegated to the external
//! `license-verifier` executable so verification logic stays out of the main binary.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use typeshare::typeshare;

use crate::utils::license_key::{decode_token_from_key_bytes, encode_token_to_key_bytes};
use crate::utils::license_verifier_exe::{machine_code_via_exe, verify_token_via_exe};

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

fn is_dev_license_bypass_enabled() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    // Enable bypass for local single-channel dev launcher by default.
    // Can be disabled with SUPPORT_FLOW_DEV_BYPASS_LICENSE=0/false/off.
    let has_dev_channel = crate::utils::env::get("DEV_CHANNEL")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    if !has_dev_channel {
        return false;
    }
    crate::utils::env::get("SUPPORT_FLOW_DEV_BYPASS_LICENSE")
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "off")
        })
        .unwrap_or(true)
}

fn read_optional_resource_text(app: &AppHandle, filename: &str) -> Result<Option<String>, String> {
    let path = app
        .path()
        .resolve(filename, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Ok(None);
    }
    crate::utils::fs::read_to_string(path).map(Some)
}

fn read_optional_resource_bytes(
    app: &AppHandle,
    filename: &str,
) -> Result<Option<Vec<u8>>, String> {
    let path = app
        .path()
        .resolve(filename, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Ok(None);
    }
    crate::utils::fs::read(path).map(Some)
}

/// 获取激活 key 文件路径（固定一个位置，简单可靠）
fn license_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("SupportFlow").join("license.key"))
}

fn legacy_license_json_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("SupportFlow").join("license.json"))
}

fn write_license_token(app: &AppHandle, token: &str) -> Result<(), String> {
    let path = license_file_path(app)?;
    if let Some(parent) = path.parent() {
        crate::utils::fs::create_dir_all(parent)?;
    }
    let key_bytes = encode_token_to_key_bytes(token)?;
    crate::utils::fs::write(path, key_bytes)
}

fn read_license_key_file(path: &PathBuf) -> Result<Option<String>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = crate::utils::fs::read(path)?;
    decode_token_from_key_bytes(&bytes).map(Some)
}

fn read_license_token_file(path: &PathBuf) -> Result<Option<String>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = crate::utils::fs::read_to_string(path)?;
    extract_token_from_license_json(&raw).map(Some)
}

fn extract_token_from_license_json(raw: &str) -> Result<String, String> {
    let v: serde_json::Value = crate::utils::json::from_str(raw)?;
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

fn read_stored_activation_token(app: &AppHandle) -> Result<Option<String>, String> {
    // 1) 先读本地可写目录的二进制 key 文件
    let path = license_file_path(app)?;
    if let Some(token) = read_license_key_file(&path)? {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(Some(token));
        }
    }

    // 2) 兼容旧版本地 JSON token 文件
    let legacy_path = legacy_license_json_path(app)?;
    if let Some(token) = read_license_token_file(&legacy_path)? {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(Some(token));
        }
    }

    // 3) 再读 resources 里的只读兜底文件
    if let Some(bytes) = read_optional_resource_bytes(app, "license.key")? {
        let token = decode_token_from_key_bytes(&bytes)?;
        if !token.trim().is_empty() {
            return Ok(Some(token));
        }
    }

    // 4) 兼容 resources 里的旧 license.json
    if let Some(raw) = read_optional_resource_text(app, "license.json")? {
        let token = extract_token_from_license_json(&raw)?;
        if !token.trim().is_empty() {
            return Ok(Some(token));
        }
    }
    Ok(None)
}

impl LicenseStore {
    fn evaluate(token: Option<&str>) -> (bool, Option<String>) {
        match token {
            Some(t) if !t.trim().is_empty() => match verify_token_via_exe(t.trim()) {
                Ok(vr) => (vr.valid, vr.reason),
                Err(e) => (false, Some(e)),
            },
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

    fn build_status(machine_code: String, token: Option<String>) -> LicenseStatus {
        let (valid, reason) = Self::evaluate(token.as_deref());
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
                let machine_code = machine_code_via_exe().unwrap_or_default();
                Self(Mutex::new(LicenseStatus {
                    valid: false,
                    reason: Some(format!("license init failed: {e}")),
                    machine_code,
                }))
            }
        }
    }

    fn try_initialize(app: &AppHandle) -> Result<Self, String> {
        let machine_code = machine_code_via_exe()?;
        let token = read_stored_activation_token(app)?;
        Ok(Self(Mutex::new(Self::build_status(machine_code, token))))
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
        let guard = crate::utils::err::lock_mutex(&self.0)?;
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

        let machine_code = {
            let guard = crate::utils::err::lock_mutex(&self.0)?;
            guard.machine_code.clone()
        };

        if machine_code.is_empty() {
            return Err("machine code not ready".to_string());
        }

        let vr = verify_token_via_exe(token)?;
        if !vr.valid {
            return Err(vr
                .reason
                .unwrap_or_else(|| "invalid activation token".to_string()));
        }

        write_license_token(app, token)?;

        let next = Self::build_status(machine_code, Some(token.to_string()));
        let dto = LicenseStatusDto {
            machine_code: next.machine_code.clone(),
            valid: next.valid,
            reason: next.reason.clone(),
        };
        *crate::utils::err::lock_mutex(&self.0)? = next;
        Ok(dto)
    }

    pub fn require_valid(&self) -> Result<(), String> {
        let guard = crate::utils::err::lock_mutex(&self.0)?;
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
