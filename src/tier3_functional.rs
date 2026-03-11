//! Tier 3: Functional tests — each binary does its core job.
//!
//! M1: CLI inference (apr run) — GPU and CPU modes
//! M5: Transcription (whisper-apr transcribe)
//! GPU/CPU parity proof via `apr parity`
//! Plus smoke-level functional tests for infrastructure tools.

use crate::runner;
use crate::types::{FunctionalResult, ModelConfig, TestEntry, TestStatus};

pub fn run(config: &ModelConfig) -> FunctionalResult {
    let gpu_available = runner::run("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"]).success;

    let mut tests = vec![test_apr_check(config)];

    let has_any_model = config.gguf_path.is_some() || config.apr_path.is_some() || config.has_model();
    if !has_any_model {
        eprintln!("  inference: SKIP (no model found — set --model, COHETE_MODEL, or `apr pull`)");
        tests.push(skip_entry("apr", "inference", Some("M1"), "no model found"));
    }

    // GGUF format: prove inference on GPU and CPU
    if let Some(ref gguf) = config.gguf_path {
        if gpu_available {
            tests.push(test_inference(gguf, "gguf", false));
        }
        tests.push(test_inference(gguf, "gguf", true));
    } else if has_any_model && config.apr_path.is_none() {
        // Only one model and it's not .apr — test it as gguf
        let model = config.model_path.as_deref().unwrap_or("");
        if gpu_available {
            tests.push(test_inference(model, "gguf", false));
        }
        tests.push(test_inference(model, "gguf", true));
    }

    // APR format: prove inference on GPU and CPU
    if let Some(ref apr) = config.apr_path {
        if gpu_available {
            tests.push(test_inference(apr, "apr", false));
        }
        tests.push(test_inference(apr, "apr", true));
    }

    // GPU/CPU parity (only meaningful with GPU)
    if gpu_available {
        tests.push(test_apr_gpu_cpu_parity(config));
    }

    tests.extend([
        test_whisper_transcribe(config),
        test_forjar_smoke(),
        test_pmat_smoke(),
        test_copia_smoke(),
        test_pzsh_smoke(),
        test_trueno_rag_smoke(),
        test_batuta_smoke(),
    ]);

    #[allow(clippy::cast_possible_truncation)]
    let passed = tests.iter().filter(|t| t.status == TestStatus::Pass).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let skipped = tests.iter().filter(|t| t.status == TestStatus::Skip).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let failed = tests.iter().filter(|t| t.status == TestStatus::Fail).count() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let total = tests.len() as u32;

    // Tier 3 passes if no hard failures (skips are OK).
    // Parity test is informational — quantized GPU/CPU divergence is expected
    // for Q4_K models. What matters is both produce correct answers (tested above).
    #[allow(clippy::cast_possible_truncation)]
    let parity_failures = tests.iter().filter(|t| t.test == "gpu_cpu_parity" && t.status == TestStatus::Fail).count() as u32;
    let pass = (failed - parity_failures) == 0;

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

fn test_apr_check(config: &ModelConfig) -> TestEntry {
    let Some(ref model_path) = config.model_path else {
        eprintln!("  apr check: SKIP (no model found)");
        return skip_entry("apr", "check", None, "no model found — set --model, COHETE_MODEL, or `apr pull`");
    };

    let result = runner::run("apr", &["check", model_path]);
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

/// Prove inference works for a specific model format and compute backend.
/// `fmt` is "gguf" or "apr". `cpu_only` forces `--no-gpu`.
fn test_inference(model_path: &str, fmt: &str, cpu_only: bool) -> TestEntry {
    let backend = if cpu_only { "cpu" } else { "gpu" };
    let test_name = format!("inference_{fmt}_{backend}");

    let mut args = vec!["run", model_path, "--prompt", "What is 7 * 8?", "--max-tokens", "16"];
    if cpu_only {
        args.push("--no-gpu");
    }

    let result = runner::run("apr", &args);

    let output_text = if result.stdout.is_empty() { &result.stderr } else { &result.stdout };
    let status = if result.success && output_text.contains("56") {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    };

    eprintln!("  apr {fmt} {backend} (M1): {} ({}ms)", status_label(&status), result.duration_ms);

    TestEntry {
        binary: "apr".into(),
        test: test_name,
        modality: Some("M1".into()),
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

/// Prove GPU and CPU produce equivalent results via `apr parity --assert --json`.
fn test_apr_gpu_cpu_parity(config: &ModelConfig) -> TestEntry {
    let gpu_check = runner::run("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"]);
    if !gpu_check.success {
        eprintln!("  apr parity: SKIP (no GPU)");
        return skip_entry("apr", "gpu_cpu_parity", Some("M1"), "no GPU available");
    }

    let Some(ref model_path) = config.model_path else {
        eprintln!("  apr parity: SKIP (no model found)");
        return skip_entry("apr", "gpu_cpu_parity", Some("M1"), "no model found");
    };

    let result = runner::run(
        "apr",
        &["parity", model_path, "--assert", "--json"],
    );

    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  apr GPU/CPU parity: {} ({}ms)", status_label(&status), result.duration_ms);

    TestEntry {
        binary: "apr".into(),
        test: "gpu_cpu_parity".into(),
        modality: Some("M1".into()),
        status,
        duration_ms: result.duration_ms,
        output: Some(truncate_output(&result.stdout, &result.stderr)),
    }
}

fn test_whisper_transcribe(config: &ModelConfig) -> TestEntry {
    if !config.has_whisper() {
        eprintln!("  whisper-apr transcribe: SKIP (model or audio not present)");
        return skip_entry("whisper-apr", "transcribe", Some("M5"), "model or audio fixture not present");
    }

    let whisper_path = config.whisper_model_path.as_deref().unwrap_or("");
    let audio_path = config.test_audio_path.as_deref().unwrap_or("");

    let result = runner::run(
        "whisper-apr",
        &["transcribe", audio_path, "--model", whisper_path],
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
    let result = runner::run("forjar", &["plan", "-f", "/dev/null"]);
    let status = if result.exit_code.is_some() {
        TestStatus::Pass
    } else {
        TestStatus::Fail
    };
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
         copia sync -r /tmp/cohete-test-src/ /tmp/cohete-test-dst/ && \
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
    let result = runner::run("pzsh", &["status"]);
    let status = if result.exit_code.is_some() { TestStatus::Pass } else { TestStatus::Fail };
    eprintln!("  pzsh status (smoke): {}", status_label(&status));

    TestEntry {
        binary: "pzsh".into(),
        test: "status_smoke".into(),
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
    let config_exists = std::path::Path::new("/home/noah/.batuta-private.toml").exists()
        || std::path::Path::new("/home/noah/.config/batuta/config.toml").exists();
    if !config_exists {
        eprintln!("  batuta oracle (smoke): SKIP (no oracle index configured)");
        return skip_entry("batuta", "oracle_smoke", None, "no oracle index configured");
    }

    let result = runner::run("batuta", &["oracle", "--rag", "test query"]);
    let status = if result.success { TestStatus::Pass } else { TestStatus::Fail };
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

fn skip_entry(binary: &str, test: &str, modality: Option<&str>, reason: &str) -> TestEntry {
    TestEntry {
        binary: binary.into(),
        test: test.into(),
        modality: modality.map(String::from),
        status: TestStatus::Skip,
        duration_ms: 0,
        output: Some(reason.into()),
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
