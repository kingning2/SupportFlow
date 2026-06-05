//! Windows-only Python launch helpers.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command as StdCommand;

fn channel_python_from_env() -> Option<String> {
    if let Some(exe) = crate::utils::env::get("CHANNEL_PYTHON_EXECUTABLE") {
        let trimmed = exe.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    option_env!("CHANNEL_PYTHON_EXECUTABLE").and_then(|exe| {
        let trimmed = exe.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn python_executable_from_launcher(launcher: &str, args: &[&str]) -> Option<String> {
    let mut cmd = StdCommand::new(launcher);
    cmd.args(args)
        .args(["-c", "import sys; print(sys.executable)"]);
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let exe = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if exe.is_empty() || !Path::new(&exe).is_file() {
        return None;
    }

    Some(exe)
}

/// Resolve the preferred Python executable for Windows sidecar launch.
///
/// # Returns
///
/// * `String` - Executable name or absolute path suitable for launching Python
pub fn resolve_python_executable() -> String {
    if let Some(exe) = channel_python_from_env() {
        return exe;
    }

    if let Some(exe) = python_executable_from_launcher("py", &["-3.10"]) {
        return exe;
    }

    "python".to_string()
}
