//! 通过独立 `license-verifier` 子进程完成订阅校验，避免在主程序源码中暴露验签逻辑。

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

/// 子进程校验结果（与 license-verifier stdout JSON 对齐）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseVerifierOutput {
    pub valid: bool,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default, rename = "localMachineCode")]
    #[allow(dead_code)]
    pub local_machine_code: Option<String>,
}

fn verifier_missing_message() -> String {
    format!(
        "未找到 license-verifier 可执行文件。请先运行: pnpm run build:license-verifier\n\
         产物路径: src-tauri/binaries/license-verifier-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    )
}

/// 解析 license-verifier 可执行文件路径。
///
/// 优先级：`LICENSE_VERIFIER_EXE` 环境变量 → `src-tauri/binaries/license-verifier-{target}.exe`
pub fn resolve_verifier_exe() -> Result<PathBuf, String> {
    if let Some(raw) = crate::utils::env::get("LICENSE_VERIFIER_EXE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_file() {
                return Ok(path);
            }
            return Err(format!(
                "LICENSE_VERIFIER_EXE points to missing file: {}",
                path.display()
            ));
        }
    }

    let manifest = crate::utils::path::crate_path("");
    let name = format!(
        "license-verifier-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    );
    let path = manifest.join("binaries").join(name);
    if path.is_file() {
        return Ok(path);
    }

    Err(verifier_missing_message())
}

fn run_verifier<I, S>(exe: &Path, args: I) -> Result<(i32, String, String), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let output = Command::new(exe)
        .args(args.into_iter().map(|s| s.as_ref().to_string()))
        .output()
        .map_err(|e| format!("spawn license-verifier failed: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let code = output.status.code().unwrap_or(2);
    Ok((code, stdout, stderr))
}

/// 调用子进程计算本机 machineCode。
///
/// # Returns
///
/// * `String` - 十六进制 machineCode
pub fn machine_code_via_exe() -> Result<String, String> {
    let exe = resolve_verifier_exe()?;
    let (code, stdout, stderr) = run_verifier(&exe, ["gen-machine-code"])?;
    if code != 0 {
        return Err(if stderr.is_empty() {
            format!("license-verifier gen-machine-code failed (exit={code})")
        } else {
            format!("license-verifier gen-machine-code failed: {stderr}")
        });
    }
    let machine_code = stdout.trim().to_string();
    if machine_code.is_empty() {
        return Err("license-verifier returned empty machineCode".to_string());
    }
    Ok(machine_code)
}

/// 调用子进程校验激活 token（含 RSA 验签、机器码、过期时间）。
///
/// # 参数
///
/// * `token` - base64url 编码的激活 token
///
/// # Returns
///
/// * `LicenseVerifierOutput` - 校验结果；`valid == false` 时 `reason` 说明原因
pub fn verify_token_via_exe(token: &str) -> Result<LicenseVerifierOutput, String> {
    let token = token.trim();
    if token.is_empty() {
        return Ok(LicenseVerifierOutput {
            valid: false,
            reason: Some("activation token is empty".to_string()),
            local_machine_code: None,
        });
    }

    let exe = resolve_verifier_exe()?;
    let (code, stdout, stderr) = run_verifier(&exe, ["verify", "--token", token])?;

    if code == 2 {
        return Err(if stderr.is_empty() {
            format!("license-verifier verify failed (exit=2)")
        } else {
            format!("license-verifier verify failed: {stderr}")
        });
    }

    if stdout.is_empty() {
        return Err("license-verifier verify returned empty stdout".to_string());
    }

    serde_json::from_str::<LicenseVerifierOutput>(&stdout)
        .map_err(|e| format!("parse license-verifier output failed: {e}; stdout={stdout}"))
}
