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

/// Spawn a long-running process in the background (does not wait).
/// Returns the child process handle for later cleanup.
pub fn spawn(program: &str, args: &[&str]) -> Option<std::process::Child> {
    Command::new(program)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .ok()
}

/// POST JSON to a URL via curl, bypassing shell quoting.
/// Returns the raw response body on success.
pub fn curl_post(url: &str, json_body: &str) -> CmdResult {
    run(
        "curl",
        &[
            "-sf",
            "-X",
            "POST",
            url,
            "-H",
            "Content-Type: application/json",
            "-d",
            json_body,
        ],
    )
}

/// GET a URL via curl, bypassing shell quoting.
pub fn curl_get(url: &str) -> CmdResult {
    run("curl", &["-sf", url])
}

/// POST to a URL and return the HTTP status code (even for errors).
pub fn curl_post_status(url: &str, body: &str) -> CmdResult {
    run(
        "curl",
        &[
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-X",
            "POST",
            url,
            "-H",
            "Content-Type: application/json",
            "-d",
            body,
        ],
    )
}

/// Check that `path` is a REGULAR FILE and executable.
///
/// `m.is_file()` is load-bearing, and the crate's first-ever unit test is what
/// found that out. The previous body checked only the mode bits — and every
/// directory has the executable bit set, so `is_executable("/tmp")` returned
/// **true**. Each tier uses this to decide whether a stack binary is installed;
/// with a directory at the expected path, a tier would have reported a
/// component present on a box where it was not, and then failed confusingly at
/// the exec.
pub fn is_executable(path: &str) -> bool {
    std::fs::metadata(path).is_ok_and(|m| {
        use std::os::unix::fs::PermissionsExt;
        m.is_file() && m.permissions().mode() & 0o111 != 0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // THE FIRST UNIT TESTS THIS CRATE HAS EVER RUN. Not because nobody wrote
    // any, but because `cargo test --lib` on a bin-only crate is a hard error
    // ("no library targets found in package `cohete`"), so the command that
    // would have run them could never succeed. See src/lib.rs.
    //
    // These cover the pure edges of `runner`: the two predicates every tier
    // depends on before it shells out at all.

    #[test]
    fn is_executable_is_false_for_a_path_that_does_not_exist() {
        // The tiers use this to decide whether a binary is installed. A `true`
        // here would report a stack component present that is not.
        assert!(!is_executable("/nonexistent/definitely/not/here"));
    }

    #[test]
    fn is_executable_is_false_for_a_directory() {
        // A directory has the executable bit set, so a naive mode check says
        // yes. The tiers would then "run" it and report a confusing failure.
        assert!(!is_executable("/tmp"));
    }

    #[test]
    fn is_executable_is_true_for_a_real_binary() {
        // The discriminating half: if this cannot say yes, the two negatives
        // above are satisfied by a function that always returns false.
        assert!(is_executable("/bin/sh"));
    }

    #[test]
    fn run_reports_the_exit_code_it_was_given() {
        // Every tier reads `exit_code` to decide pass/fail, and an unwrapped
        // `None` here is the difference between "failed" and "did not run".
        let ok = run("/bin/sh", &["-c", "exit 0"]);
        assert_eq!(ok.exit_code, Some(0), "a successful command must report 0");

        let bad = run("/bin/sh", &["-c", "exit 3"]);
        assert_eq!(
            bad.exit_code,
            Some(3),
            "a failing command must report ITS code, not a generic 1"
        );
    }

    #[test]
    fn run_captures_stdout_and_stderr_separately() {
        // tier3/tier5 parse stdout for metrics while reporting stderr as the
        // diagnostic. Merging them would make a warning look like a data point.
        let r = run("/bin/sh", &["-c", "echo out; echo err >&2"]);
        assert!(r.stdout.contains("out"), "stdout: {:?}", r.stdout);
        assert!(r.stderr.contains("err"), "stderr: {:?}", r.stderr);
        assert!(
            !r.stdout.contains("err"),
            "stderr must not leak into stdout"
        );
    }

    #[test]
    fn run_on_a_missing_program_does_not_report_success() {
        // The failure mode that matters: a missing binary must never come back
        // looking like a clean run, or a tier reports a component healthy on a
        // box where it is not installed.
        let r = run("/nonexistent/definitely/not/here", &[]);
        assert_ne!(r.exit_code, Some(0));
    }
}
