//! Shared types for cohete verification artifacts.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::uat::UatResult;

/// Binary definition — what cohete expects to find installed.
pub struct BinaryDef {
    pub name: &'static str,
    /// Preferred path (where forjar installs on jetson).
    pub preferred_path: &'static str,
}

impl BinaryDef {
    /// Resolve the actual binary path: preferred path first, then PATH lookup.
    pub fn resolve_path(&self) -> Option<String> {
        // Check preferred path first (jetson installs by forjar)
        if std::path::Path::new(self.preferred_path).exists() {
            return Some(self.preferred_path.to_string());
        }
        // Fall back to PATH lookup (dev machines, cargo install, etc.)
        which(self.name)
    }
}

/// Look up a binary on PATH via `which`.
pub fn which(name: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// All binaries in the matrix.
pub const BINARIES: &[BinaryDef] = &[
    BinaryDef {
        name: "apr",
        preferred_path: "/home/noah/.cargo/bin/apr",
    },
    BinaryDef {
        name: "whisper-apr",
        preferred_path: "/home/noah/.cargo/bin/whisper-apr",
    },
    BinaryDef {
        name: "trueno-rag",
        preferred_path: "/home/noah/.cargo/bin/trueno-rag",
    },
    BinaryDef {
        name: "forjar",
        preferred_path: "/home/noah/.cargo/bin/forjar",
    },
    BinaryDef {
        name: "pmat",
        preferred_path: "/home/noah/.cargo/bin/pmat",
    },
    BinaryDef {
        name: "copia",
        preferred_path: "/home/noah/.cargo/bin/copia",
    },
    BinaryDef {
        name: "pzsh",
        preferred_path: "/home/noah/.cargo/bin/pzsh",
    },
    BinaryDef {
        name: "batuta",
        preferred_path: "/home/noah/.cargo/bin/batuta",
    },
];

/// Legacy hardcoded paths (Jetson provisioned by forjar).
const LEGACY_MODEL_PATH: &str = "/home/noah/data/models/canary/qwen-1.5b-q4k.apr";
const LEGACY_WHISPER_PATH: &str = "/home/noah/data/models/canary/whisper-tiny.en";
const LEGACY_AUDIO_PATH: &str = "/home/noah/data/models/canary/test-2s.wav";

/// Known model cache directory used by `apr pull`.
const PACHA_CACHE_DIR: &str = "/home/noah/.cache/pacha/models";

/// Resolved model paths for the current run.
/// Discovers both .gguf and .apr models to prove both formats work.
/// Priority: CLI flag > env var > cache discovery > legacy path > None.
#[allow(clippy::struct_field_names)]
pub struct ModelConfig {
    /// Primary model (any format). Used by tiers 4/5 for server + bench.
    pub model_path: Option<String>,
    /// GGUF-format model for format-specific verification.
    pub gguf_path: Option<String>,
    /// APR-format model for format-specific verification.
    pub apr_path: Option<String>,
    pub whisper_model_path: Option<String>,
    pub test_audio_path: Option<String>,
}

impl ModelConfig {
    /// Resolve model paths from all available sources.
    pub fn resolve(cli_model: Option<&str>) -> Self {
        let (gguf_path, apr_path) = discover_cached_models();

        // CLI/env override becomes the primary model
        let explicit = Self::resolve_explicit(cli_model);
        // Primary model: explicit > gguf > apr
        let model_path = explicit
            .or_else(|| gguf_path.clone())
            .or_else(|| apr_path.clone())
            .or_else(|| {
                if std::path::Path::new(LEGACY_MODEL_PATH).exists() {
                    Some(LEGACY_MODEL_PATH.to_string())
                } else {
                    None
                }
            });

        let whisper_model_path = Self::resolve_path_chain(
            std::env::var("COHETE_WHISPER_MODEL").ok().as_deref(),
            LEGACY_WHISPER_PATH,
        );
        let test_audio_path = Self::resolve_path_chain(
            std::env::var("COHETE_TEST_AUDIO").ok().as_deref(),
            LEGACY_AUDIO_PATH,
        );

        if let Some(ref p) = gguf_path {
            eprintln!("  gguf: {p}");
        }
        if let Some(ref p) = apr_path {
            eprintln!("  apr:  {p}");
        }
        if gguf_path.is_none() && apr_path.is_none() {
            if let Some(ref p) = model_path {
                eprintln!("  model: {p}");
            } else {
                eprintln!("  model: none found (set --model, COHETE_MODEL, or `apr pull`)");
            }
        }

        Self {
            model_path,
            gguf_path,
            apr_path,
            whisper_model_path,
            test_audio_path,
        }
    }

    /// Resolve explicit model from CLI flag or env var.
    fn resolve_explicit(cli_model: Option<&str>) -> Option<String> {
        if let Some(p) = cli_model {
            if std::path::Path::new(p).exists() {
                return Some(p.to_string());
            }
            eprintln!("  WARN: --model {p} does not exist");
        }
        if let Ok(p) = std::env::var("COHETE_MODEL") {
            if std::path::Path::new(&p).exists() {
                return Some(p);
            }
        }
        None
    }

    fn resolve_path_chain(env_val: Option<&str>, legacy: &str) -> Option<String> {
        if let Some(p) = env_val {
            if std::path::Path::new(p).exists() {
                return Some(p.to_string());
            }
        }
        if std::path::Path::new(legacy).exists() {
            return Some(legacy.to_string());
        }
        None
    }

    pub const fn has_model(&self) -> bool {
        self.model_path.is_some()
    }

    pub const fn has_whisper(&self) -> bool {
        self.whisper_model_path.is_some() && self.test_audio_path.is_some()
    }
}

/// Discover cached models by format. Returns (newest .gguf, newest .apr).
fn discover_cached_models() -> (Option<String>, Option<String>) {
    let dir = std::path::Path::new(PACHA_CACHE_DIR);
    if !dir.is_dir() {
        return (None, None);
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (None, None);
    };

    let mut best_gguf: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    let mut best_apr: Option<(std::time::SystemTime, std::path::PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mtime = entry.metadata().ok().and_then(|m| m.modified().ok());
        let Some(t) = mtime else { continue };

        if ext.eq_ignore_ascii_case("gguf") {
            if best_gguf.as_ref().map_or(true, |(bt, _)| t > *bt) {
                best_gguf = Some((t, path));
            }
        } else if ext.eq_ignore_ascii_case("apr")
            && best_apr.as_ref().map_or(true, |(bt, _)| t > *bt)
        {
            best_apr = Some((t, path));
        }
    }

    (
        best_gguf.map(|(_, p)| p.to_string_lossy().to_string()),
        best_apr.map(|(_, p)| p.to_string_lossy().to_string()),
    )
}

// ─── Tier 1: Smoke ──────────────────────────────────────────

#[derive(Serialize)]
pub struct SmokeResult {
    pub tier: u8,
    pub pass: bool,
    pub binaries: Vec<BinarySmoke>,
}

impl SmokeResult {
    /// Returns true if all *installed* binaries passed smoke tests.
    /// Missing binaries are ignored (for use with `--allow-missing`).
    pub fn pass_installed(&self) -> bool {
        self.binaries
            .iter()
            .filter(|b| b.exists && b.executable)
            .all(|b| b.help_ok && b.version.is_some())
    }
}

#[derive(Serialize)]
pub struct BinarySmoke {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub executable: bool,
    pub version: Option<String>,
    pub help_ok: bool,
}

// ─── Tier 2: Hardware ───────────────────────────────────────

#[derive(Serialize)]
pub struct HardwareResult {
    pub tier: u8,
    pub pass: bool,
    pub gpu: Option<GpuInfo>,
    pub vulkan: Option<String>,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk_ok: bool,
    pub power_mode: Option<String>,
    pub jetpack: Option<String>,
}

#[derive(Serialize)]
pub struct GpuInfo {
    pub model: String,
    pub cuda_version: Option<String>,
}

#[derive(Serialize)]
pub struct CpuInfo {
    pub neon: bool,
    pub cores: u32,
}

#[derive(Serialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

// ─── Tier 3: Functional ─────────────────────────────────────

#[derive(Serialize)]
pub struct FunctionalResult {
    pub tier: u8,
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub skipped: u32,
    pub failed: u32,
    pub warned: u32,
    pub tests: Vec<TestEntry>,
}

#[derive(Serialize)]
pub struct TestEntry {
    pub binary: String,
    pub test: String,
    pub modality: Option<String>,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub output: Option<String>,
}

#[derive(Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Pass,
    Fail,
    Warn,
    Skip,
}

