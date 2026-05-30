use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{mpsc::Sender, OnceLock},
    thread,
};

use crate::utils::date;

#[macro_export]
macro_rules! log_error { ($($arg:tt)+) => { $crate::utils::log::log("ERROR", &format!($($arg)+)) }; }

#[macro_export]
macro_rules! log_warn  { ($($arg:tt)+) => { $crate::utils::log::log("WARN",  &format!($($arg)+)) }; }

#[macro_export]
macro_rules! log_info  { ($($arg:tt)+) => { $crate::utils::log::log("INFO",  &format!($($arg)+)) }; }

#[macro_export]
macro_rules! log_debug { ($($arg:tt)+) => { $crate::utils::log::log("DEBUG", &format!($($arg)+)) }; }

static LOG_TX: OnceLock<Sender<String>> = OnceLock::new();

fn start_log_writer(log_path: &PathBuf) -> Result<(), String> {
    if LOG_TX.get().is_some() {
        return Ok(());
    }

    if let Some(parent) = log_path.parent() {
        create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let log_file_path = format!(
        "{}.{}.log",
        log_path.to_string_lossy(),
        date::current_date_string()
    );

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(&log_file_path)
        .map_err(|e| e.to_string())?;

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    thread::Builder::new()
        .name("tauri-app-log".to_string())
        .spawn(move || {
            let mut file = file;
            while let Ok(line) = rx.recv() {
                let _ = file.write_all(line.as_bytes());
                let _ = file.flush();
            }
        })
        .map_err(|e| e.to_string())?;

    LOG_TX
        .set(tx)
        .map_err(|_| "log service already initialized".to_string())?;
    log("INFO", "===== Successfully started the log service =====");
    Ok(())
}

pub fn log(level: &str, msg: &str) {
    let datetime = date::current_datetime_string();
    let line = format!("[{}] [{}] {}\n\n", datetime, level, msg);

    print!("{}", line);

    if let Some(tx) = LOG_TX.get() {
        let _ = tx.send(line);
    }
}

pub fn init_log() -> Result<(), String> {
    let dirs = directories::ProjectDirs::from("com", "polymerization", "gybte")
        .ok_or_else(|| "could not resolve app log directory".to_string())?;

    let log_root = dirs.data_local_dir().join("logs");
    let log_path = log_root.join("tauri-app");

    trace_result("utils.log", "init_log", start_log_writer(&log_path))
}

/// 记录 `Result` 的成功或失败后原样返回，供各模块在出口统一包一层日志。
#[inline]
pub fn trace_result<T>(scope: &str, op: &str, result: Result<T, String>) -> Result<T, String> {
    match &result {
        Ok(_) => log("INFO", &format!("{scope}.{op} ok")),
        Err(err) => log("ERROR", &format!("{scope}.{op} err={err}")),
    }
    result
}

/// 同步闭包返回的 `Result` 经 [`trace_result`] 记录后返回。
#[inline]
pub fn trace_result_fn<T>(
    scope: &str,
    op: &str,
    f: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    trace_result(scope, op, f())
}

/// 异步 Future 产出的 `Result` 经 [`trace_result`] 记录后返回。
#[inline]
pub async fn trace_result_async<T>(
    scope: &str,
    op: &str,
    fut: impl std::future::Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let result = fut.await;
    trace_result(scope, op, result)
}
