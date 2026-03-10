//! Process runner — execute commands and capture output.

use std::process::Command;
use std::time::Instant;

/// Result of running a shell command.
pub struct CmdResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Run a command with arguments, capturing stdout/stderr.
pub fn run(program: &str, args: &[&str]) -> CmdResult {
    let start = Instant::now();

    match Command::new(program).args(args).output() {
        Ok(output) => CmdResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            duration_ms: elapsed_ms(&start),
        },
        Err(e) => CmdResult {
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: format!("failed to execute {program}: {e}"),
            duration_ms: elapsed_ms(&start),
        },
    }
}

/// Run a shell command via `sh -c`, for pipelines and complex commands.
pub fn shell(cmd: &str) -> CmdResult {
    run("sh", &["-c", cmd])
}

fn elapsed_ms(start: &std::time::Instant) -> u64 {
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Check if a file exists and is executable.
pub fn is_executable(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| {
            use std::os::unix::fs::PermissionsExt;
            m.permissions().mode() & 0o111 != 0
        })
        .unwrap_or(false)
}
