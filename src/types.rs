//! Shared types for cohete verification artifacts.

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Binary definition — what cohete expects to find installed.
pub struct BinaryDef {
    pub name: &'static str,
    pub path: &'static str,
}

/// All binaries in the matrix.
pub const BINARIES: &[BinaryDef] = &[
    BinaryDef { name: "apr", path: "/home/noah/.cargo/bin/apr" },
    BinaryDef { name: "whisper-apr", path: "/home/noah/.cargo/bin/whisper-apr" },
    BinaryDef { name: "trueno-rag", path: "/home/noah/.cargo/bin/trueno-rag" },
    BinaryDef { name: "forjar", path: "/home/noah/.cargo/bin/forjar" },
    BinaryDef { name: "pmat", path: "/home/noah/.cargo/bin/pmat" },
    BinaryDef { name: "copia", path: "/home/noah/.cargo/bin/copia" },
    BinaryDef { name: "pzsh", path: "/home/noah/.cargo/bin/pzsh" },
    BinaryDef { name: "batuta", path: "/home/noah/.cargo/bin/batuta" },
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
    pub date: String,
    pub started: DateTime<Utc>,
    pub duration_s: u64,
    pub pass: bool,
    pub tiers: TiersSummary,
    pub binaries: Vec<BinarySummary>,
}

#[derive(Serialize)]
pub struct TiersSummary {
    pub smoke: TierStatus,
    pub hardware: Option<TierStatus>,
    pub functional: Option<TierStatus>,
    pub integration: Option<TierStatus>,
    pub performance: Option<TierStatus>,
}

#[derive(Serialize)]
pub struct TierStatus {
    pub pass: bool,
}

#[derive(Serialize)]
pub struct BinarySummary {
    pub name: String,
    pub version: Option<String>,
    pub installed: bool,
}

impl Summary {
    pub fn new(
        started: DateTime<Utc>,
        pass: bool,
        smoke: &SmokeResult,
        hardware: Option<&HardwareResult>,
        functional: Option<&FunctionalResult>,
        integration: Option<&IntegrationResult>,
        performance: Option<&PerformanceResult>,
    ) -> Self {
        let now = Utc::now();
        let duration_s = (now - started).num_seconds().unsigned_abs();

        let binaries = smoke
            .binaries
            .iter()
            .map(|b| BinarySummary {
                name: b.name.clone(),
                version: b.version.clone(),
                installed: b.exists && b.executable,
            })
            .collect();

        Self {
            date: started.format("%Y-%m-%d").to_string(),
            started,
            duration_s,
            pass,
            tiers: TiersSummary {
                smoke: TierStatus { pass: smoke.pass },
                hardware: hardware.map(|h| TierStatus { pass: h.pass }),
                functional: functional.map(|f| TierStatus { pass: f.pass }),
                integration: integration.map(|i| TierStatus { pass: i.pass }),
                performance: performance.map(|p| TierStatus { pass: p.pass }),
            },
            binaries,
        }
    }
}
