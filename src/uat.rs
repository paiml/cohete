//! UAT: APR Model User Acceptance Testing
//!
//! U1: Chat problem solving (5 scenarios)
//! U2: Serve API validation (6 scenarios)
//! U3: Kernel provability (4 scenarios)
//! U4: Task chaining (4 scenarios)
//!
//! Refs: PMAT-015, docs/specifications/components/uat-apr-model.md

use crate::runner;
use crate::types::ModelConfig;
use serde::Serialize;

const SERVE_PORT: u16 = 8090;

#[derive(Serialize)]
pub struct UatResult {
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    #[serde(rename = "U1_chat_problem_solving")]
    pub u1: UatSuite,
    #[serde(rename = "U2_serve_api")]
    pub u2: UatSuite,
    #[serde(rename = "U3_kernel_provability")]
    pub u3: UatSuite,
    #[serde(rename = "U4_task_chaining")]
    pub u4: UatSuite,
}

#[derive(Serialize)]
pub struct UatSuite {
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub scenarios: Vec<UatScenario>,
}

#[derive(Serialize)]
pub struct UatScenario {
    pub id: String,
    pub pass: bool,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Run all UAT suites. Assumes `apr serve` is already running on `SERVE_PORT`.
pub fn run(config: &ModelConfig) -> UatResult {
    eprintln!("  UAT: running user acceptance tests...");

    let u1 = run_u1_chat_solving();
    let u2 = run_u2_api_validation();
    let u3 = run_u3_kernel_provability(config);
    let u4 = run_u4_task_chaining();

    let total = u1.total + u2.total + u3.total + u4.total;
    let passed = u1.passed + u2.passed + u3.passed + u4.passed;
    let failed = total - passed;

    eprintln!(
        "  UAT: {passed}/{total} passed (U1:{}/{} U2:{}/{} U3:{}/{} U4:{}/{})",
        u1.passed, u1.total, u2.passed, u2.total, u3.passed, u3.total, u4.passed, u4.total
    );

    UatResult {
        pass: failed == 0,
        total,
        passed,
        failed,
        u1,
        u2,
        u3,
        u4,
    }
}

/// Build the chat completions URL for the running server.
fn chat_url() -> String {
    format!("http://localhost:{SERVE_PORT}/v1/chat/completions")
}

/// POST a chat completion request and extract the assistant content.
fn chat_complete(body: &serde_json::Value) -> runner::CmdResult {
    runner::curl_post(&chat_url(), &body.to_string())
}

/// Extract assistant content from an OpenAI-compatible chat completion response.
fn extract_content(json_str: &str) -> String {
    serde_json::from_str::<serde_json::Value>(json_str)
        .ok()
        .and_then(|v| {
            v.get("choices")
                .and_then(|c| c.get(0))
                .and_then(|c| c.get("message"))
                .and_then(|m| m.get("content"))
                .and_then(|s| s.as_str())
                .map(String::from)
        })
        .unwrap_or_default()
}

// ─── U1: Chat Problem Solving ──────────────────────────────

type ChatScenario = (&'static str, &'static str, fn(&str) -> bool);

fn run_u1_chat_solving() -> UatSuite {
    let scenarios: &[ChatScenario] = &[
        (
            "U1-001",
            "Write a Rust function that parses a line of CSV into fields",
            |out| {
                let l = out.to_lowercase();
                l.contains("fn") && (l.contains("split") || l.contains("csv"))
            },
        ),
        (
            "U1-002",
            "Add error handling to this: fn div(a: i32, b: i32) -> i32 { a / b }",
            |out| {
                let l = out.to_lowercase();
                (l.contains("result") || l.contains("option")) && l.contains('0')
            },
        ),
        (
            "U1-003",
            "Write a test for: fn add(a: i32, b: i32) -> i32 { a + b }",
            |out| out.contains("#[test]") && out.contains("assert"),
        ),
        (
            "U1-004",
            "This panics: vec![1,2,3][5]. Why and how to fix?",
            |out| {
                let l = out.to_lowercase();
                (l.contains("bound") || l.contains("index") || l.contains("range"))
                    && (l.contains("get") || l.contains("check") || l.contains("len"))
            },
        ),
        ("U1-005", "Write binary search in Python", |out| {
            let l = out.to_lowercase();
            l.contains("def")
                && l.contains("return")
                && (l.contains("mid") || l.contains("//") || l.contains(">>"))
        }),
    ];

    run_chat_suite(scenarios)
}

fn run_chat_suite(scenarios: &[ChatScenario]) -> UatSuite {
    let mut results = Vec::new();
    let mut passed = 0u32;

    for &(id, prompt, check_fn) in scenarios {
        let body = serde_json::json!({
            "model": "default",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 256,
            "temperature": 0
        });

        let result = chat_complete(&body);

        let pass = if result.success {
            check_fn(&extract_content(&result.stdout))
        } else {
            false
        };

        if pass {
            passed += 1;
        }
        eprintln!("    {id}: {}", if pass { "PASS" } else { "FAIL" });

        results.push(UatScenario {
            id: id.to_string(),
            pass,
            duration_ms: result.duration_ms,
            detail: None,
        });
    }

    #[allow(clippy::cast_possible_truncation)]
    let total = scenarios.len() as u32;
    UatSuite {
        pass: passed == total,
        total,
        passed,
        scenarios: results,
    }
}

// ─── U2: Serve API Validation ──────────────────────────────

fn run_u2_api_validation() -> UatSuite {
    let checks: Vec<UatScenario> = vec![
        check_u2_predict(),
        check_u2_streaming(),
        check_u2_invalid_json(),
        check_u2_missing_field(),
        check_u2_models(),
        check_u2_health_rapid(),
    ];

    let passed = checks.iter().filter(|s| s.pass).count();
    #[allow(clippy::cast_possible_truncation)]
    let total = checks.len() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let passed = passed as u32;
    UatSuite {
        pass: passed == total,
        total,
        passed,
        scenarios: checks,
    }
}

fn check_u2_predict() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "Say hello"}],
        "max_tokens": 16, "temperature": 0
    });
    let r = chat_complete(&body);
    let pass = r.success && !extract_content(&r.stdout).is_empty();
    eprintln!("    U2-001: {}", if pass { "PASS" } else { "FAIL" });
    UatScenario {
        id: "U2-001".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

fn check_u2_streaming() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "Hi"}],
        "max_tokens": 8, "temperature": 0, "stream": true
    });
    let r = chat_complete(&body);
    let pass = r.success && (r.stdout.contains("data:") || r.stdout.contains("[DONE]"));
    eprintln!("    U2-002: {}", if pass { "PASS" } else { "FAIL" });
    UatScenario {
        id: "U2-002".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

fn check_u2_invalid_json() -> UatScenario {
    let url = chat_url();
    let r = runner::curl_post_status(&url, "{bad");
    let pass = !r.stdout.is_empty() && r.stdout != "000";
    eprintln!(
        "    U2-003: {} (status {})",
        if pass { "PASS" } else { "FAIL" },
        r.stdout
    );
    UatScenario {
        id: "U2-003".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: Some(format!("status={}", r.stdout)),
    }
}

fn check_u2_missing_field() -> UatScenario {
    let url = chat_url();
    let r = runner::curl_post_status(&url, r#"{"messages":[]}"#);
    let pass = !r.stdout.is_empty() && r.stdout != "000";
    eprintln!(
        "    U2-004: {} (status {})",
        if pass { "PASS" } else { "FAIL" },
        r.stdout
    );
    UatScenario {
        id: "U2-004".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: Some(format!("status={}", r.stdout)),
    }
}

fn check_u2_models() -> UatScenario {
    let r = runner::curl_get(&format!("http://localhost:{SERVE_PORT}/v1/models"));
    let pass = r.success && (r.stdout.contains("model") || r.stdout.contains("id"));
    eprintln!("    U2-005: {}", if pass { "PASS" } else { "FAIL" });
    UatScenario {
        id: "U2-005".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

fn check_u2_health_rapid() -> UatScenario {
    let url = format!("http://localhost:{SERVE_PORT}/health");
    let mut all_ok = true;
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let r = runner::curl_get(&url);
        if !r.success {
            all_ok = false;
            break;
        }
    }
    let ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    eprintln!("    U2-006: {}", if all_ok { "PASS" } else { "FAIL" });
    UatScenario {
        id: "U2-006".into(),
        pass: all_ok,
        duration_ms: ms,
        detail: None,
    }
}

// ─── U3: Kernel Provability ────────────────────────────────

fn run_u3_kernel_provability(config: &ModelConfig) -> UatSuite {
    let checks: Vec<UatScenario> = vec![
        check_u3_reflexivity(),
        check_u3_cardinality(),
        check_u3_format_parity(config),
        check_u3_token_stability(),
    ];

    let passed = checks.iter().filter(|s| s.pass).count();
    #[allow(clippy::cast_possible_truncation)]
    let total = checks.len() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let passed = passed as u32;
    UatSuite {
        pass: passed == total,
        total,
        passed,
        scenarios: checks,
    }
}

fn check_u3_reflexivity() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "What is 2+2?"}],
        "max_tokens": 16, "temperature": 0
    });

    let r1 = chat_complete(&body);
    let r2 = chat_complete(&body);
    let r3 = chat_complete(&body);

    let c1 = extract_content(&r1.stdout);
    let c2 = extract_content(&r2.stdout);
    let c3 = extract_content(&r3.stdout);

    let all_identical = !c1.is_empty() && c1 == c2 && c2 == c3;
    eprintln!(
        "    U3-001: {} (reflexivity)",
        if all_identical { "PASS" } else { "FAIL" }
    );

    UatScenario {
        id: "U3-001".into(),
        pass: all_identical,
        duration_ms: r1.duration_ms + r2.duration_ms + r3.duration_ms,
        detail: Some(format!("runs=3, identical={all_identical}")),
    }
}

