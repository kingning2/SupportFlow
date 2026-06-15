use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use clap::Args;

use crate::cli::paths;

#[derive(Args)]
pub struct StartArgs {
    /// Run in foreground (don't daemonize)
    #[arg(short, long)]
    pub foreground: bool,
    /// Don't tail logs after starting
    #[arg(long)]
    pub no_logs: bool,
}

pub fn cmd_start(args: StartArgs) -> Result<()> {
    let workspace = paths::resolve_workspace()?;
    if let Some(pid) = read_pid(&workspace)? {
        println!("SupportFlow is already running (PID: {pid}).");
        return Ok(());
    }

    let exe = find_desktop_exe().ok_or_else(|| {
        anyhow!(
            "desktop app binary not found. Build with `pnpm run tauri build`, set {env}, or run `pnpm run tauri:dev`",
            env = paths::ENV_DESKTOP_APP
        )
    })?;

    let log_file = paths::log_path(&workspace);

    if args.foreground {
        println!("Starting SupportFlow in foreground...");
        let status = Command::new(&exe)
            .current_dir(exe.parent().unwrap_or(std::path::Path::new(".")))
            .status()
            .with_context(|| format!("run {}", exe.display()))?;
        if !status.success() {
            return Err(anyhow!("process exited with {status}"));
        }
        return Ok(());
    }

    println!("Starting SupportFlow...");
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    #[cfg(windows)]
    let child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new(&exe)
            .stdout(Stdio::from(log.try_clone()?))
            .stderr(Stdio::from(log))
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
    };

    #[cfg(not(windows))]
    let child = Command::new(&exe)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn();

    let child = child.with_context(|| format!("spawn {}", exe.display()))?;
    write_pid(&workspace, child.id())?;
    println!("✓ SupportFlow started (PID: {})", child.id());
    println!("  Logs: {}", log_file.display());

    if !args.no_logs {
        println!("  Last log lines:\n");
        tail_log(&log_file, 40)?;
    }
    Ok(())
}

pub fn cmd_stop() -> Result<()> {
    let workspace = paths::resolve_workspace()?;
    let Some(pid) = read_pid(&workspace)? else {
        println!("SupportFlow is not running.");
        return Ok(());
    };
    println!("Stopping SupportFlow (PID: {pid})...");
    kill_pid(pid)?;
    remove_pid(&workspace)?;
    println!("✓ Stopped.");
    Ok(())
}

pub fn cmd_restart() -> Result<()> {
    cmd_stop()?;
    cmd_start(StartArgs {
        foreground: false,
        no_logs: false,
    })
}

pub fn cmd_status() -> Result<()> {
    let workspace = paths::resolve_workspace()?;
    match read_pid(&workspace)? {
        Some(pid) => println!("SupportFlow is running (PID: {pid})."),
        None => println!("SupportFlow is not running."),
    }
    println!("  Workspace: {}", workspace.display());
    Ok(())
}

pub fn cmd_logs(lines: usize) -> Result<()> {
    let workspace = paths::resolve_workspace()?;
    let log_file = paths::log_path(&workspace);
    if !log_file.is_file() {
        println!("No log file at {}", log_file.display());
        return Ok(());
    }
    tail_log(&log_file, lines)?;
    Ok(())
}

pub fn cmd_update() -> Result<()> {
    println!("Update the desktop app via git pull + rebuild, or your OS installer.");
    println!("  pnpm run tauri build");
    Ok(())
}

fn find_desktop_exe() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var(paths::ENV_DESKTOP_APP) {
        let p = PathBuf::from(raw.trim());
        if p.is_file() {
            return Some(p);
        }
    }
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/tauri-app.exe"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/tauri-app"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/tauri-app.exe"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/tauri-app"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn pid_file(workspace: &std::path::Path) -> PathBuf {
    paths::pid_path(workspace)
}

fn read_pid(workspace: &std::path::Path) -> Result<Option<u32>> {
    let path = pid_file(workspace);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = crate::io::read_to_string(&path).context("read pid file")?;
    let pid: u32 = raw.trim().parse().context("parse pid")?;
    if is_pid_alive(pid) {
        return Ok(Some(pid));
    }
    let _ = crate::io::remove_file(&path);
    Ok(None)
}

fn write_pid(workspace: &std::path::Path, pid: u32) -> Result<()> {
    crate::io::write(pid_file(workspace), pid.to_string())?;
    Ok(())
}

fn remove_pid(workspace: &std::path::Path) -> Result<()> {
    let path = pid_file(workspace);
    if path.is_file() {
        crate::io::remove_file(path)?;
    }
    Ok(())
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let out = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .creation_flags(0x0800_0000)
            .output();
        match out {
            Ok(o) => String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()),
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn kill_pid(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()?;
        if !status.success() {
            return Err(anyhow!("taskkill failed for pid {pid}"));
        }
    }
    #[cfg(not(windows))]
    {
        let status = Command::new("kill").arg(pid.to_string()).status()?;
        if !status.success() {
            return Err(anyhow!("kill failed for pid {pid}"));
        }
    }
    Ok(())
}

fn tail_log(path: &std::path::Path, max_lines: usize) -> Result<()> {
    let mut file = std::fs::File::open(path)?;
    if max_lines > 0 {
        let content = crate::io::read_to_string(path)?;
        let lines: Vec<_> = content.lines().collect();
        let start = lines.len().saturating_sub(max_lines);
        for line in &lines[start..] {
            println!("{line}");
        }
        return Ok(());
    }
    file.seek(SeekFrom::End(0))?;
    let mut buf = [0u8; 4096];
    loop {
        let n = file.read(&mut buf)?;
        if n > 0 {
            print!("{}", String::from_utf8_lossy(&buf[..n]));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        file.seek(SeekFrom::End(0))?;
    }
}
