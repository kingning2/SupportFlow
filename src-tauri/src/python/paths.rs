//! 仓库根目录下的 `channel_agent/` 路径解析（Python 源码与 sidecar 产物）。

use std::path::PathBuf;

/// `src-tauri` 的 Cargo manifest 目录。
pub fn tauri_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// monorepo 根目录（`src-tauri` 的上一级）。
pub fn repo_root() -> PathBuf {
    tauri_manifest_dir().join("..")
}

/// 根目录 `channel_agent/`（Python SDK + sidecar 源码）。
pub fn channel_agent_root() -> PathBuf {
    repo_root().join("channel_agent")
}

/// MarkItDown 转换脚本（`channel_agent/scripts/markitdown_convert.py`）。
pub fn markitdown_script() -> PathBuf {
    channel_agent_root()
        .join("scripts")
        .join("markitdown_convert.py")
}

/// 开发模式：`python -m channel` 的工作目录（含 `channel/__main__.py`）。
pub fn dev_channel_source_dir() -> Option<PathBuf> {
    let root = channel_agent_root();
    if root.join("channel").join("__main__.py").is_file() {
        root.canonicalize().ok()
    } else {
        None
    }
}

/// PyInstaller 产物：`src-tauri/binaries/channel-sidecar-{target}.exe`。
pub fn sidecar_binary_in_binaries() -> PathBuf {
    let name = format!(
        "channel-sidecar-{}{}",
        env!("BUILD_TARGET"),
        std::env::consts::EXE_SUFFIX
    );
    tauri_manifest_dir().join("binaries").join(name)
}

/// Tauri 打包资源中的 MarkItDown 脚本（相对 `src-tauri` manifest）。
pub fn bundled_markitdown_resource_key() -> &'static str {
    "../channel_agent/scripts/markitdown_convert.py"
}
