//! Python 互操作层：路径、MarkItDown 子进程、渠道 sidecar RPC。
//!
//! 结构约定见仓库根目录 [`docs/rust-folder-structure.md`](../../../docs/rust-folder-structure.md) → **`src/python/`**。
//!
//! ```text
//! python/
//!   paths.rs           — 无 Tauri 的 channel_agent / 产物路径
//!   paths_desktop.rs   — AppHandle + Resource 解析（desktop）
//!   markitdown.rs      — 单次子进程：markitdown_convert.py
//!   client.rs          — sidecar RPC 薄封装（desktop）
//!   sidecar/           — wx/wework 长驻 sidecar（desktop）
//! ```

mod paths;

mod markitdown;

#[cfg(feature = "desktop")]
mod paths_desktop;
#[cfg(feature = "desktop")]
mod sidecar;

pub use paths::{
    bundled_markitdown_resource_key, channel_agent_root, dev_channel_source_dir, markitdown_script,
    repo_root, sidecar_binary_in_binaries, tauri_manifest_dir,
};

pub use markitdown::{convert_file_to_markdown, resolve_python as resolve_markitdown_python};
#[cfg(feature = "desktop")]
pub use paths_desktop::resolve_markitdown_script;
#[cfg(feature = "desktop")]
pub use sidecar::{spawn_sidecar, ChannelPythonSidecar};
