//! Tier 2: Hardware probes — GPU, CUDA, NEON, memory, power mode.
//!
//! Failures here are informational, they do not gate the run.

use crate::runner;
use crate::types::{CpuInfo, GpuInfo, HardwareResult, MemoryInfo};

pub fn run() -> HardwareResult {
    let gpu = probe_gpu();
    let vulkan = probe_vulkan();
    let cpu = probe_cpu();
    let memory = probe_memory();
    let disk_ok = probe_disk();
    let power_mode = probe_power_mode();
    let jetpack = probe_jetpack();

    // On jetson (aarch64): require NEON + 7GB. On x86_64: just check memory.
    let is_aarch64 = std::env::consts::ARCH == "aarch64";
    let pass = (!is_aarch64 || cpu.neon) && memory.total_mb >= 7000;

    HardwareResult {
        tier: 2,
        pass,
        gpu,
        vulkan,
        cpu,
        memory,
        disk_ok,
        power_mode,
        jetpack,
    }
}

fn probe_gpu() -> Option<GpuInfo> {
    let result = runner::run("nvidia-smi", &["--query-gpu=name", "--format=csv,noheader"]);
    if !result.success {
        eprintln!("  nvidia-smi not available: {}", result.stderr);
        return None;
    }

    let model = result.stdout.lines().next().unwrap_or("unknown").trim().to_string();

    let cuda_version = {
        let r = runner::shell("nvidia-smi | grep 'CUDA Version' | sed 's/.*CUDA Version: *//;s/ .*//'");
        if r.success && !r.stdout.is_empty() {
            Some(r.stdout.trim().to_string())
        } else {
            None
        }
    };

    eprintln!("  GPU: {model}, CUDA: {}", cuda_version.as_deref().unwrap_or("n/a"));

    Some(GpuInfo {
        model,
        cuda_version,
    })
}

fn probe_vulkan() -> Option<String> {
    let result = runner::shell("vulkaninfo --summary 2>&1 | grep -i 'deviceName' | head -1");
    if result.success && !result.stdout.is_empty() {
        let name = result.stdout.trim().to_string();
        eprintln!("  Vulkan: {name}");
        Some(name)
    } else {
        eprintln!("  Vulkan: not available");
        None
    }
}

fn probe_disk() -> bool {
    let result = runner::shell("df -BG /home | tail -1 | awk '{print $4}' | tr -d 'G'");
    let avail_gb = result.stdout.trim().parse::<u64>().unwrap_or(0);
    let ok = avail_gb >= 10;
    eprintln!("  Disk: {avail_gb} GB available (need >= 10)");
    ok
}

fn probe_cpu() -> CpuInfo {
    // ARMv7 reports "neon", ARMv8/aarch64 reports "asimd" (Advanced SIMD).
    // Both indicate NEON/SIMD support.
    let neon = runner::shell("grep -qE 'neon|asimd' /proc/cpuinfo && echo yes || echo no");
    let neon_available = neon.stdout.trim() == "yes";

    let cores_result = runner::shell("nproc");
    let cores = cores_result
        .stdout
        .trim()
        .parse()
        .unwrap_or(0);

    eprintln!("  CPU cores: {cores}, NEON: {neon_available}");

    CpuInfo {
        neon: neon_available,
        cores,
    }
}

fn probe_memory() -> MemoryInfo {
    let total = parse_meminfo("MemTotal");
    let available = parse_meminfo("MemAvailable");

    eprintln!("  Memory: {total} MB total, {available} MB available");

    MemoryInfo {
        total_mb: total,
        available_mb: available,
    }
}

fn parse_meminfo(field: &str) -> u64 {
    let result = runner::shell(&format!("grep {field} /proc/meminfo | awk '{{print $2}}'"));
    // /proc/meminfo reports kB
    result.stdout.trim().parse::<u64>().unwrap_or(0) / 1024
}

fn probe_power_mode() -> Option<String> {
    let result = runner::run("nvpmodel", &["-q"]);
    if result.success {
        // Extract mode name from output
        let mode = result
            .stdout
            .lines()
            .find(|l| l.contains("NV Power Mode"))
            .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string());
        if let Some(ref m) = mode {
            eprintln!("  Power mode: {m}");
        }
        mode
    } else {
        None
    }
}

fn probe_jetpack() -> Option<String> {
    let result = runner::shell("cat /etc/nv_tegra_release 2>/dev/null | head -1");
    if result.success && !result.stdout.is_empty() {
        let jp = result.stdout.trim().to_string();
        eprintln!("  JetPack: {jp}");
        Some(jp)
    } else {
        None
    }
}
