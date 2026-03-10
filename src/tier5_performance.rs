//! Tier 5: Performance baselines — tok/s, RTF, query latency, memory.
//!
//! No gate. Metrics recorded for regression tracking across nightly runs.
//! Compares against 7-day rolling average from history/ to detect regressions.

use crate::runner;
use crate::types::{PerformanceMetrics, PerformanceResult, MODEL_PATH, TEST_AUDIO_PATH, WHISPER_MODEL_PATH};
use std::path::Path;

/// Regression threshold: 20% deviation from 7-day rolling average.
const REGRESSION_THRESHOLD: f64 = 0.20;
/// Minimum history days required before regression detection activates.
const MIN_HISTORY_DAYS: usize = 7;

pub fn run(history_dir: Option<&Path>) -> PerformanceResult {
    let inference_tok_s = bench_inference();
    let whisper_rtf = bench_whisper();
    let rag_query_ms = bench_rag_query();
    let memory_available_mb = read_available_memory();

    let metrics = PerformanceMetrics {
        inference_tok_s,
        whisper_rtf,
        rag_query_ms,
        memory_available_mb,
    };

    let regressions = detect_regressions(&metrics, history_dir);

    PerformanceResult {
        tier: 5,
        pass: true, // Performance tier never gates
        regressions,
        metrics,
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

/// Compare today's metrics against 7-day rolling average from history files.
/// Returns the number of regressions detected (>20% deviation).
fn detect_regressions(metrics: &PerformanceMetrics, history_dir: Option<&Path>) -> u32 {
    let Some(dir) = history_dir else {
        return 0;
    };

    let history = load_recent_metrics(dir, MIN_HISTORY_DAYS);
    if history.len() < MIN_HISTORY_DAYS {
        eprintln!(
            "  regression: SKIP ({} days of history, need {MIN_HISTORY_DAYS})",
            history.len()
        );
        return 0;
    }

    let mut regressions = 0u32;

    // inference_tok_s: higher is better → regression if delta < -20%
    if let Some(today) = metrics.inference_tok_s {
        let avgs: Vec<f64> = history.iter().filter_map(|h| h.inference_tok_s).collect();
        if let Some(avg) = rolling_avg(&avgs) {
            let delta = (today - avg) / avg;
            if delta < -REGRESSION_THRESHOLD {
                eprintln!("  REGRESSION: inference {today:.1} tok/s vs avg {avg:.1} ({delta:+.0}%)");
                regressions += 1;
            }
        }
    }

    // whisper_rtf: lower is better → regression if delta > +20%
    if let Some(today) = metrics.whisper_rtf {
        let avgs: Vec<f64> = history.iter().filter_map(|h| h.whisper_rtf).collect();
        if let Some(avg) = rolling_avg(&avgs) {
            let delta = (today - avg) / avg;
            if delta > REGRESSION_THRESHOLD {
                eprintln!("  REGRESSION: whisper RTF {today:.2} vs avg {avg:.2} ({delta:+.0}%)");
                regressions += 1;
            }
        }
    }

    // rag_query_ms: lower is better → regression if delta > +20%
    if let Some(today) = metrics.rag_query_ms {
        let avgs: Vec<f64> = history.iter().filter_map(|h| h.rag_query_ms).collect();
        if let Some(avg) = rolling_avg(&avgs) {
            let delta = (today - avg) / avg;
            if delta > REGRESSION_THRESHOLD {
                eprintln!("  REGRESSION: RAG query {today:.0}ms vs avg {avg:.0}ms ({delta:+.0}%)");
                regressions += 1;
            }
        }
    }

    if regressions == 0 {
        eprintln!("  regression: none detected (vs {MIN_HISTORY_DAYS}-day avg)");
    }

    regressions
}

/// Load performance metrics from the most recent N history files.
fn load_recent_metrics(dir: &Path, days: usize) -> Vec<PerformanceMetrics> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut metrics = Vec::new();

    for i in 1..=days {
        #[allow(clippy::cast_possible_wrap)]
        let offset = i as i64;
        let date = (chrono::Utc::now() - chrono::Duration::days(offset))
            .format("%Y-%m-%d")
            .to_string();
        if date == today {
            continue;
        }
        let path = dir.join(format!("{date}.json"));
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(val) = serde_json::from_str::<serde_json::Value>(&contents) else {
            continue;
        };

        // Extract performance metrics from summary.json history format
        // The history file is a copy of summary.json — performance metrics
        // are not directly in it. Try the top-level "metrics" field first
        // (if someone stored performance.json), then fall back to parsing
        // the tiers.performance section.
        let perf = extract_perf_metrics(&val);
        if perf.inference_tok_s.is_some()
            || perf.whisper_rtf.is_some()
            || perf.rag_query_ms.is_some()
        {
            metrics.push(perf);
        }
    }

    metrics
}

/// Extract performance metrics from a history JSON value.
fn extract_perf_metrics(val: &serde_json::Value) -> PerformanceMetrics {
    // Try direct "metrics" field (if the history file is a performance.json)
    if let Some(m) = val.get("metrics") {
        return PerformanceMetrics {
            inference_tok_s: m.get("inference_tok_s").and_then(serde_json::Value::as_f64),
            whisper_rtf: m.get("whisper_rtf").and_then(serde_json::Value::as_f64),
            rag_query_ms: m.get("rag_query_ms").and_then(serde_json::Value::as_f64),
            memory_available_mb: m.get("memory_available_mb").and_then(serde_json::Value::as_u64),
        };
    }

    // Fallback: empty metrics
    PerformanceMetrics {
        inference_tok_s: None,
        whisper_rtf: None,
        rag_query_ms: None,
        memory_available_mb: None,
    }
}

fn rolling_avg(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    #[allow(clippy::cast_precision_loss)]
    let len = values.len() as f64;
    Some(values.iter().sum::<f64>() / len)
}
