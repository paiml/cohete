//! Tier 4: Integration tests — cross-binary pipelines.
//!
//! M2: Chat server (apr serve run)
//! M3: Correctness (6 deterministic tests against chat server)
//! M4: Load test (concurrent requests)
//! M6: RAG pipeline (whisper → trueno-rag)

use crate::runner;
use crate::types::{
    CorrectnessStatus, IntegrationModalities, IntegrationResult, ModalityStatus, ModelConfig,
};
use crate::uat;

const SERVE_PORT: u16 = 8090; // Avoid common ports
const SERVE_TIMEOUT_S: u64 = 30;

pub fn run(config: &ModelConfig) -> IntegrationResult {
    let mut total: u32 = 0;
    let mut passed: u32 = 0;
    let mut failed: u32 = 0;
    let mut skipped: u32 = 0;

    // M2 + M3 + UAT + M4: Chat server, correctness, UAT, then load test
    let (m2, m3, m4, uat_result) = if config.has_model() {
        let model_path = config.model_path.as_deref().unwrap_or("");
        let (server, correct, uat_r, load) = run_server_tests(model_path, config);
        total += 3;
        count_status(&server, &mut passed, &mut failed, &mut skipped);
        if let Some(ref c) = correct {
            if c.pass { passed += 1; } else { failed += 1; }
        } else {
            skipped += 1;
        }
        count_modality_status(&load, &mut passed, &mut failed, &mut skipped);
        // UAT counts
        if let Some(ref u) = uat_r {
            total += u.total;
            passed += u.passed;
            failed += u.failed;
        }
        (Some(server), correct, Some(load), uat_r)
    } else {
        eprintln!("  M2/M3/M4: SKIP (no model found — set --model, COHETE_MODEL, or `apr pull`)");
        total += 3;
        skipped += 3;
        (None, None, None, None)
    };

    // M6: RAG pipeline
    let m6 = if config.has_whisper() {
        total += 1;
        let whisper_path = config.whisper_model_path.as_deref().unwrap_or("");
        let audio_path = config.test_audio_path.as_deref().unwrap_or("");
        let rag = run_rag_pipeline(whisper_path, audio_path);
        count_modality_status(&rag, &mut passed, &mut failed, &mut skipped);
        Some(rag)
    } else {
        eprintln!("  M6: SKIP (whisper model or audio not present)");
        total += 1;
        skipped += 1;
        None
    };

    let pass = failed == 0;

    IntegrationResult {
        tier: 4,
        pass,
        total,
        passed,
        failed,
        skipped,
        modalities: IntegrationModalities {
            m2_chat_server: m2,
            m3_correctness: m3,
            m4_load_test: m4,
            m6_rag_pipeline: m6,
        },
        uat: uat_result,
    }
}

fn count_status(ms: &ModalityStatus, passed: &mut u32, failed: &mut u32, _skipped: &mut u32) {
    if ms.pass { *passed += 1; } else { *failed += 1; }
}

fn count_modality_status(ms: &ModalityStatus, passed: &mut u32, failed: &mut u32, _skipped: &mut u32) {
    if ms.pass { *passed += 1; } else { *failed += 1; }
}

/// Start apr serve run, run correctness + UAT + load tests, then stop server.
fn run_server_tests(
    model_path: &str,
    config: &ModelConfig,
) -> (ModalityStatus, Option<CorrectnessStatus>, Option<uat::UatResult>, ModalityStatus) {
    eprintln!("  M2: starting apr serve run on port {SERVE_PORT}...");
    let port_str = SERVE_PORT.to_string();
    let Some(mut child) = runner::spawn(
        "apr",
        &["serve", "run", model_path, "--port", &port_str],
    ) else {
        eprintln!("  M2: failed to spawn apr serve");
        return (
            ModalityStatus { pass: false, detail: "failed to spawn apr serve".into() },
            None,
            None,
            ModalityStatus { pass: false, detail: "skipped (no server)".into() },
        );
    };

    // Wait for health endpoint
    let health_cmd = format!(
        "for i in $(seq 1 {SERVE_TIMEOUT_S}); do \
           curl -sf http://localhost:{SERVE_PORT}/health && exit 0; \
           sleep 1; \
         done; exit 1"
    );
    let health = runner::shell(&health_cmd);

    let m2 = if health.success {
        eprintln!("  M2: chat server healthy");
        ModalityStatus { pass: true, detail: "health endpoint OK".into() }
    } else {
        eprintln!("  M2: chat server failed to start");
        let _ = child.kill();
        let _ = child.wait();
        cleanup_server();
        return (
            ModalityStatus { pass: false, detail: "server did not become healthy".into() },
            None,
            None,
            ModalityStatus { pass: false, detail: "skipped (no server)".into() },
        );
    };

    // M3: Correctness tests
    let m3 = run_correctness_tests();
    eprintln!(
        "  M3: correctness {}/{} passed",
        m3.passed, m3.total
    );

    // UAT: Real-world problem solving (U1-U4) — server is alive
    let uat_r = uat::run(config);

    // M4: Simple load test (2 concurrent requests)
    let m4 = run_load_test();
    eprintln!("  M4: load test {}", if m4.pass { "PASS" } else { "FAIL" });

    let _ = child.kill();
    let _ = child.wait();
    cleanup_server();

    (m2, Some(m3), Some(uat_r), m4)
}

