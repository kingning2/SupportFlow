//! 通过独立 `license-verifier` 子进程完成订阅校验，避免在主程序源码中暴露验签逻辑。

use std::path::PathBuf;

use process_runtime::{run_sync, CommandSpec, ProcessSharedContext};
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
    if let Some(path) = process_runtime::resolve_exe_from_env("LICENSE_VERIFIER_EXE")? {
        return Ok(path);
    }

    let path = process_runtime::binary_in_dir(
        &crate::utils::path::crate_path("binaries"),
        "license-verifier",
        env!("BUILD_TARGET"),
    );
    if path.is_file() {
        return Ok(path);
    }

    Err(verifier_missing_message())
}

fn verifier_spec(exe: &PathBuf, args: Vec<&str>) -> CommandSpec {
    CommandSpec::binary("license-verifier", exe.clone()).with_args(args)
}

fn run_verifier(exe: &PathBuf, args: Vec<&str>) -> Result<(i32, String, String), String> {
    let output = run_sync(&verifier_spec(exe, args), &ProcessSharedContext::default())?;
    Ok((
        output.code.unwrap_or(2),
        output.stdout_lossy(),
        output.stderr_lossy(),
    ))
}

/// 调用子进程计算本机 machineCode。
pub fn machine_code_via_exe() -> Result<String, String> {
    let exe = resolve_verifier_exe()?;
    let (code, stdout, stderr) = run_verifier(&exe, vec!["gen-machine-code"])?;
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
    let (code, stdout, stderr) = run_verifier(&exe, vec!["verify", "--token", token])?;

    if code == 2 {
        return Err(if stderr.is_empty() {
            "license-verifier verify failed (exit=2)".to_string()
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
