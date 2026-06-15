//! 一次性子进程：跑完即退出（MarkItDown、license-verifier、bash 等）。

use super::shared::ProcessSharedContext;
use super::spec::CommandSpec;

pub struct OneshotOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OneshotOutput {
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

pub fn run_sync(
    spec: &CommandSpec,
    shared: &ProcessSharedContext,
) -> Result<OneshotOutput, String> {
    spec.validate_program()?;
    let output = spec
        .into_std_command(shared)
        .output()
        .map_err(|e| format!("{}: spawn failed: {e}", spec.name))?;
    Ok(OneshotOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub async fn run_async(
    spec: &CommandSpec,
    shared: &ProcessSharedContext,
) -> Result<OneshotOutput, String> {
    spec.validate_program()?;
    let output = spec
        .into_tokio_command(shared)
        .output()
        .await
        .map_err(|e| format!("{}: spawn failed: {e}", spec.name))?;
    Ok(OneshotOutput {
        code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}
