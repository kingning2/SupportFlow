//! 依赖 Tauri `AppHandle` 的 Python 相关路径解析（仅 desktop）。

use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::python::paths;

/// 从 Tauri Resource 或开发树 `channel_agent/` 解析 MarkItDown 脚本绝对路径。
///
/// # 参数
///
/// * `app` - 当前 Tauri 应用句柄
///
/// # 返回
///
/// * `PathBuf` - `markitdown_convert.py` 的绝对路径
pub fn resolve_markitdown_script(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(p) = app.path().resolve(
        paths::bundled_markitdown_resource_key(),
        tauri::path::BaseDirectory::Resource,
    ) {
        if p.is_file() {
            return Ok(p);
        }
    }
    let dev = paths::markitdown_script();
    if dev.is_file() {
        return Ok(dev);
    }
    Err("markitdown_convert.py not found in resources or channel_agent/".into())
}