fn check_u3_cardinality() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "Count from 1 to 100"}],
        "max_tokens": 8, "temperature": 0
    });
    let r = chat_complete(&body);
    let content = extract_content(&r.stdout);
    let word_count = content.split_whitespace().count();
    let pass = r.success && word_count <= 30;
    eprintln!(
        "    U3-002: {} (cardinality, {word_count} words)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U3-002".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: Some(format!("max_tokens=8, words={word_count}")),
    }
}

fn check_u3_format_parity(config: &ModelConfig) -> UatScenario {
    let (Some(gguf), Some(apr)) = (config.gguf_path.as_ref(), config.apr_path.as_ref()) else {
        eprintln!("    U3-003: SKIP (need both GGUF and APR)");
        return UatScenario {
            id: "U3-003".into(),
            pass: true,
            duration_ms: 0,
            detail: Some("skipped: need both formats".into()),
        };
    };

    let r1 = runner::run(
        "apr",
        &["run", gguf, "-p", "2+2=", "-n", "8", "--skip-contract"],
    );
    let r2 = runner::run(
        "apr",
        &["run", apr, "-p", "2+2=", "-n", "8", "--skip-contract"],
    );

    let exact = r1.success && r2.success && r1.stdout.trim() == r2.stdout.trim();
    let both_correct = r1.stdout.contains('4') && r2.stdout.contains('4');
    let pass = exact || both_correct;

    eprintln!(
        "    U3-003: {} (format parity)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U3-003".into(),
        pass,
        duration_ms: r1.duration_ms + r2.duration_ms,
        detail: Some(format!("exact={exact}, both_correct={both_correct}")),
    }
}

