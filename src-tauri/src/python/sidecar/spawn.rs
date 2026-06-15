//! 渠道 sidecar 启动模式解析与 spawn 入口。

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use super::handler::ChannelInboundHandler;
use super::ChannelPythonSidecar;
use crate::process_runtime::{
    resolve_exe_from_env, CommandSpec, LaunchMode, ProcessSharedContext, StdioJsonRpcConfig,
    StdioJsonRpcRuntime,
};
use serde_json::json;
use tauri::AppHandle;

/// 通道 sidecar 的启动方式。
#[derive(Clone)]
pub(crate) enum SpawnMode {
    TauriSidecar,
    PythonSource { dir: PathBuf },
    CustomExe { path: PathBuf },
}

pub(crate) fn sidecar_missing_message() -> String {
    format!(
        "未找到通道 sidecar 可执行文件。请先运行: pnpm run build:channel-sidecar\n\
         产物路径: src-tauri/binaries/channel-sidecar-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    )
}

pub(crate) fn resolve_spawn_mode(app: &AppHandle) -> Result<SpawnMode, String> {
    if let Some(path) = resolve_exe_from_env("CHANNEL_SIDECAR_EXE")? {
        return Ok(SpawnMode::CustomExe { path });
    }

    #[cfg(debug_assertions)]
    if let Some(dev) = crate::python::paths::dev_channel_source_dir() {
        return Ok(SpawnMode::PythonSource { dir: dev });
    }

    if sidecar_binary_exists() {
        return Ok(SpawnMode::TauriSidecar);
    }

    if let Some(dev) = crate::python::paths::dev_channel_source_dir() {
        return Ok(SpawnMode::PythonSource { dir: dev });
    }

    let _ = app;
    Err(sidecar_missing_message())
}

fn sidecar_binary_exists() -> bool {
    crate::python::paths::sidecar_binary_in_binaries().is_file()
}

pub(crate) fn channel_env_pairs(shared: &ProcessSharedContext) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = shared
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    pairs.push(("PYTHONUNBUFFERED".into(), "1".into()));
    pairs.push(("PYTHONIOENCODING".into(), "utf-8".into()));
    pairs.push(("TAURI_CHANNEL_MODE".into(), "1".into()));
    if let Some(dev) = crate::utils::env::get("DEV_CHANNEL") {
        let trimmed = dev.trim();
        if !trimmed.is_empty() {
            pairs.push(("DEV_CHANNEL".into(), trimmed.to_string()));
        }
    }
    pairs
}

pub(crate) fn command_spec_for_mode(
    mode: &SpawnMode,
    shared: &ProcessSharedContext,
) -> Result<CommandSpec, String> {
    let base_env = |spec: CommandSpec| {
        let pairs = channel_env_pairs(shared);
        pairs.into_iter().fold(spec, |s, (k, v)| s.with_env(k, v))
    };

    match mode {
        SpawnMode::PythonSource { dir } => {
            let python = crate::utils::platform::python::resolve_python_executable();
            Ok(base_env(
                CommandSpec::binary("channel", python)
                    .with_args(["-m", "channel"])
                    .with_cwd(dir.clone())
                    .piped(),
            ))
        }
        SpawnMode::CustomExe { path } => Ok(base_env(
            CommandSpec::binary("channel", path.clone()).piped(),
        )),
        SpawnMode::TauriSidecar => Err("tauri sidecar uses shell adapter".into()),
    }
}

/// 启动渠道 sidecar 并完成首次 ping。
pub async fn spawn_sidecar(
    app: &AppHandle,
    shared: &ProcessSharedContext,
) -> Result<Arc<ChannelPythonSidecar>, String> {
    let mode = resolve_spawn_mode(app)?;
    let sidecar = ChannelPythonSidecar::new(app.clone(), shared.clone());
    sidecar.ensure_running_with_mode(mode).await?;
    Ok(sidecar)
}

pub(crate) fn build_runtime(
    shared: ProcessSharedContext,
    handler: Arc<ChannelInboundHandler>,
) -> Arc<StdioJsonRpcRuntime> {
    let python = crate::utils::platform::python::resolve_python_executable();
    let spec = CommandSpec::binary("channel", python).piped();
    StdioJsonRpcRuntime::new(
        StdioJsonRpcConfig {
            name: "channel",
            log_prefix: "channel",
            rpc_timeout: Duration::from_secs(30),
            health_check: None,
        },
        shared,
        spec,
        handler,
    )
}

pub(crate) async fn run_health_check(runtime: &Arc<StdioJsonRpcRuntime>) {
    match tokio::time::timeout(Duration::from_secs(15), runtime.rpc("ping", json!({}))).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            crate::log_warn!("Channel sidecar ping failed: {e}");
        }
        Err(_) => {
            crate::log_warn!(
                "Channel sidecar ping timed out after 15s (process may still be starting)"
            );
        }
    }
}

pub(crate) fn launch_mode_from_spawn(
    mode: &SpawnMode,
    shared: &ProcessSharedContext,
) -> Result<LaunchMode, String> {
    Ok(LaunchMode::Command(command_spec_for_mode(mode, shared)?))
}