fn run_correctness_tests() -> CorrectnessStatus {
    struct CorrectnessTest {
        name: &'static str,
        prompt: &'static str,
    }

    let tests = [
        CorrectnessTest { name: "basic_math", prompt: "What is 7 * 8?" },
        CorrectnessTest { name: "python_fibonacci", prompt: "Write a Python fibonacci function" },
        CorrectnessTest { name: "rust_hello", prompt: "Write hello world in Rust" },
        CorrectnessTest { name: "json_output", prompt: "Return JSON with name Alice" },
        CorrectnessTest { name: "code_explanation", prompt: "What does map do on a vector?" },
        CorrectnessTest { name: "sql_query", prompt: "Write SQL for top 5 users by orders" },
    ];

    let mut passed = 0u32;
    #[allow(clippy::cast_possible_truncation)]
    let total = tests.len() as u32;

    for test in &tests {
        let body = serde_json::json!({
            "model": "default",
            "messages": [{"role": "user", "content": test.prompt}],
            "max_tokens": 128,
            "temperature": 0
        });

        let cmd = format!(
            "curl -sf -X POST http://localhost:{SERVE_PORT}/v1/chat/completions \
             -H 'Content-Type: application/json' \
             -d '{body}'"
        );
        let result = runner::shell(&cmd);

        let pass = if result.success {
            check_correctness(test.name, &result.stdout)
        } else {
            false
        };

        if pass {
            passed += 1;
        }
        eprintln!("    {}: {}", test.name, if pass { "PASS" } else { "FAIL" });
    }

    CorrectnessStatus {
        pass: passed == total,
        total,
        passed,
    }
}

/// Check correctness per spec-defined criteria.
fn check_correctness(test_name: &str, output: &str) -> bool {
    let lower = output.to_lowercase();
    match test_name {
        "basic_math" => output.contains("56"),
        "python_fibonacci" => output.contains("def fib") || output.contains("def fibonacci"),
        "rust_hello" => output.contains("fn main"),
        "json_output" => output.contains("name") && output.contains("Alice"),
        "code_explanation" => {
            lower.contains("double")
                || lower.contains("multiply")
                || lower.contains("transform")
                || output.contains('2')
        }
        "sql_query" => {
            let up = output.to_uppercase();
            up.contains("SELECT") && up.contains("ORDER BY") && up.contains("LIMIT")
        }
        _ => false,
    }
}

fn run_load_test() -> ModalityStatus {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "Hello"}],
        "max_tokens": 16,
        "temperature": 0
    });

    let cmd = format!(
        "for i in 1 2; do \
           curl -sf -X POST http://localhost:{SERVE_PORT}/v1/chat/completions \
             -H 'Content-Type: application/json' \
             -d '{body}' &\n\
         done; wait"
    );

    let result = runner::shell(&cmd);

    ModalityStatus {
        pass: result.success,
        detail: format!("2 concurrent requests, {}ms", result.duration_ms),
    }
}

fn run_rag_pipeline(whisper_path: &str, audio_path: &str) -> ModalityStatus {
    eprintln!("  M6: running RAG pipeline (transcribe → index → query)...");

    // Step 1: Transcribe
    let transcribe = runner::shell(&format!(
        "whisper-apr transcribe {audio_path} \
         --model {whisper_path} \
         -o /tmp/cohete-transcript.txt"
    ));

    if !transcribe.success {
        return ModalityStatus {
            pass: false,
            detail: format!("transcription failed: {}", transcribe.stderr),
        };
    }

    // Step 2: Index
    let index = runner::shell(
        "while IFS= read -r line; do \
           printf '{\"text\": \"%s\"}\\n' \"$line\"; \
         done < /tmp/cohete-transcript.txt > /tmp/cohete-pipeline.jsonl && \
         trueno-rag index --sqlite /tmp/cohete-pipeline.db /tmp/cohete-pipeline.jsonl"
    );

    if !index.success {
        cleanup_rag_tmp();
        return ModalityStatus {
            pass: false,
            detail: format!("indexing failed: {}", index.stderr),
        };
    }

    // Step 3: Query
    let query = runner::shell(
        "trueno-rag query --sqlite /tmp/cohete-pipeline.db 'what was said'"
    );

    cleanup_rag_tmp();

    let pass = query.success && !query.stdout.is_empty();
    eprintln!("  M6: RAG pipeline {}", if pass { "PASS" } else { "FAIL" });

    ModalityStatus {
        pass,
        detail: if pass { "transcribe→index→query complete".into() } else { format!("query failed: {}", query.stderr) },
    }
}

fn cleanup_server() {
    let _ = runner::shell("pkill -f 'apr serve' 2>/dev/null || true");
}

fn cleanup_rag_tmp() {
    let _ = runner::shell("rm -f /tmp/cohete-transcript.txt /tmp/cohete-pipeline.jsonl /tmp/cohete-pipeline.db");
}
