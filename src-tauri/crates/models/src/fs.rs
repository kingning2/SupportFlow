//! Models 侧文件 IO：复用 [`fs_io`]，对外提供 `Result<_, String>`。

use std::path::Path;

fn map_err(error: std::io::Error) -> String {
    error.to_string()
}

/// 递归创建目录。
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), String> {
    fs_io::create_dir_all(path).map_err(map_err)
}

/// 读取文本文件内容。
pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, String> {
    fs_io::read_to_string(path).map_err(map_err)
}

/// 写入文件内容。
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), String> {
    fs_io::write(path, contents).map_err(map_err)
}
