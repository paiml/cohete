//! Tier 5: Performance baselines — tok/s, RTF, query latency, memory.
//!
//! No gate. Metrics recorded for regression tracking across nightly runs.

use crate::runner;
use crate::types::{PerformanceMetrics, PerformanceResult, MODEL_PATH, TEST_AUDIO_PATH, WHISPER_MODEL_PATH};

pub fn run() -> PerformanceResult {
    let inference_tok_s = bench_inference();
    let whisper_rtf = bench_whisper();
    let rag_query_ms = bench_rag_query();
    let memory_available_mb = read_available_memory();

    PerformanceResult {
        tier: 5,
        pass: true, // Performance tier never gates
        regressions: 0, // TODO: compare against history/
        metrics: PerformanceMetrics {
            inference_tok_s,
            whisper_rtf,
            rag_query_ms,
            memory_available_mb,
        },
    }
}

fn bench_inference() -> Option<f64> {
    if !std::path::Path::new(MODEL_PATH).exists() {
        eprintln!("  inference bench: SKIP (model not present)");
        return None;
    }

    let result = runner::run(
        "apr",
        &["bench", "--model", MODEL_PATH, "--tokens", "32"],
    );

    if !result.success {
        eprintln!("  inference bench: SKIP (apr bench not available)");
        return None;
    }

    // Parse tok/s from output (format depends on apr bench implementation)
    let tok_s = result
        .stdout
        .lines()
        .chain(result.stderr.lines())
        .find_map(|line| {
            if line.contains("tok/s") || line.contains("tokens/s") {
                line.split_whitespace()
                    .find_map(|w| w.parse::<f64>().ok())
            } else {
                None
            }
        });

    if let Some(t) = tok_s {
        eprintln!("  inference: {t:.1} tok/s");
    } else {
        eprintln!("  inference bench: could not parse tok/s");
    }

    tok_s
}

fn bench_whisper() -> Option<f64> {
    let model_exists = std::path::Path::new(WHISPER_MODEL_PATH).exists();
    let audio_exists = std::path::Path::new(TEST_AUDIO_PATH).exists();

    if !model_exists || !audio_exists {
        eprintln!("  whisper bench: SKIP");
        return None;
    }

    let result = runner::run(
        "whisper-apr",
        &["bench", "--model", WHISPER_MODEL_PATH, "--input", TEST_AUDIO_PATH],
    );

    if !result.success {
        eprintln!("  whisper bench: SKIP (whisper-apr bench not available)");
        return None;
    }

    // Parse RTF from output
    let rtf = result
        .stdout
        .lines()
        .chain(result.stderr.lines())
        .find_map(|line| {
            if line.contains("RTF") || line.contains("rtf") || line.contains("real-time") {
                line.split_whitespace()
                    .find_map(|w| w.parse::<f64>().ok())
            } else {
                None
            }
        });

    if let Some(r) = rtf {
        eprintln!("  whisper RTF: {r:.2}");
    }

    rtf
}

fn bench_rag_query() -> Option<f64> {
    // Create a small test index and measure query time
    let setup = runner::shell(
        "echo '{\"text\": \"Rust is a systems programming language\"}' > /tmp/cohete-perf.jsonl && \
         trueno-rag index --sqlite /tmp/cohete-perf.db /tmp/cohete-perf.jsonl 2>/dev/null"
    );

    if !setup.success {
        eprintln!("  rag query bench: SKIP");
        return None;
    }

    let result = runner::shell(
        "trueno-rag query --sqlite /tmp/cohete-perf.db 'systems programming' 2>/dev/null"
    );

    let _ = runner::shell("rm -f /tmp/cohete-perf.jsonl /tmp/cohete-perf.db");

    if result.success {
        #[allow(clippy::cast_precision_loss)]
        let ms = result.duration_ms as f64;
        eprintln!("  rag query: {ms:.0}ms");
        Some(ms)
    } else {
        None
    }
}

fn read_available_memory() -> Option<u64> {
    let result = runner::shell("grep MemAvailable /proc/meminfo | awk '{print $2}'");
    let mb = result.stdout.trim().parse::<u64>().ok().map(|kb| kb / 1024);
    if let Some(m) = mb {
        eprintln!("  memory available: {m} MB");
    }
    mb
}
