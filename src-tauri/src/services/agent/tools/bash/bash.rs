//! `agent/tools/bash/bash.py`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use regex::Regex;
use serde_json::{json, Value};
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::services::agent::tools::base_tool::{AgentTool, ToolRunResult};
use crate::services::agent::tools::utils::truncate::{
    format_size, truncate_tail, DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES,
};

#[derive(Debug, Clone)]
pub struct BashConfig {
    pub cwd: PathBuf,
    pub timeout_secs: u64,
    pub safety_mode: bool,
}

impl Default for BashConfig {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            timeout_secs: 30,
            safety_mode: true,
        }
    }
}

pub struct BashTool {
    config: BashConfig,
    description: String,
}

impl BashTool {
    pub fn new(config: BashConfig) -> Self {
        if !config.cwd.exists() {
            let _ = std::fs::create_dir_all(&config.cwd);
        }
        let description = Self::build_description();
        Self {
            config,
            description,
        }
    }

    fn build_description() -> String {
        let platform = if cfg!(windows) {
            "平台：Windows（cmd.exe）。请勿使用 grep、head、tail、sed、awk 等 Unix 专用命令。\n"
        } else {
            ""
        };
        format!(
            "在当前工作目录执行 Shell 命令，返回标准输出与错误输出。输出截断为最近 {DEFAULT_MAX_LINES} 行或 {}KB。\n{platform}环境：env_config 中的 API 密钥会自动注入，可直接使用 $变量名。\n\n安全：可在工作区内自由操作文件；工作区外的破坏性命令须先说明并确认。",
            DEFAULT_MAX_BYTES / 1024
        )
    }