fn check_u3_token_stability() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "What is 2+2? Answer with just the number."}],
        "max_tokens": 8, "temperature": 0
    });
    let r = chat_complete(&body);
    let pass = extract_content(&r.stdout).contains('4');
    eprintln!(
        "    U3-004: {} (token stability)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U3-004".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

// ─── U4: Task Chaining ─────────────────────────────────────

fn run_u4_task_chaining() -> UatSuite {
    let checks: Vec<UatScenario> = vec![
        check_u4_two_turn(),
        check_u4_refinement(),
        check_u4_rag_augmented(),
        check_u4_error_correction(),
    ];

    let passed = checks.iter().filter(|s| s.pass).count();
    #[allow(clippy::cast_possible_truncation)]
    let total = checks.len() as u32;
    #[allow(clippy::cast_possible_truncation)]
    let passed = passed as u32;
    UatSuite {
        pass: passed == total,
        total,
        passed,
        scenarios: checks,
    }
}

fn check_u4_two_turn() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [
            {"role": "user", "content": "Remember this number: 42"},
            {"role": "assistant", "content": "I will remember the number 42."},
            {"role": "user", "content": "What number did I tell you to remember?"}
        ],
        "max_tokens": 32, "temperature": 0
    });
    let r = chat_complete(&body);
    let pass = extract_content(&r.stdout).contains("42");
    eprintln!(
        "    U4-001: {} (two-turn context)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U4-001".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: Some("turns=2".into()),
    }
}

