//! 调用 `channel_agent/scripts/markitdown_convert.py` 将本地文件转为 Markdown（单次子进程，非 sidecar）。

use std::path::{Path, PathBuf};
use std::process::Command;

use process_runtime::{run_sync, CommandSpec, ProcessSharedContext};

use crate::python::paths;

const EXIT_NOT_INSTALLED: i32 = 2;

/// 解析用于 MarkItDown 的 Python 可执行文件。
///
/// 优先级：`CHANNEL_MARKITDOWN_PYTHON` → `CHANNEL_PYTHON_EXECUTABLE` → `py -3.x` → `python`。
pub fn resolve_python() -> Option<PathBuf> {
    for key in ["CHANNEL_MARKITDOWN_PYTHON", "CHANNEL_PYTHON_EXECUTABLE"] {
        if let Ok(raw) = std::env::var(key) {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }

    #[cfg(windows)]
    {
        for ver in ["3.13", "3.12", "3.11", "3.10"] {
            if let Ok(out) = Command::new("py")
                .args(["-", ver, "-c", "import sys; print(sys.executable)"])
                .output()
            {
                if out.status.success() {
                    let exe = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !exe.is_empty() {
                        return Some(PathBuf::from(exe));
                    }
                }
            }
        }
    }

    for candidate in ["python3", "python"] {
        if let Ok(out) = Command::new(candidate)
            .args(["-c", "import sys; print(sys.executable)"])
            .output()
        {
            if out.status.success() {
                let exe = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !exe.is_empty() {
                    return Some(PathBuf::from(exe));
                }
            }
        }
    }
    None
}

fn script_path() -> PathBuf {
    if let Ok(raw) = std::env::var("MARKITDOWN_SCRIPT") {
        let p = PathBuf::from(raw.trim());
        if p.is_file() {
            return p;
        }
    }
    paths::markitdown_script()
}

/// 将本地文件转为 Markdown 文本；输出为空时返回空字符串。
pub fn convert_file_to_markdown(path: &Path) -> Result<String, String> {
    let python = resolve_python().ok_or_else(|| {
        "MarkItDown: no Python found (set CHANNEL_MARKITDOWN_PYTHON, Python 3.10+)".to_string()
    })?;
    let script = script_path();
    if !script.is_file() {
        return Err(format!("MarkItDown helper missing: {}", script.display()));
    }

    let spec = CommandSpec::binary("markitdown", python).with_args([
        script.to_string_lossy().to_string(),
        path.display().to_string(),
    ]);

    let output = run_sync(&spec, &ProcessSharedContext::default())
        .map_err(|e| format!("MarkItDown spawn: {e}"))?;

    if output.code == Some(EXIT_NOT_INSTALLED) {
        return Err(
            "markitdown package not installed (pip install 'markitdown[all]', see channel_agent/requirements-markitdown.txt)".into(),
        );
    }
    if !output.success() {
        return Err(format!(
            "MarkItDown failed (exit={:?}): {}",
            output.code,
            output.stderr_lossy().trim()
        ));
    }

    Ok(output.stdout_lossy())
}
