//! Shared types for cohete verification artifacts.

use chrono::{DateTime, Utc};
use serde::Serialize;

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
fn which(name: &str) -> Option<String> {
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
    BinaryDef { name: "apr", preferred_path: "/home/noah/.cargo/bin/apr" },
    BinaryDef { name: "whisper-apr", preferred_path: "/home/noah/.cargo/bin/whisper-apr" },
    BinaryDef { name: "trueno-rag", preferred_path: "/home/noah/.cargo/bin/trueno-rag" },
    BinaryDef { name: "forjar", preferred_path: "/home/noah/.cargo/bin/forjar" },
    BinaryDef { name: "pmat", preferred_path: "/home/noah/.cargo/bin/pmat" },
    BinaryDef { name: "copia", preferred_path: "/home/noah/.cargo/bin/copia" },
    BinaryDef { name: "pzsh", preferred_path: "/home/noah/.cargo/bin/pzsh" },
    BinaryDef { name: "batuta", preferred_path: "/home/noah/.cargo/bin/batuta" },
];

pub const MODEL_PATH: &str = "/home/noah/data/models/canary/qwen-1.5b-q4k.apr";
pub const WHISPER_MODEL_PATH: &str = "/home/noah/data/models/canary/whisper-tiny.en";
pub const TEST_AUDIO_PATH: &str = "/home/noah/data/models/canary/test-2s.wav";

// ─── Tier 1: Smoke ──────────────────────────────────────────

#[derive(Serialize)]
pub struct SmokeResult {
    pub tier: u8,
    pub pass: bool,
    pub binaries: Vec<BinarySmoke>,
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
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
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
    pub functional: Option<TierStatusCounts>,
    pub integration: Option<TierStatusCounts>,
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

#[derive(Serialize)]
pub struct TierStatusCounts {
    pub pass: bool,
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
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
        });

        #[allow(clippy::cast_possible_truncation)]
        let smoke_total = smoke.binaries.len() as u32;
        #[allow(clippy::cast_possible_truncation)]
        let smoke_passed = smoke.binaries.iter().filter(|b| b.exists && b.executable && b.help_ok && b.version.is_some()).count() as u32;

        Self {
            schema_version: 1,
            date: started.format("%Y-%m-%d").to_string(),
            timestamp: now,
            cohete_version: env!("CARGO_PKG_VERSION").to_string(),
            pass,
            duration_s,
            tiers: TiersSummary {
                smoke: TierStatus {
                    pass: smoke.pass,
                    total: smoke_total,
                    passed: smoke_passed,
                    failed: smoke_total - smoke_passed,
                    skipped: 0,
                },
                hardware: hardware.map(|h| TierStatus {
                    pass: h.pass,
                    total: 1,
                    passed: u32::from(h.pass),
                    failed: u32::from(!h.pass),
                    skipped: 0,
                }),
                functional: functional.map(|f| TierStatusCounts {
                    pass: f.pass,
                    total: f.total,
                    passed: f.passed,
                    failed: f.failed,
                    skipped: f.skipped,
                }),
                integration: integration.map(|i| TierStatusCounts {
                    pass: i.pass,
                    total: i.total,
                    passed: i.passed,
                    failed: i.failed,
                    skipped: i.skipped,
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

    // Find yesterday's file
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();
    let yesterday_file = dir.join(format!("{yesterday}.json"));

    let Ok(data) = std::fs::read_to_string(&yesterday_file) else {
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
        let Some(ref current_ver) = bin.version else { continue };
        let prev_ver = prev_bins
            .iter()
            .find(|b| b.get("name").and_then(|n| n.as_str()) == Some(&bin.name))
            .and_then(|b| b.get("version"))
            .and_then(|v| v.as_str());

        if let Some(prev) = prev_ver {
            if prev != current_ver {
                changes.push(VersionChange {
                    binary: bin.name.clone(),
                    from: prev.to_string(),
                    to: current_ver.clone(),
                });
            }
        }
    }
    changes
}
