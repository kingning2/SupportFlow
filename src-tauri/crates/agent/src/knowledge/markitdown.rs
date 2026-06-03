//! Microsoft MarkItDown — upload / document parse → Markdown before chunking.

use std::path::{Path, PathBuf};
use std::process::Command;

const EXIT_NOT_INSTALLED: i32 = 2;

fn markitdown_script_path() -> PathBuf {
    // 1. Explicit override (set by Tauri runtime at startup from bundled resource or dev tree)
    if let Ok(raw) = std::env::var("MARKITDOWN_SCRIPT") {
        let p = PathBuf::from(raw.trim());
        if p.is_file() {
            return p;
        }
    }

    // 2. Dev fallback: baked compile-time path from agent crate (works inside source checkout)
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../channel_agent/scripts/markitdown_convert.py");
    if dev.is_file() {
        return dev;
    }

    // 3. Last resort: assume cwd layout
    std::env::current_dir()
        .unwrap_or_default()
        .join("src-tauri/channel_agent/scripts/markitdown_convert.py")
}

/// Python 3.10+ with `pip install 'markitdown[all]'` (see `channel_agent/requirements-markitdown.txt`).
pub fn resolve_markitdown_python() -> Option<PathBuf> {
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

/// Convert a local file to Markdown text via MarkItDown. Returns empty string if output is blank.
pub fn convert_file_to_markdown(path: &Path) -> Result<String, String> {
    let python = resolve_markitdown_python().ok_or_else(|| {
        "MarkItDown: no Python found (set CHANNEL_MARKITDOWN_PYTHON, Python 3.10+)".to_string()
    })?;
    let script = markitdown_script_path();
    if !script.is_file() {
        return Err(format!("MarkItDown helper missing: {}", script.display()));
    }

    let output = Command::new(&python)
        .arg(&script)
        .arg(path)
        .output()
        .map_err(|e| format!("MarkItDown spawn {}: {e}", python.display()))?;

    if output.status.code() == Some(EXIT_NOT_INSTALLED) {
        return Err(
            "markitdown package not installed (pip install 'markitdown[all]', Python 3.10+)".into(),
        );
    }
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "MarkItDown failed ({}): {}",
            output.status,
            stderr.trim()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(text)
}
