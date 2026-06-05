//! Tauri 侧文件 IO：复用 [`fs_io`]，对外提供 `Result<_, String>` 以匹配 command/context 约定。

use std::path::Path;

pub use fs_io::{
    create_dir_all as create_dir_all_io, read as read_io, read_to_string as read_to_string_io,
    write as write_io,
};

fn map_err<E: std::fmt::Display>(error: E) -> String {
    error.to_string()
}

/// 递归创建目录。
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    create_dir_all_io(path).map_err(map_err)
}

/// 读取文本文件内容。
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    read_to_string_io(path).map_err(map_err)
}

/// 读取二进制文件内容。
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, String> {
    read_io(path).map_err(map_err)
}

/// 写入文件内容。
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), String> {
    write_io(path, contents).map_err(map_err)
}
