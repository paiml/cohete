//! Cohete: Nightly E2E verification for the sovereign AI stack on Jetson.
//!
//! Runs 5 tiers of tests against pre-installed binaries and produces
//! falsifiable JSON artifacts proving the stack works on edge hardware.

mod runner;
mod tier1_smoke;
mod tier2_hardware;
mod tier3_functional;
mod tier4_integration;
mod tier5_performance;
mod types;
mod uat;

use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "cohete", version, about = "Nightly E2E verification for the sovereign AI stack")]
enum Cli {
    /// Run all 5 verification tiers and emit JSON artifacts.
    Verify {
        /// Directory to write artifact JSON files.
        #[arg(short, long, default_value = "artifacts/latest")]
        output: PathBuf,

        /// Only run tiers up to this level (1-5).
        #[arg(long)]
        max_tier: Option<u8>,

        /// Print results to stdout instead of writing files.
        #[arg(long, default_value_t = false)]
        stdout: bool,

        /// Continue past tier 1 even if some binaries are missing.
        /// Missing binaries are reported as warnings, not failures.
        #[arg(long, default_value_t = false)]
        allow_missing: bool,

        /// Path to LLM model file (.gguf or .apr).
        /// Overrides `COHETE_MODEL` env var and auto-discovery.
        #[arg(long)]
        model: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Verify {
            output,
            max_tier,
            stdout,
            allow_missing,
            model,
        } => {
            let max = max_tier.unwrap_or(5).clamp(1, 5);
            let model_str = model.as_ref().map(|p| p.to_string_lossy().to_string());
            let result = run_verify(&output, max, stdout, allow_missing, model_str.as_deref());
            std::process::exit(i32::from(!result));
        }
    }
}

fn run_verify(output: &Path, max_tier: u8, stdout: bool, allow_missing: bool, cli_model: Option<&str>) -> bool {
    let started = chrono::Utc::now();

    if !stdout {
        if let Err(e) = std::fs::create_dir_all(output) {
            eprintln!("error: cannot create output directory {}: {e}", output.display());
            return false;
        }
    }

    // Resolve model paths (CLI > env > cache > legacy)
    eprintln!("=== Model Resolution ===");
    let models = types::ModelConfig::resolve(cli_model);

    // Tier 1: Smoke (gate)
    eprintln!("=== Tier 1: Smoke ===");
    let smoke = tier1_smoke::run();
    emit("smoke.json", &smoke, output, stdout);

    let history_dir = output.parent().map(|p| p.join("history"));
    let history_ref = history_dir.as_deref();

    if !smoke.pass && !allow_missing {
        eprintln!("FATAL: tier 1 smoke failed — aborting (use --allow-missing to continue)");
        let summary = types::Summary::new(started, false, &smoke, None, None, None, None, history_ref);
        emit("summary.json", &summary, output, stdout);
        return false;
    }
    if !smoke.pass && allow_missing {
        eprintln!("WARN: tier 1 smoke has failures — continuing (--allow-missing)");
    }

    // Tier 2: Hardware
    let hardware = if max_tier >= 2 {
        eprintln!("=== Tier 2: Hardware ===");
        let hw = tier2_hardware::run();
        emit("hardware.json", &hw, output, stdout);
        Some(hw)
    } else {
        None
    };

    // Tier 3: Functional (M1, M5, GPU/CPU parity)
    let functional = if max_tier >= 3 {
        eprintln!("=== Tier 3: Functional ===");
        let func = tier3_functional::run(&models);
        emit("functional.json", &func, output, stdout);
        Some(func)
    } else {
        None
    };

    // Tier 4: Integration (M2, M3, M4, M6)
    let integration = if max_tier >= 4 {
        eprintln!("=== Tier 4: Integration ===");
        let integ = tier4_integration::run(&models);
        emit("integration.json", &integ, output, stdout);
        Some(integ)
    } else {
        None
    };

    // Tier 5: Performance
    let performance = if max_tier >= 5 {
        eprintln!("=== Tier 5: Performance ===");
        let perf = tier5_performance::run(&models, history_ref);
        emit("performance.json", &perf, output, stdout);
        Some(perf)
    } else {
        None
    };

    let smoke_ok = if allow_missing { smoke.pass_installed() } else { smoke.pass };
    let pass = smoke_ok
        && hardware.as_ref().map_or(true, |h| h.pass)
        && functional.as_ref().map_or(true, |f| f.pass)
        && integration.as_ref().map_or(true, |i| i.pass)
        && performance.as_ref().map_or(true, |p| p.pass);

    let summary = types::Summary::new(
        started,
        pass,
        &smoke,
        hardware.as_ref(),
        functional.as_ref(),
        integration.as_ref(),
        performance.as_ref(),
        history_ref,
    );
    emit("summary.json", &summary, output, stdout);

    if pass {
        eprintln!("ALL TIERS PASSED");
    } else {
        eprintln!("SOME TIERS FAILED — see artifacts for details");
    }

    pass
}

fn emit(name: &str, value: &impl serde::Serialize, output: &Path, stdout: bool) {
    let json = match serde_json::to_string_pretty(value) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("error: failed to serialize {name}: {e}");
            return;
        }
    };

    if stdout {
        println!("--- {name} ---");
        println!("{json}");
    } else {
        let path = output.join(name);
        if let Err(e) = std::fs::write(&path, &json) {
            eprintln!("error: failed to write {}: {e}", path.display());
        } else {
            eprintln!("  wrote {}", path.display());
        }
    }
}