// ─── Tier 4: Integration ────────────────────────────────────

#[derive(Serialize)]
pub struct IntegrationResult {
    pub tier: u8,
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub modalities: IntegrationModalities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uat: Option<UatResult>,
}

#[derive(Serialize)]
pub struct IntegrationModalities {
    #[serde(rename = "M2_chat_server")]
    pub m2_chat_server: Option<ModalityStatus>,
    #[serde(rename = "M3_correctness")]
    pub m3_correctness: Option<CorrectnessStatus>,
    #[serde(rename = "M4_load_test")]
    pub m4_load_test: Option<ModalityStatus>,
    #[serde(rename = "M6_rag_pipeline")]
    pub m6_rag_pipeline: Option<ModalityStatus>,
}

#[derive(Serialize)]
pub struct ModalityStatus {
    pub pass: bool,
    pub detail: String,
}

#[derive(Serialize)]
pub struct CorrectnessStatus {
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
}

// ─── Tier 5: Performance ────────────────────────────────────

#[derive(Serialize)]
pub struct PerformanceResult {
    pub tier: u8,
    pub pass: bool,
    pub regressions: u32,
    pub metrics: PerformanceMetrics,
}

#[derive(Serialize)]
pub struct PerformanceMetrics {
    pub inference_tok_s: Option<f64>,
    pub whisper_rtf: Option<f64>,
    pub rag_query_ms: Option<f64>,
    pub memory_available_mb: Option<u64>,
}

