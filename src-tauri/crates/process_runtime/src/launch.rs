//! 长驻子进程启动模式。

use crate::spec::CommandSpec;

/// 长驻进程的启动方式（crate 层仅描述 tokio 命令；Tauri shell 由桌面适配层扩展）。
#[derive(Clone)]
pub enum LaunchMode {
    Command(CommandSpec),
}
