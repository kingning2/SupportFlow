//! 子进程 stdio 管道配置。

use std::process::Stdio;

use tokio::process::Command;

pub fn piped_stdio(cmd: &mut Command) {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_windows_background_flags(cmd);
}

/// Windows：子进程不弹出控制台窗口。
#[cfg(windows)]
pub fn apply_windows_background_flags(cmd: &mut Command) {
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub fn apply_windows_background_flags(_cmd: &mut Command) {}