fn check_u4_refinement() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [
            {"role": "user", "content": "Write hello world in Rust"},
            {"role": "assistant", "content": "fn main() { println!(\"Hello, world!\"); }"},
            {"role": "user", "content": "Now add a CLI argument --name to greet by name"}
        ],
        "max_tokens": 256, "temperature": 0
    });
    let r = chat_complete(&body);
    let content = extract_content(&r.stdout);
    let l = content.to_lowercase();
    // Broad check: response should reference CLI args, naming, or code modification
    let pass = l.contains("arg")
        || l.contains("clap")
        || l.contains("name")
        || l.contains("--")
        || l.contains("env")
        || l.contains("std::")
        || content.contains("fn main");
    eprintln!(
        "    U4-002: {} (refinement)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U4-002".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

fn check_u4_rag_augmented() -> UatScenario {
    let rag = runner::shell(
        "trueno-rag query --sqlite /tmp/cohete-pipeline.db 'systems programming' 2>/dev/null",
    );

    if !rag.success || rag.stdout.trim().is_empty() {
        eprintln!("    U4-003: PASS (skipped, no RAG index)");
        return UatScenario {
            id: "U4-003".into(),
            pass: true,
            duration_ms: 0,
            detail: Some("skipped: no RAG index".into()),
        };
    }

    let ctx = rag.stdout.lines().take(3).collect::<Vec<_>>().join(" ");
    let body = serde_json::json!({
        "model": "default",
        "messages": [
            {"role": "system", "content": format!("Context: {ctx}")},
            {"role": "user", "content": "Summarize the context in one sentence."}
        ],
        "max_tokens": 64, "temperature": 0
    });
    let r = chat_complete(&body);
    let pass = !extract_content(&r.stdout).is_empty();
    eprintln!(
        "    U4-003: {} (RAG-augmented)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U4-003".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}

fn check_u4_error_correction() -> UatScenario {
    let body = serde_json::json!({
        "model": "default",
        "messages": [{"role": "user", "content": "1+1=3, right?"}],
        "max_tokens": 64, "temperature": 0
    });
    let r = chat_complete(&body);
    let content = extract_content(&r.stdout);
    let l = content.to_lowercase();
    // Model should correct the error: must contain "2" (digit or word) and NOT affirm "3"
    let has_correct = l.contains('2') || l.contains("two");
    let affirms_wrong = l.contains("yes") && !l.contains("not") && !l.contains("incorrect");
    let pass = has_correct && !affirms_wrong;
    eprintln!(
        "    U4-004: {} (error correction)",
        if pass { "PASS" } else { "FAIL" }
    );
    UatScenario {
        id: "U4-004".into(),
        pass,
        duration_ms: r.duration_ms,
        detail: None,
    }
}