// ─── Summary ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Summary {
    pub schema_version: u32,
    pub date: String,
    pub timestamp: DateTime<Utc>,
    pub cohete_version: String,
    pub pass: bool,
    pub duration_s: u64,
    pub tiers: TiersSummary,
    pub binaries: Vec<BinarySummary>,
    pub version_changes: Vec<VersionChange>,
    pub hardware: Option<HardwareSummary>,
    pub metrics: Option<PerformanceMetrics>,
}

#[derive(Serialize)]
pub struct TiersSummary {
    pub smoke: TierStatus,
    pub hardware: Option<TierStatus>,
    pub functional: Option<TierStatus>,
    pub integration: Option<TierStatus>,
    pub performance: Option<TierStatusPerf>,
}

#[derive(Serialize)]
pub struct TierStatus {
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

impl TierStatus {
    pub const fn from_counts(
        pass: bool,
        total: u32,
        passed: u32,
        failed: u32,
        skipped: u32,
    ) -> Self {
        Self {
            pass,
            total,
            passed,
            failed,
            skipped,
        }
    }
}

#[derive(Serialize)]
pub struct TierStatusPerf {
    pub pass: bool,
    pub regressions: u32,
}

#[derive(Serialize)]
pub struct BinarySummary {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
}

#[derive(Serialize)]
pub struct VersionChange {
    pub binary: String,
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
pub struct HardwareSummary {
    pub gpu: Option<String>,
    pub cuda: Option<String>,
    pub neon: bool,
    pub power_mode: Option<String>,
    pub jetpack: Option<String>,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        started: DateTime<Utc>,
        pass: bool,
        smoke: &SmokeResult,
        hardware: Option<&HardwareResult>,
        functional: Option<&FunctionalResult>,
        integration: Option<&IntegrationResult>,
        performance: Option<&PerformanceResult>,
        history_dir: Option<&std::path::Path>,
    ) -> Self {
        let now = Utc::now();
        let duration_s = (now - started).num_seconds().unsigned_abs();

        let binaries: Vec<BinarySummary> = smoke
            .binaries
            .iter()
            .map(|b| BinarySummary {
                name: b.name.clone(),
                version: b.version.clone(),
                installed: b.exists && b.executable,
            })
            .collect();

        let version_changes = detect_version_changes(&binaries, history_dir);
        let hw_summary = hardware.map(|h| HardwareSummary {
            gpu: h.gpu.as_ref().map(|g| g.model.clone()),
            cuda: h.gpu.as_ref().and_then(|g| g.cuda_version.clone()),
            neon: h.cpu.neon,
            power_mode: h.power_mode.clone(),
            jetpack: h.jetpack.clone(),
        });
        #[allow(clippy::cast_possible_truncation)]
        let smoke_total = smoke.binaries.len() as u32;
        #[allow(clippy::cast_possible_truncation)]
        let smoke_passed = smoke
            .binaries
            .iter()
            .filter(|b| b.exists && b.executable && b.help_ok && b.version.is_some())
            .count() as u32;

        Self {
            schema_version: 1,
            date: started.format("%Y-%m-%d").to_string(),
            timestamp: now,
            cohete_version: env!("CARGO_PKG_VERSION").to_string(),
            pass,
            duration_s,
            tiers: TiersSummary {
                smoke: TierStatus::from_counts(
                    smoke.pass,
                    smoke_total,
                    smoke_passed,
                    smoke_total - smoke_passed,
                    0,
                ),
                hardware: hardware.map(|h| {
                    TierStatus::from_counts(h.pass, 1, u32::from(h.pass), u32::from(!h.pass), 0)
                }),
                functional: functional.map(|f| {
                    TierStatus::from_counts(f.pass, f.total, f.passed, f.failed, f.skipped)
                }),
                integration: integration.map(|i| {
                    TierStatus::from_counts(i.pass, i.total, i.passed, i.failed, i.skipped)
                }),
                performance: performance.map(|p| TierStatusPerf {
                    pass: p.pass,
                    regressions: p.regressions,
                }),
            },
            binaries,
            version_changes,
            hardware: hw_summary,
            metrics: performance.map(|p| PerformanceMetrics {
                inference_tok_s: p.metrics.inference_tok_s,
                whisper_rtf: p.metrics.whisper_rtf,
                rag_query_ms: p.metrics.rag_query_ms,
                memory_available_mb: p.metrics.memory_available_mb,
            }),
        }
    }
}

/// Compare current binary versions against yesterday's history file.
fn detect_version_changes(
    binaries: &[BinarySummary],
    history_dir: Option<&std::path::Path>,
) -> Vec<VersionChange> {
    let Some(dir) = history_dir else {
        return Vec::new();
    };
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let Ok(data) = std::fs::read_to_string(dir.join(format!("{yesterday}.json"))) else {
        return Vec::new();
    };
    let Ok(prev) = serde_json::from_str::<serde_json::Value>(&data) else {
        return Vec::new();
    };
    let Some(prev_bins) = prev.get("binaries").and_then(|b| b.as_array()) else {
        return Vec::new();
    };
    let mut changes = Vec::new();
    for bin in binaries {
        let Some(ref cur) = bin.version else { continue };
        let prev_ver = prev_bins
            .iter()
            .find(|b| b.get("name").and_then(|n| n.as_str()) == Some(&bin.name))
            .and_then(|b| b.get("version"))
            .and_then(|v| v.as_str());
        if let Some(p) = prev_ver {
            if p != cur {
                changes.push(VersionChange {
                    binary: bin.name.clone(),
                    from: p.to_string(),
                    to: cur.clone(),
                });
            }
        }
    }
    changes
}