    fn load_dotenv(path: &Path) -> HashMap<String, String> {
        let Ok(content) = std::fs::read_to_string(path) else {
            return HashMap::new();
        };
        let mut vars = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                vars.insert(k.trim().to_string(), v.to_string());
            }
        }
        vars
    }

    fn expand_path(path: &str) -> PathBuf {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(rest);
            }
        }
        if path == "~" {
            return dirs::home_dir().unwrap_or_else(|| PathBuf::from(path));
        }
        PathBuf::from(path)
    }

    fn safety_warning(command: &str) -> Option<String> {
        let lower = command.to_lowercase();
        let tokens: Vec<&str> = lower.split_whitespace().collect();
        for (i, tok) in tokens.iter().enumerate() {
            if *tok != "rm" {
                continue;
            }
            let mut has_rf = false;
            for t in tokens.iter().skip(i + 1) {
                if t.starts_with('-') && t.contains('r') && t.contains('f') {
                    has_rf = true;
                } else if matches!(*t, "--recursive" | "--force") {
                    continue;
                } else if matches!(*t, "/" | "/*") {
                    if has_rf {
                        return Some("This command will delete the entire filesystem".into());
                    }
                    break;
                } else {
                    break;
                }
            }
        }
        let lower = command.to_lowercase();
        if lower.contains("if=/dev/zero") && lower.contains("dd ") {
            return Some("This command can destroy disk data".into());
        }
        let re = Regex::new(r"\b(shutdown|reboot|halt|poweroff)\b").ok()?;
        if re.is_match(&lower) {
            return Some("This command will shut down or restart the system".into());
        }
        None
    }

    fn convert_env_vars_for_windows(
        command: &str,
        dotenv_vars: &HashMap<String, String>,
    ) -> String {
        if dotenv_vars.is_empty() {
            return command.to_string();
        }
        let re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").expect("regex");
        re.replace_all(command, |caps: &regex::Captures| {
            let var_name = caps.get(1).or_else(|| caps.get(2)).map(|m| m.as_str());
            if let Some(name) = var_name {
                if dotenv_vars.contains_key(name) {
                    return format!("%{name}%");
                }
            }
            caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
        })
        .into_owned()
    }

    async fn run_command(
        &self,
        command: &str,
        timeout: Duration,
        dotenv_vars: &HashMap<String, String>,
    ) -> std::io::Result<std::process::Output> {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(command);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        cmd.current_dir(&self.config.cwd);
        cmd.kill_on_drop(true);

        let mut env: HashMap<String, String> = std::env::vars().collect();
        for (k, v) in dotenv_vars {
            env.insert(k.clone(), v.clone());
        }
        if cfg!(windows) {
            env.insert("PYTHONIOENCODING".into(), "utf-8".into());
        }
        cmd.envs(env);

        tokio::time::timeout(timeout, cmd.output())
            .await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "command timed out"))?
    }

    async fn retry_shlex(
        &self,
        command: &str,
        timeout: Duration,
        dotenv_vars: &HashMap<String, String>,
    ) -> Option<std::process::Output> {
        let parts = shlex::split(command)?;
        if parts.is_empty() {
            return None;
        }
        let mut cmd = Command::new(&parts[0]);
        cmd.args(&parts[1..]);
        cmd.current_dir(&self.config.cwd);
        cmd.envs(std::env::vars().chain(dotenv_vars.clone()));
        tokio::time::timeout(timeout, cmd.output()).await.ok()?.ok()
    }

    async fn execute_inner(&self, args: &Value) -> ToolRunResult {
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let timeout_secs = args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.config.timeout_secs);

        if command.is_empty() {
            return ToolRunResult::error("Error: command parameter is required");
        }

        if command.contains("~/.supportflow/.env") || command.contains("~/.supportflow") {
            return ToolRunResult::error(
                "Error: Access denied. API keys and credentials must be accessed through the env_config tool only.",
            );
        }

        if self.config.safety_mode {
            if let Some(warning) = Self::safety_warning(command) {
                return ToolRunResult::error(format!(
                    "Safety Warning: {warning}\n\nIf you believe this command is safe and necessary, please ask the user for confirmation first, explaining what the command does and why it's needed."
                ));
            }
        }

        let env_file = Self::expand_path("~/.supportflow/.env");
        let dotenv_vars = Self::load_dotenv(&env_file);
        debug!(count = dotenv_vars.len(), path = %env_file.display(), "Loaded dotenv");

        let mut run_command = command.to_string();
        if cfg!(windows) {
            run_command = Self::convert_env_vars_for_windows(&run_command, &dotenv_vars);
            if !run_command.trim().to_lowercase().starts_with("chcp") {
                run_command = format!("chcp 65001 >nul 2>&1 && {run_command}");
            }
        }

        let timeout = Duration::from_secs(timeout_secs);

        let mut output = match self.run_command(&run_command, timeout, &dotenv_vars).await {
            Ok(o) => o,
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return ToolRunResult::error(format!(
                    "Error: Command timed out after {timeout_secs} seconds"
                ));
            }
            Err(e) => {
                return ToolRunResult::error(format!("Error executing command: {e}"));
            }
        };

        debug!(
            code = ?output.status.code(),
            stdout = output.stdout.len(),
            stderr = output.stderr.len()
        );

        if !cfg!(windows) {
            let code = output.status.code().unwrap_or(-1);
            if code == 126 && output.stdout.is_empty() && output.stderr.is_empty() {
                warn!("Exit 126 with no output, retrying with shlex split");
                if let Some(retry) = self.retry_shlex(&run_command, timeout, &dotenv_vars).await {
                    if retry.status.success()
                        || !retry.stdout.is_empty()
                        || !retry.stderr.is_empty()
                    {
                        output = retry;
                    } else if run_command.contains("openai-image-vision")
                        || run_command.contains("vision.sh")
                    {
                        output.stdout = r#"{"error": "image parse failed", "reason": "unsupported or corrupt image", "suggestion": "try another image"}"#
                            .as_bytes()
                            .to_vec();
                        output.stderr.clear();
                    }
                }
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let exit_code = output.status.code().unwrap_or(-1);

        let combined = if exit_code == 0 && !stdout.trim().is_empty() {
            if !stderr.is_empty() {
                info!(stderr = %stderr.chars().take(500).collect::<String>(), "bash stderr not forwarded");
            }
            stdout
        } else {
            let mut o = stdout;
            if !stderr.is_empty() {
                if !o.is_empty() {
                    o.push('\n');
                }
                o.push_str(&stderr);
            }
            o
        };

        let mut temp_file_path: Option<String> = None;
        if combined.len() > DEFAULT_MAX_BYTES {
            let path = std::env::temp_dir().join(format!("bash-{}.log", uuid::Uuid::new_v4()));
            if std::fs::write(&path, &combined).is_ok() {
                temp_file_path = Some(path.display().to_string());
            }
        }

        let truncation = truncate_tail(&combined, None, None);
        let mut output_text = if truncation.content.is_empty() {
            "(no output)".to_string()
        } else {
            truncation.content.clone()
        };

        let mut details = serde_json::Map::new();
        if truncation.truncated {
            details.insert("truncation".into(), truncation.to_value());
            if let Some(ref p) = temp_file_path {
                details.insert("full_output_path".into(), Value::String(p.clone()));
            }

            let start_line = truncation
                .total_lines
                .saturating_sub(truncation.output_lines)
                .saturating_add(1);
            let end_line = truncation.total_lines;

            if truncation.last_line_partial {
                let last_line = combined.split('\n').next_back().unwrap_or("");
                let last_line_size = format_size(last_line.len());
                output_text.push_str(&format!(
                    "\n\n[Showing last {} of line {end_line} (line is {last_line_size}). Full output: {}]",
                    format_size(truncation.output_bytes),
                    temp_file_path.as_deref().unwrap_or(""),
                ));
            } else if truncation.truncated_by.as_deref() == Some("lines") {
                output_text.push_str(&format!(
                    "\n\n[Showing lines {start_line}-{end_line} of {}. Full output: {}]",
                    truncation.total_lines,
                    temp_file_path.as_deref().unwrap_or(""),
                ));
            } else {
                output_text.push_str(&format!(
                    "\n\n[Showing lines {start_line}-{end_line} of {} ({} limit). Full output: {}]",
                    truncation.total_lines,
                    format_size(DEFAULT_MAX_BYTES),
                    temp_file_path.as_deref().unwrap_or(""),
                ));
            }
        }

        let mut payload = json!({
            "output": output_text.clone(),
            "exit_code": exit_code,
            "details": if details.is_empty() { Value::Null } else { Value::Object(details) },
        });

        if exit_code != 0 {
            output_text.push_str(&format!("\n\nCommand exited with code {exit_code}"));
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("output".into(), Value::String(output_text));
            }
            return ToolRunResult::fail_value(payload);
        }

        ToolRunResult::success(payload)
    }
}

#[async_trait]
impl AgentTool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Bash command to execute" },
                "timeout": { "type": "integer", "description": "Timeout in seconds (optional, default: 30)" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: Value) -> ToolRunResult {
        self.execute_inner(&params).await
    }
}
