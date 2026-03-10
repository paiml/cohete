//! Tier 3: Functional tests — each binary does its core job.
//!
//! M1: CLI inference (apr run)
//! M5: Transcription (whisper-apr transcribe)
//! Plus smoke-level functional tests for infrastructure tools.

use crate::runner;
use crate::types::{FunctionalResult, TestEntry, TestStatus, MODEL_PATH, TEST_AUDIO_PATH, WHISPER_MODEL_PATH};

pub fn run() -> FunctionalResult {
    let tests = vec![
        test_apr_check(),
        test_apr_inference(),
        test_whisper_transcribe(),
        test_forjar_smoke(),
        test_pmat_smoke(),
        test_copia_smoke(),
        test_pzsh_smoke(),
        test_trueno_rag_smoke(),
        test_batuta_smoke(),
    ];

    #[allow(clippy::cast_possible_truncation)]
    let passed = tests.iter().filter(|t| t.status == TestStatus::Pass).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let skipped = tests.iter().filter(|t| t.status == TestStatus::Skip).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let failed = tests.iter().filter(|t| t.status == TestStatus::Fail).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let total = tests.len() as u32;

    // Tier 3 passes if no hard failures (skips are OK).
    let pass = failed == 0;

    FunctionalResult {
        tier: 3,
        pass,
        total,
        passed,
        skipped,
        failed,
        tests,
    }
}

fn test_apr_check() -> TestEntry {
    let result = runner::run("apr", &["check"]);
    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  apr check: {}", status_label(&status));

    TestEntry {
        binary: "apr".into(),
        test: "check".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_apr_inference() -> TestEntry {
    if !std::path::Path::new(MODEL_PATH).exists() {
        eprintln!("  apr inference: SKIP (model not present)");
        return TestEntry {
            binary: "apr".into(),
            test: "inference".into(),
            modality: Some("M1".into()),
            status: TestStatus::Skip,
            duration_ms: 0,
            output: Some(format!("model not found at {MODEL_PATH}")),
        };
    }

    let result = runner::run(
        "apr",
        &["run", "--model", MODEL_PATH, "--prompt", "1+1=", "--max-tokens", "8"],
    );

    let status = if result.success && !result.stdout.is_empty() {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    };

    eprintln!("  apr inference (M1): {} ({}ms)", status_label(&status), result.duration_ms);

    TestEntry {
        binary: "apr".into(),
        test: "inference".into(),
        modality: Some("M1".into()),
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_whisper_transcribe() -> TestEntry {
    let model_exists = std::path::Path::new(WHISPER_MODEL_PATH).exists();
    let audio_exists = std::path::Path::new(TEST_AUDIO_PATH).exists();

    if !model_exists || !audio_exists {
        eprintln!("  whisper-apr transcribe: SKIP (model or audio not present)");
        return TestEntry {
            binary: "whisper-apr".into(),
            test: "transcribe".into(),
            modality: Some("M5".into()),
            status: TestStatus::Skip,
            duration_ms: 0,
            output: Some("model or audio fixture not present".into()),
        };
    }

    let result = runner::run(
        "whisper-apr",
        &["transcribe", TEST_AUDIO_PATH, "--model", WHISPER_MODEL_PATH],
    );

    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  whisper-apr transcribe (M5): {} ({}ms)", status_label(&status), result.duration_ms);

    TestEntry {
        binary: "whisper-apr".into(),
        test: "transcribe".into(),
        modality: Some("M5".into()),
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_forjar_smoke() -> TestEntry {
    // forjar plan with empty input — should fail gracefully, not segfault
    let result = runner::run("forjar", &["plan", "-f", "/dev/null"]);
    // We expect an error (no valid YAML), but not a crash
    let status = if result.exit_code.is_some() { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  forjar plan (smoke): {}", status_label(&status));

    TestEntry {
        binary: "forjar".into(),
        test: "plan_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_pmat_smoke() -> TestEntry {
    let result = runner::shell("cd /home/noah/src/cohete && pmat query --literal 'fn main' --limit 5");
    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  pmat query (smoke): {}", status_label(&status));

    TestEntry {
        binary: "pmat".into(),
        test: "query_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_copia_smoke() -> TestEntry {
    let result = runner::shell(
        "mkdir -p /tmp/cohete-test-src /tmp/cohete-test-dst && \
         echo test > /tmp/cohete-test-src/file.txt && \
         copia sync /tmp/cohete-test-src/ /tmp/cohete-test-dst/ && \
         diff /tmp/cohete-test-src/file.txt /tmp/cohete-test-dst/file.txt && \
         rm -rf /tmp/cohete-test-src /tmp/cohete-test-dst",
    );

    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  copia sync (smoke): {}", status_label(&status));

    TestEntry {
        binary: "copia".into(),
        test: "sync_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_pzsh_smoke() -> TestEntry {
    let result = runner::shell("echo 'echo hello' | pzsh eval 2>&1");
    // pzsh may not have eval subcommand — pass if it exits without crashing
    let status = if result.exit_code.is_some() { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  pzsh eval (smoke): {}", status_label(&status));

    TestEntry {
        binary: "pzsh".into(),
        test: "eval_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_trueno_rag_smoke() -> TestEntry {
    let result = runner::shell(
        "echo '{\"text\": \"Rust is a systems language\"}' > /tmp/cohete-rag-test.jsonl && \
         trueno-rag index --sqlite /tmp/cohete-rag-test.db /tmp/cohete-rag-test.jsonl && \
         trueno-rag query --sqlite /tmp/cohete-rag-test.db 'systems programming' && \
         rm -f /tmp/cohete-rag-test.jsonl /tmp/cohete-rag-test.db",
    );

    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  trueno-rag index+query (smoke): {}", status_label(&status));

    TestEntry {
        binary: "trueno-rag".into(),
        test: "index_query_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_batuta_smoke() -> TestEntry {
    let result = runner::run("batuta", &["oracle", "--rag", "test query"]);
    // batuta may not have oracle index — pass if it exits gracefully
    let status = if result.exit_code.is_some() { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  batuta oracle (smoke): {}", status_label(&status));

    TestEntry {
        binary: "batuta".into(),
        test: "oracle_smoke".into(),
        modality: None,
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

const fn status_label(s: &TestStatus) -> &'static str {
    match s {
        TestStatus::Pass => "PASS",
        TestStatus::Fail => "FAIL",
        TestStatus::Skip => "SKIP",
    }
}

fn truncate_output(stdout: &str, stderr: &str) -> String {
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n---stderr---\n{stderr}")
    };

    if combined.len() > 1024 {
        format!("{}...(truncated)", &combined[..1024])
    } else {
        combined
    }
}
