# Cohete: NVIDIA Jetson Edge Hardware Integration for Sovereign AI

**Sovereign AI Stack Component**

| Field | Value |
|-------|-------|
| **Crate** | `cohete` |
| **Version** | 0.1.0 |
| **Location** | `paiml/cohete` |
| **Status** | SPECIFICATION |
| **Philosophy** | *"A theory that explains everything, explains nothing."* — Karl Popper |
| **Governance** | Toyota Way + Scientific Method |

---

## Part I: Overview

### 1. Executive Summary

Cohete (Spanish: "rocket") provides **safe, pure Rust interfaces** to NVIDIA Jetson edge hardware for the Sovereign AI Stack. It enables edge deployment, thermal-aware inference, and fleet orchestration for constrained ARM64 devices with integrated GPUs.

### 1.1 Problem Statement

Edge AI deployment on NVIDIA Jetson devices faces critical challenges:

1. **Memory Constraints**: 8GB unified memory requires aggressive quantization and budget management
2. **Thermal Throttling**: Sustained inference causes thermal throttling without proactive monitoring
3. **ARM NEON Dispatch**: Existing compute libraries lack Jetson-optimized ARM SIMD paths
4. **Fleet Management**: No unified tooling for multi-device orchestration
5. **Provisioning Gap**: Manual setup via serial/SSH lacks automation

### 1.2 Solution

Cohete integrates with the Sovereign AI Stack to provide:

| Capability | Implementation | Stack Integration |
|------------|----------------|-------------------|
| Device Discovery | USB CDC, mDNS, SSH | `repartir` remote executor |
| Thermal Monitoring | tegrastats parsing | `renacer` tracing |
| Memory Budgeting | VRAM tracking, auto-quantization | `pacha` model registry |
| ARM NEON Compute | trueno backend selection | `trueno` SIMD dispatch |
| Fleet Orchestration | Multi-device scheduling | `batuta` orchestration |
| Provisioning | Declarative YAML config | Zero-code setup |

---

## Part II: Supported Hardware

### 2.1 NVIDIA Jetson Family

| Device | Module | Memory | GPU | TOPS | Status |
|--------|--------|--------|-----|------|--------|
| Jetson Orin Nano 4GB | `orin_nano` | 4GB LPDDR5 | 512 CUDA | 20 | Primary |
| Jetson Orin Nano 8GB | `orin_nano` | 8GB LPDDR5 | 1024 CUDA | 40 | Primary |
| Jetson Orin NX 8GB | `orin_nx` | 8GB LPDDR5 | 1024 CUDA | 70 | Supported |
| Jetson Orin NX 16GB | `orin_nx` | 16GB LPDDR5 | 1024 CUDA | 100 | Supported |
| Jetson AGX Orin 32GB | `agx_orin` | 32GB LPDDR5 | 2048 CUDA | 200 | Planned |
| Jetson AGX Orin 64GB | `agx_orin` | 64GB LPDDR5 | 2048 CUDA | 275 | Planned |

*References: [1] NVIDIA Jetson Orin Nano Developer Kit, [2] NVIDIA Jetson Modules Datasheet*

### 2.2 Hardware Subsystems

| Subsystem | Module | Capability | API |
|-----------|--------|------------|-----|
| Tegra Stats | `tegra` | CPU/GPU/memory monitoring | `TegraMonitor` |
| Power Modes | `nvpmodel` | MAXN/15W/7W selection | `PowerMode` |
| Jetson Clocks | `clocks` | Frequency control | `JetsonClocks` |
| Thermal Zones | `thermal` | Temperature monitoring | `ThermalZone` |
| NVMe Storage | `storage` | SSD management | `NvmeDevice` |
| USB CDC | `usb` | Serial console access | `UsbCdc` |
| CUDA Restricted | `cuda` | Limited GPU compute | `CudaDevice` |

### 2.3 ARM Architecture

| Feature | Support | trueno Integration |
|---------|---------|-------------------|
| ARMv8.2-A | Native | CPU detection |
| NEON SIMD | Full | `trueno/neon` backend |
| ARM SVE | Partial | Future |
| Unified Memory | Native | Zero-copy tensors |

*Reference: [3] ARM Cortex-A78AE Technical Reference Manual*

---

## Part III: Integration with Sovereign AI Stack

### 3.1 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         BATUTA ORCHESTRATION                                 │
│                                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────────────────┐  │
│  │ realizar │  │ repartir │  │  pacha   │  │          cohete            │  │
│  │ (infer)  │  │ (sched)  │  │ (models) │  │  (Jetson edge hardware)    │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┬─────────────────┘  │
│       │             │             │                   │                    │
│       └─────────────┴─────────────┴───────────────────┘                    │
│                           │                                                │
│                    ┌──────▼──────┐                                         │
│                    │   trueno    │                                         │
│                    │ ARM NEON    │                                         │
│                    └─────────────┘                                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   Jetson Fleet        │
                    │  ┌─────┐ ┌─────┐      │
                    │  │ J1  │ │ J2  │ ...  │
                    │  └─────┘ └─────┘      │
                    └───────────────────────┘
```

### 3.2 Integration Points

#### 3.2.1 trueno (Compute Foundation)

```rust
// cohete selects ARM NEON backend via trueno
use trueno::{Vector, Backend, BackendSelector};
use cohete::device::JetsonDevice;

let device = JetsonDevice::discover_local()?;
let selector = BackendSelector::new()
    .with_device_hint(device.compute_hint())  // ARM NEON preference
    .with_memory_budget(device.available_memory());

let backend = selector.select_for_size(matrix_size)?;
// Returns Backend::Neon for Jetson, Backend::Avx2 for x86
```

*Integration: trueno v0.11+ with `neon` feature*

#### 3.2.2 repartir (Distributed Compute)

```rust
// cohete provides JetsonExecutor for repartir pools
use repartir::{Pool, Executor};
use cohete::fleet::{JetsonExecutor, ThermalPolicy};

let pool = Pool::builder()
    .add_executor(JetsonExecutor::new("192.168.55.1")
        .with_thermal_policy(ThermalPolicy::Conservative)  // Pause at 70°C
        .with_memory_budget_mb(6000))  // Reserve 2GB for system
    .build()?;
```

*Integration: repartir v2.0+ with `remote-jetson` feature*

#### 3.2.3 pacha (Model Registry)

```rust
// cohete auto-quantizes models for Jetson memory constraints
use pacha::{ModelRegistry, Model};
use cohete::quantize::{JetsonQuantizer, QuantLevel};

let registry = ModelRegistry::connect("https://models.sovereign.ai")?;
let model = registry.pull("llama2-7b")?;

// Auto-quantize for 8GB Jetson
let quantized = JetsonQuantizer::new(QuantLevel::Q4_0)
    .with_target_memory_mb(6000)
    .quantize(&model)?;

// Deploy to device
device.deploy(quantized)?;
```

*Integration: pacha v0.2+ with Ed25519 model signatures*

#### 3.2.4 realizar (Inference Engine)

```rust
// cohete runs realizar inference server on-device
use realizar::{InferenceServer, Config};
use cohete::device::JetsonDevice;

let device = JetsonDevice::connect("192.168.55.1")?;
let config = Config::for_jetson(&device)
    .with_max_batch_size(4)  // Memory-constrained batching
    .with_context_length(2048);  // Reduced context for 8GB

device.run_inference_server(config).await?;
```

*Integration: realizar v0.5+ with `jetson` feature*

#### 3.2.5 renacer (Tracing)

```rust
// cohete exports thermal/memory traces via renacer
use renacer::{Span, Tracer};
use cohete::monitor::TegraMonitor;

let tracer = Tracer::new("cohete");
let monitor = TegraMonitor::new()?;

// Thermal trace with Lamport timestamps
let span = tracer.span("inference_batch");
let stats = monitor.sample()?;
span.record("gpu_temp_c", stats.gpu_temp);
span.record("memory_used_mb", stats.memory_used_mb);
```

*Integration: renacer v0.9+ spans*

---

## Part IV: Public API

### 4.1 Core Types

```rust
// Device discovery and management
pub use cohete::device::{
    JetsonDevice,       // Main device handle
    DeviceInfo,         // Hardware info (model, memory, CUDA cores)
    ConnectionMethod,   // USB, Ethernet, mDNS
};

// Thermal monitoring
pub use cohete::thermal::{
    TegraMonitor,       // tegrastats interface
    TegraStats,         // CPU/GPU/memory snapshot
    ThermalZone,        // Temperature sensor
    ThermalPolicy,      // Conservative/Aggressive/Custom
};

// Power management
pub use cohete::power::{
    PowerMode,          // MAXN, 15W, 7W modes
    JetsonClocks,       // Frequency control
    PowerProfile,       // Custom power profiles
};

// Storage management
pub use cohete::storage::{
    NvmeDevice,         // NVMe SSD management
    SwapConfig,         // Swap file configuration
    StorageLayout,      // Mount points, partitions
};

// Fleet orchestration
pub use cohete::fleet::{
    Fleet,              // Multi-device manager
    JetsonExecutor,     // repartir executor
    DeploymentConfig,   // Model deployment settings
};

// Provisioning
pub use cohete::provision::{
    ProvisionConfig,    // YAML-based configuration
    SetupWizard,        // Interactive setup
    SshConfig,          // SSH key management
};

// Model quantization
pub use cohete::quantize::{
    JetsonQuantizer,    // Memory-aware quantization
    QuantLevel,         // Q4_0, Q4_1, Q5_0, Q5_1, Q8_0
    QuantResult,        // Quantization metrics
};

// Error handling
pub use cohete::{Error, Result, Subsystem};
```

### 4.2 Example: Complete Edge Deployment

```rust
use cohete::{
    device::JetsonDevice,
    thermal::{TegraMonitor, ThermalPolicy},
    quantize::{JetsonQuantizer, QuantLevel},
    fleet::Fleet,
};
use pacha::ModelRegistry;
use realizar::InferenceServer;

async fn deploy_sovereign_edge() -> cohete::Result<()> {
    // 1. Discover Jetson devices on network
    let devices = JetsonDevice::discover_all().await?;
    println!("Found {} Jetson devices", devices.len());

    // 2. Connect to model registry
    let registry = ModelRegistry::connect("https://models.sovereign.ai")?;
    let model = registry.pull("mistral-7b-instruct")?;

    // 3. Create fleet with thermal awareness
    let mut fleet = Fleet::new();
    for device in devices {
        // Check thermal headroom before adding
        let monitor = TegraMonitor::connect(&device)?;
        let stats = monitor.sample()?;

        if stats.gpu_temp < 65.0 && stats.available_memory_mb > 4000 {
            fleet.add_device(device, ThermalPolicy::Conservative)?;
        }
    }

    // 4. Quantize model for Jetson constraints
    let quantized = JetsonQuantizer::new(QuantLevel::Q4_0)
        .with_target_memory_mb(6000)
        .quantize(&model)?;

    // 5. Deploy to fleet
    fleet.deploy_model(quantized).await?;

    // 6. Start inference servers
    fleet.start_inference_servers().await?;

    Ok(())
}
```

---

## Part V: Safety Architecture

### 5.1 Memory Safety Model

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PUBLIC API (100% Safe Rust)                            │
│  #![deny(unsafe_code)]                                                      │
│                                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │ JetsonDevice│ │TegraMonitor │ │ NvmeDevice  │ │   JetsonQuantizer   │   │
│  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────────┬──────────┘   │
├─────────┼───────────────┼───────────────┼───────────────────┼──────────────┤
│         │    SAFE BOUNDARY (Poka-Yoke)  │                   │              │
│         ▼               ▼               ▼                   ▼              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    FFI QUARANTINE ZONE                              │   │
│  │  #![allow(unsafe_code)] — Audited, MIRI-verified                    │   │
│  │  src/ffi/tegra.rs | src/ffi/nvml.rs | src/ffi/cuda.rs              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│                         JETSON LINUX / CUDA                                 │
│  tegrastats | nvpmodel | jetson_clocks | CUDA Runtime (restricted)          │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Thermal Safety (Jidoka)

```rust
/// Thermal circuit breaker - automatically stops inference
/// when temperature exceeds safe threshold
pub struct ThermalCircuitBreaker {
    threshold_c: f32,      // Default: 70°C
    cooldown_c: f32,       // Resume at: 60°C
    check_interval_ms: u64, // Default: 1000ms
}

impl ThermalCircuitBreaker {
    /// Jidoka: Stop the line when thermal threshold exceeded
    pub async fn guard<F, T>(&self, work: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        loop {
            let temp = self.monitor.gpu_temp()?;
            if temp > self.threshold_c {
                tracing::warn!(
                    temp_c = temp,
                    threshold_c = self.threshold_c,
                    "Thermal circuit breaker OPEN - pausing work"
                );
                self.wait_for_cooldown().await?;
            } else {
                break;
            }
        }
        work.await
    }
}
```

### 5.3 Memory Budget Safety (Poka-Yoke)

```rust
/// Memory budget enforcer - prevents OOM on constrained devices
pub struct MemoryBudget {
    total_mb: u64,
    reserved_mb: u64,  // System reserve (default: 2GB)
    allocated: AtomicU64,
}

impl MemoryBudget {
    /// Poka-Yoke: Prevent allocation that would exceed budget
    pub fn try_allocate(&self, size_mb: u64) -> Result<MemoryGuard> {
        let available = self.total_mb - self.reserved_mb - self.allocated.load(Ordering::Acquire);
        if size_mb > available {
            return Err(Error::InsufficientMemory {
                requested_mb: size_mb,
                available_mb: available,
            });
        }
        self.allocated.fetch_add(size_mb, Ordering::Release);
        Ok(MemoryGuard { budget: self, size_mb })
    }
}
```

---

## Part VI: Declarative Configuration (Architectural Invariant)

### 6.1 YAML-First Design

Per Sovereign AI Stack requirements, cohete provides **full functionality via declarative YAML** without writing code:

```yaml
# cohete.yaml - Complete edge deployment configuration
version: "1.0"

# Device discovery
discovery:
  methods:
    - usb      # USB-C direct connection
    - mdns     # mDNS on local network
    - static:  # Static IP list
        - 192.168.1.100
        - 192.168.1.101

# Fleet configuration
fleet:
  name: "sovereign-edge-cluster"
  devices:
    - id: jetson-01
      connection: usb
      thermal_policy: conservative
      memory_budget_mb: 6000
    - id: jetson-02
      connection: ethernet
      ip: 192.168.1.100
      thermal_policy: aggressive
      memory_budget_mb: 7000

# Model deployment
models:
  - name: mistral-7b-instruct
    source: pacha://models.sovereign.ai/mistral-7b
    quantization: q4_0
    devices: all  # Deploy to all fleet devices

# Thermal policies
thermal:
  conservative:
    threshold_c: 65
    cooldown_c: 55
    check_interval_ms: 500
  aggressive:
    threshold_c: 75
    cooldown_c: 65
    check_interval_ms: 1000

# Inference server
inference:
  port: 8080
  max_batch_size: 4
  context_length: 2048
  api_compatibility: openai  # OpenAI-compatible API

# Provisioning (initial setup)
provision:
  nvme:
    enabled: true
    mount_point: /mnt/nvme
    swap_size_gb: 16
  ssh:
    copy_id: true
    config_host: jetson
  packages:
    - nvtop
    - htop
```

### 6.2 CLI Usage

```bash
# Discover devices
cohete discover

# Deploy from YAML config
cohete deploy --config cohete.yaml

# Monitor fleet
cohete status --fleet

# Interactive setup (generates YAML)
cohete setup --interactive
```

---

## Part VII: 100-Point Popperian Falsification Checklist

### Nullification Protocol

For each claim:
- **Claim:** Assertion made by cohete
- **Falsification Test:** Experiment to attempt disproof (*Genchi Genbutsu*)
- **Null Hypothesis (H₀):** Assumed true until disproven
- **Rejection Criteria:** Conditions that falsify claim (*Jidoka trigger*)
- **TPS Principle:** Toyota Way mapping
- **Evidence Required:** Data for evaluation

### Severity Levels

| Level | Impact | TPS Response |
|-------|--------|--------------|
| **Critical** | Invalidates core claims | Stop the Line (Andon), retract claim |
| **Major** | Significantly weakens validity | Kaizen required, revise with caveats |
| **Minor** | Edge case/boundary | Document limitation |
| **Informational** | Clarification needed | Update documentation |

---

## Section 1: Device Discovery & Connection [15 Items]

### DDC-01: USB CDC Device Detection

**Claim:** cohete detects Jetson devices connected via USB-C within 5 seconds.

**Falsification Test:**
```bash
cargo test --package cohete --test usb_discovery
```

**Null Hypothesis:** USB detection fails or exceeds 5 seconds.

**Rejection Criteria:**
- Device not detected within 5 seconds
- False positive detection of non-Jetson USB devices

**TPS Principle:** Genchi Genbutsu — verify at the source

**Evidence Required:**
- [ ] USB enumeration timing measurements
- [ ] Device descriptor validation
- [ ] libusb integration tests

*Reference: [4] USB CDC ACM Class Specification*

---

### DDC-02: mDNS Service Discovery

**Claim:** cohete discovers Jetson devices on local network via mDNS.

**Falsification Test:**
```bash
cargo test --package cohete --test mdns_discovery
```

**Null Hypothesis:** mDNS discovery fails.

**Rejection Criteria:**
- Jetson with Avahi not discovered
- Discovery takes >10 seconds

**TPS Principle:** Jidoka — automatic detection

**Evidence Required:**
- [ ] mDNS query/response capture
- [ ] Service type registration (`_jetson._tcp`)
- [ ] Cross-subnet behavior documentation

*Reference: [5] RFC 6762 Multicast DNS*

---

### DDC-03: SSH Connection Establishment

**Claim:** cohete establishes SSH connection with key-based authentication.

**Falsification Test:**
```bash
cargo test --package cohete --test ssh_connection
```

**Null Hypothesis:** SSH connection fails.

**Rejection Criteria:**
- Connection timeout >30 seconds
- Key rejection without clear error

**TPS Principle:** Poka-Yoke — prevent auth errors

**Evidence Required:**
- [ ] SSH handshake timing
- [ ] Key format compatibility
- [ ] Error message clarity

*Reference: [6] RFC 4252 SSH Authentication Protocol*

---

### DDC-04: Serial Console Access

**Claim:** cohete provides serial console access via USB CDC.

**Falsification Test:**
```bash
cargo test --package cohete --test serial_console
```

**Null Hypothesis:** Serial console fails.

**Rejection Criteria:**
- Baud rate mismatch (115200)
- Character encoding issues

**TPS Principle:** Genchi Genbutsu — direct hardware access

**Evidence Required:**
- [ ] Serial terminal echo test
- [ ] Multi-platform testing (Linux/macOS/Windows)
- [ ] Flow control verification

---

### DDC-05: Device Model Identification

**Claim:** cohete correctly identifies Jetson model variant.

**Falsification Test:**
```bash
cargo test --package cohete --test model_identification
```

**Null Hypothesis:** Model misidentified.

**Rejection Criteria:**
- Orin Nano 4GB identified as 8GB
- NX identified as Nano

**TPS Principle:** Genchi Genbutsu — verify hardware

**Evidence Required:**
- [ ] `/etc/nv_tegra_release` parsing
- [ ] Memory size verification
- [ ] CUDA core count validation

*Reference: [1] NVIDIA Jetson Orin Nano Developer Kit*

---

### DDC-06: Network Interface Detection

**Claim:** cohete detects all network interfaces (USB, Ethernet, WiFi).

**Falsification Test:**
```bash
cargo test --package cohete --test network_detection
```

**Null Hypothesis:** Interface missed.

**Rejection Criteria:**
- USB gadget interface (192.168.55.1) not detected
- Ethernet IP not discovered

**TPS Principle:** Complete visibility

**Evidence Required:**
- [ ] Interface enumeration
- [ ] IP address extraction
- [ ] Link state detection

---

### DDC-07: Connection Health Monitoring

**Claim:** cohete detects connection loss within 10 seconds.

**Falsification Test:**
```bash
cargo test --package cohete --test connection_health
```

**Null Hypothesis:** Connection loss undetected.

**Rejection Criteria:**
- Detection latency >10 seconds
- False positive disconnection

**TPS Principle:** Jidoka — automatic problem detection

**Evidence Required:**
- [ ] Heartbeat mechanism
- [ ] Reconnection logic
- [ ] Network partition handling

---

### DDC-08: Multi-Device Enumeration

**Claim:** cohete correctly enumerates multiple Jetson devices.

**Falsification Test:**
```bash
cargo test --package cohete --test multi_device
```

**Null Hypothesis:** Device count incorrect.

**Rejection Criteria:**
- Duplicate device entries
- Device ordering inconsistent

**TPS Principle:** Fleet visibility

**Evidence Required:**
- [ ] Unique device ID generation
- [ ] Concurrent discovery
- [ ] Race condition handling

---

### DDC-09: Device Persistence

**Claim:** cohete persists device configuration across restarts.

**Falsification Test:**
```bash
cargo test --package cohete --test device_persistence
```

**Null Hypothesis:** Configuration lost.

**Rejection Criteria:**
- SSH config not saved
- IP addresses not cached

**TPS Principle:** Configuration management

**Evidence Required:**
- [ ] Config file format
- [ ] File permission handling
- [ ] Migration between versions

---

### DDC-10: Connection Security

**Claim:** cohete uses secure connections (SSH, TLS) by default.

**Falsification Test:**
```bash
cargo test --package cohete --test connection_security
```

**Null Hypothesis:** Insecure connection possible.

**Rejection Criteria:**
- Plaintext credentials transmitted
- Man-in-the-middle possible

**TPS Principle:** Security by default

**Evidence Required:**
- [ ] SSH host key verification
- [ ] Certificate pinning (if applicable)
- [ ] Credential storage security

*Reference: [7] NIST SP 800-52 TLS Guidelines*

---

### DDC-11: Firewall Compatibility

**Claim:** cohete works through standard firewall configurations.

**Falsification Test:**
```bash
cargo test --package cohete --test firewall_compatibility
```

**Null Hypothesis:** Firewall blocks required traffic.

**Rejection Criteria:**
- Non-standard ports required
- UDP multicast blocked

**TPS Principle:** Enterprise compatibility

**Evidence Required:**
- [ ] Port usage documentation
- [ ] Corporate firewall testing
- [ ] NAT traversal behavior

---

### DDC-12: IPv6 Support

**Claim:** cohete supports IPv6 device discovery and connection.

**Falsification Test:**
```bash
cargo test --package cohete --test ipv6_support
```

**Null Hypothesis:** IPv6 not supported.

**Rejection Criteria:**
- IPv6-only network fails
- Link-local address handling broken

**TPS Principle:** Future-proofing

**Evidence Required:**
- [ ] IPv6 address parsing
- [ ] Dual-stack behavior
- [ ] IPv6-only testing

---

### DDC-13: Connection Timeout Configuration

**Claim:** cohete allows configurable connection timeouts.

**Falsification Test:**
```bash
cargo test --package cohete --test timeout_config
```

**Null Hypothesis:** Timeout not configurable.

**Rejection Criteria:**
- Hardcoded timeout values
- Invalid timeout accepted

**TPS Principle:** Configurability

**Evidence Required:**
- [ ] YAML config validation
- [ ] Range checking
- [ ] Default documentation

---

### DDC-14: Proxy Support

**Claim:** cohete supports SSH proxy (jump host) connections.

**Falsification Test:**
```bash
cargo test --package cohete --test proxy_support
```

**Null Hypothesis:** Proxy not supported.

**Rejection Criteria:**
- ProxyJump directive ignored
- Bastion host connection fails

**TPS Principle:** Enterprise deployment

**Evidence Required:**
- [ ] SSH config parsing
- [ ] Multi-hop verification
- [ ] Authentication forwarding

---

### DDC-15: Device State Caching

**Claim:** cohete caches device state to minimize network queries.

**Falsification Test:**
```bash
cargo test --package cohete --test state_caching
```

**Null Hypothesis:** Excessive network queries.

**Rejection Criteria:**
- Query per property access
- Stale cache served

**TPS Principle:** Muda (Waiting) reduction

**Evidence Required:**
- [ ] Cache invalidation logic
- [ ] TTL configuration
- [ ] Freshness indicators

---

## Section 2: Thermal Monitoring & Management [15 Items]

### TM-01: tegrastats Parsing Accuracy

**Claim:** cohete correctly parses all tegrastats fields.

**Falsification Test:**
```bash
cargo test --package cohete --test tegrastats_parsing
```

**Null Hypothesis:** Parsing errors exist.

**Rejection Criteria:**
- CPU/GPU temperature incorrect
- Memory usage incorrect

**TPS Principle:** Genchi Genbutsu — accurate measurement

**Evidence Required:**
- [ ] Golden file testing
- [ ] All JetPack versions
- [ ] Edge case handling

*Reference: [8] NVIDIA L4T Developer Guide - tegrastats*

---

### TM-02: Temperature Accuracy (±1°C)

**Claim:** cohete reports temperature within ±1°C of actual.

**Falsification Test:**
```bash
cargo test --package cohete --test temperature_accuracy
```

**Null Hypothesis:** Temperature reading inaccurate.

**Rejection Criteria:**
- Deviation >1°C from thermal sensor
- Stale readings

**TPS Principle:** Measurement precision

**Evidence Required:**
- [ ] External thermometer comparison
- [ ] Sampling frequency verification
- [ ] Thermal zone correlation

---

### TM-03: Thermal Circuit Breaker Activation

**Claim:** cohete stops inference when temperature exceeds threshold.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_circuit_breaker
```

**Null Hypothesis:** Circuit breaker fails to activate.

**Rejection Criteria:**
- Inference continues above threshold
- Activation latency >2 seconds

**TPS Principle:** Jidoka — automatic stop

**Evidence Required:**
- [ ] Stress test under thermal load
- [ ] Activation timing measurement
- [ ] State transition verification

---

### TM-04: Thermal Cooldown Behavior

**Claim:** cohete resumes work only after cooldown threshold reached.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_cooldown
```

**Null Hypothesis:** Premature work resumption.

**Rejection Criteria:**
- Resume before cooldown temperature
- Hysteresis not enforced

**TPS Principle:** Stable operation

**Evidence Required:**
- [ ] Temperature trend tracking
- [ ] Hysteresis verification
- [ ] Resume timing

---

### TM-05: Multi-Zone Thermal Monitoring

**Claim:** cohete monitors all thermal zones (CPU, GPU, SOC).

**Falsification Test:**
```bash
cargo test --package cohete --test multi_zone_thermal
```

**Null Hypothesis:** Zone missed.

**Rejection Criteria:**
- GPU thermal zone not monitored
- SOC junction temperature missed

**TPS Principle:** Complete visibility

**Evidence Required:**
- [ ] Zone enumeration
- [ ] Cross-zone correlation
- [ ] Hottest zone identification

---

### TM-06: Thermal Event Tracing

**Claim:** cohete exports thermal events via renacer tracing.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_tracing
```

**Null Hypothesis:** Events not traced.

**Rejection Criteria:**
- Thermal spike not logged
- Missing Lamport timestamp

**TPS Principle:** Auditability

**Evidence Required:**
- [ ] renacer span verification
- [ ] Event correlation
- [ ] Trace export format

---

### TM-07: Fan Control Integration

**Claim:** cohete can control PWM fan speed.

**Falsification Test:**
```bash
cargo test --package cohete --test fan_control
```

**Null Hypothesis:** Fan control fails.

**Rejection Criteria:**
- PWM write rejected
- Speed not affected

**TPS Principle:** Active thermal management

**Evidence Required:**
- [ ] PWM file access
- [ ] Speed verification
- [ ] Permission handling

---

### TM-08: Thermal Policy Customization

**Claim:** cohete supports custom thermal policies via YAML.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_policy_yaml
```

**Null Hypothesis:** Custom policy ignored.

**Rejection Criteria:**
- YAML threshold not applied
- Invalid policy accepted

**TPS Principle:** Configurability

**Evidence Required:**
- [ ] YAML schema validation
- [ ] Policy application verification
- [ ] Error message clarity

---

### TM-09: Thermal Prediction

**Claim:** cohete predicts thermal runaway before threshold breach.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_prediction
```

**Null Hypothesis:** Prediction inaccurate.

**Rejection Criteria:**
- False negative (missed runaway)
- >50% false positive rate

**TPS Principle:** Proactive safety

**Evidence Required:**
- [ ] Prediction model validation
- [ ] Lead time measurement
- [ ] Accuracy metrics

*Reference: [9] Wang et al. "Thermal Prediction for Embedded Systems"*

---

### TM-10: Sustained Load Thermal Stability

**Claim:** cohete maintains stable temperature under sustained inference load.

**Falsification Test:**
```bash
cargo test --package cohete --test sustained_thermal
```

**Null Hypothesis:** Temperature unstable or trending upward indefinitely.

**Rejection Criteria:**
- Thermal runaway under normal load
- Oscillation >5°C

**TPS Principle:** Heijunka — load leveling

**Evidence Required:**
- [ ] 1-hour sustained load test
- [ ] Temperature time series
- [ ] Throttling events logged

---

### TM-11: Power Mode Thermal Correlation

**Claim:** cohete adjusts thermal policy based on power mode.

**Falsification Test:**
```bash
cargo test --package cohete --test power_thermal_correlation
```

**Null Hypothesis:** Power mode ignored in thermal policy.

**Rejection Criteria:**
- Same threshold for 7W and MAXN
- Power consumption not tracked

**TPS Principle:** Context-aware management

**Evidence Required:**
- [ ] Mode-specific thresholds
- [ ] Power consumption measurement
- [ ] Thermal margin calculation

---

### TM-12: Thermal Alert Notification

**Claim:** cohete emits alerts on thermal threshold approach.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_alerts
```

**Null Hypothesis:** No advance warning.

**Rejection Criteria:**
- Alert only after threshold breach
- No configurable warning level

**TPS Principle:** Andon — visual/audible warning

**Evidence Required:**
- [ ] Warning threshold configuration
- [ ] Alert delivery mechanism
- [ ] Escalation levels

---

### TM-13: Thermal Throttling Detection

**Claim:** cohete detects when Jetson is thermally throttled.

**Falsification Test:**
```bash
cargo test --package cohete --test throttling_detection
```

**Null Hypothesis:** Throttling undetected.

**Rejection Criteria:**
- CPU/GPU frequency reduction not reported
- Throttling cause not identified

**TPS Principle:** Problem visibility

**Evidence Required:**
- [ ] Frequency monitoring
- [ ] Throttling state parsing
- [ ] Cause attribution

---

### TM-14: Ambient Temperature Compensation

**Claim:** cohete adjusts thresholds based on ambient temperature delta.

**Falsification Test:**
```bash
cargo test --package cohete --test ambient_compensation
```

**Null Hypothesis:** Ambient temperature ignored.

**Rejection Criteria:**
- Same threshold for 20°C and 35°C ambient
- No ambient sensing

**TPS Principle:** Environmental awareness

**Evidence Required:**
- [ ] Ambient measurement method
- [ ] Threshold adjustment algorithm
- [ ] Operating range documentation

---

### TM-15: Thermal History Logging

**Claim:** cohete maintains thermal history for analysis.

**Falsification Test:**
```bash
cargo test --package cohete --test thermal_history
```

**Null Hypothesis:** History not maintained.

**Rejection Criteria:**
- No historical data access
- Unbounded storage growth

**TPS Principle:** Kaizen — continuous improvement data

**Evidence Required:**
- [ ] Storage format
- [ ] Retention policy
- [ ] Query interface

---

## Section 3: Memory Management [10 Items]

### MM-01: Available Memory Accuracy

**Claim:** cohete reports available memory within ±50MB of actual.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_accuracy
```

**Null Hypothesis:** Memory reporting inaccurate.

**Rejection Criteria:**
- Deviation >50MB from `/proc/meminfo`
- Unified memory not accounted

**TPS Principle:** Accurate measurement

**Evidence Required:**
- [ ] meminfo comparison
- [ ] GPU memory accounting
- [ ] Buffer/cache handling

---

### MM-02: Memory Budget Enforcement

**Claim:** cohete prevents allocation exceeding memory budget.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_budget
```

**Null Hypothesis:** Budget exceeded.

**Rejection Criteria:**
- Allocation succeeds beyond budget
- OOM occurs despite budget

**TPS Principle:** Poka-Yoke — error prevention

**Evidence Required:**
- [ ] Allocation tracking
- [ ] Budget enforcement tests
- [ ] OOM avoidance verification

---

### MM-03: Model Memory Estimation

**Claim:** cohete accurately estimates model memory requirements.

**Falsification Test:**
```bash
cargo test --package cohete --test model_memory_estimation
```

**Null Hypothesis:** Estimation inaccurate.

**Rejection Criteria:**
- Underestimate causes OOM
- Overestimate >20%

**TPS Principle:** Resource planning

**Evidence Required:**
- [ ] Per-layer accounting
- [ ] Activation memory
- [ ] KV cache sizing

*Reference: [10] Sheng et al. "FlexGen: High-Throughput Generative Inference"*

---

### MM-04: Auto-Quantization Trigger

**Claim:** cohete auto-quantizes models exceeding memory budget.

**Falsification Test:**
```bash
cargo test --package cohete --test auto_quantization
```

**Null Hypothesis:** Oversized model deployed.

**Rejection Criteria:**
- fp16 model deployed on 8GB device
- Quantization quality degradation undocumented

**TPS Principle:** Automatic adaptation

**Evidence Required:**
- [ ] Size threshold logic
- [ ] Quantization level selection
- [ ] Quality metrics preservation

---

### MM-05: Swap Usage Monitoring

**Claim:** cohete tracks swap usage and warns on excessive swapping.

**Falsification Test:**
```bash
cargo test --package cohete --test swap_monitoring
```

**Null Hypothesis:** Swap usage unmonitored.

**Rejection Criteria:**
- Swap thrashing undetected
- Performance degradation unwarned

**TPS Principle:** Problem visibility

**Evidence Required:**
- [ ] Swap rate measurement
- [ ] Warning threshold
- [ ] Recommendation engine

---

### MM-06: Memory Leak Detection

**Claim:** cohete detects memory leaks in long-running inference.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_leak_detection
```

**Null Hypothesis:** Leaks undetected.

**Rejection Criteria:**
- Linear memory growth undetected
- False positive rate >10%

**TPS Principle:** Jidoka — defect detection

**Evidence Required:**
- [ ] Trend analysis
- [ ] Leak signature identification
- [ ] Alert mechanism

---

### MM-07: Unified Memory Zero-Copy

**Claim:** cohete uses zero-copy transfers on unified memory architecture.

**Falsification Test:**
```bash
cargo test --package cohete --test zero_copy
```

**Null Hypothesis:** Unnecessary copies occur.

**Rejection Criteria:**
- CPU→GPU copy on unified memory
- Buffer duplication

**TPS Principle:** Muda (Motion) elimination

**Evidence Required:**
- [ ] Memory allocation verification
- [ ] Copy count measurement
- [ ] Performance comparison

*Reference: [11] NVIDIA Jetson Memory Allocation Guide*

---

### MM-08: Memory Fragmentation Handling

**Claim:** cohete handles memory fragmentation gracefully.

**Falsification Test:**
```bash
cargo test --package cohete --test fragmentation
```

**Null Hypothesis:** Fragmentation causes failure.

**Rejection Criteria:**
- Allocation fails despite sufficient free memory
- No defragmentation strategy

**TPS Principle:** Resilient operation

**Evidence Required:**
- [ ] Fragmentation simulation
- [ ] Large allocation handling
- [ ] Recovery strategy

---

### MM-09: Memory Pressure Response

**Claim:** cohete reduces batch size under memory pressure.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_pressure
```

**Null Hypothesis:** Batch size unchanged under pressure.

**Rejection Criteria:**
- OOM instead of batch reduction
- No gradual degradation

**TPS Principle:** Graceful degradation

**Evidence Required:**
- [ ] Pressure detection
- [ ] Batch size adjustment
- [ ] Performance impact measurement

---

### MM-10: Memory Accounting Correctness

**Claim:** cohete tracks all memory allocations for audit.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_accounting
```

**Null Hypothesis:** Allocations untracked.

**Rejection Criteria:**
- Discrepancy between tracked and actual
- Audit log incomplete

**TPS Principle:** Auditability

**Evidence Required:**
- [ ] Allocation/deallocation logging
- [ ] Category breakdown
- [ ] Audit report generation

---

## Section 4: Power Management [10 Items]

### PM-01: Power Mode Setting

**Claim:** cohete correctly sets nvpmodel power modes.

**Falsification Test:**
```bash
cargo test --package cohete --test power_mode
```

**Null Hypothesis:** Power mode not applied.

**Rejection Criteria:**
- Mode change rejected
- Settings not persisted

**TPS Principle:** Configuration control

**Evidence Required:**
- [ ] nvpmodel invocation
- [ ] Verification after set
- [ ] Persistence across reboot

*Reference: [12] NVIDIA L4T Power Management*

---

### PM-02: Jetson Clocks Control

**Claim:** cohete enables/disables jetson_clocks correctly.

**Falsification Test:**
```bash
cargo test --package cohete --test jetson_clocks
```

**Null Hypothesis:** Clock control fails.

**Rejection Criteria:**
- Frequency not maximized
- Thermal safety not checked

**TPS Principle:** Performance optimization

**Evidence Required:**
- [ ] Clock frequency verification
- [ ] Fan speed correlation
- [ ] Thermal impact

---

### PM-03: Power Consumption Measurement

**Claim:** cohete reports power consumption within ±0.5W.

**Falsification Test:**
```bash
cargo test --package cohete --test power_measurement
```

**Null Hypothesis:** Power measurement inaccurate.

**Rejection Criteria:**
- Deviation >0.5W from INA sensor
- Stale readings

**TPS Principle:** Accurate measurement

**Evidence Required:**
- [ ] INA3221 sensor reading
- [ ] Calibration verification
- [ ] Multi-rail accounting

*Reference: [13] Texas Instruments INA3221 Datasheet*

---

### PM-04: Power Budget Enforcement

**Claim:** cohete throttles inference to stay within power budget.

**Falsification Test:**
```bash
cargo test --package cohete --test power_budget
```

**Null Hypothesis:** Power budget exceeded.

**Rejection Criteria:**
- Sustained power >budget
- No throttling mechanism

**TPS Principle:** Resource constraint compliance

**Evidence Required:**
- [ ] Budget configuration
- [ ] Throttling activation
- [ ] Power time series

---

### PM-05: Battery Operation Support

**Claim:** cohete supports battery-powered operation with appropriate policies.

**Falsification Test:**
```bash
cargo test --package cohete --test battery_operation
```

**Null Hypothesis:** Battery mode not supported.

**Rejection Criteria:**
- No power-saving mode
- Battery drain unoptimized

**TPS Principle:** Mobile deployment support

**Evidence Required:**
- [ ] Battery detection
- [ ] Power-saving activation
- [ ] Runtime estimation

---

### PM-06: Wake-on-LAN Support

**Claim:** cohete can wake Jetson devices from network.

**Falsification Test:**
```bash
cargo test --package cohete --test wake_on_lan
```

**Null Hypothesis:** WoL not supported.

**Rejection Criteria:**
- Device doesn't wake
- Magic packet not sent correctly

**TPS Principle:** Fleet management

**Evidence Required:**
- [ ] WoL packet generation
- [ ] Wake verification
- [ ] Network configuration

---

### PM-07: Idle Power Optimization

**Claim:** cohete reduces power during idle periods.

**Falsification Test:**
```bash
cargo test --package cohete --test idle_power
```

**Null Hypothesis:** Full power during idle.

**Rejection Criteria:**
- No power reduction when idle
- Slow wake from idle

**TPS Principle:** Muda (Inventory) — energy waste

**Evidence Required:**
- [ ] Idle detection
- [ ] Power state transition
- [ ] Wake latency

---

### PM-08: Power Profile Customization

**Claim:** cohete supports custom power profiles via YAML.

**Falsification Test:**
```bash
cargo test --package cohete --test power_profile_yaml
```

**Null Hypothesis:** Custom profile ignored.

**Rejection Criteria:**
- YAML settings not applied
- Invalid profile accepted

**TPS Principle:** Configurability

**Evidence Required:**
- [ ] YAML schema
- [ ] Profile application
- [ ] Validation errors

---

### PM-09: Reboot on Power Change

**Claim:** cohete safely reboots when power mode change requires it.

**Falsification Test:**
```bash
cargo test --package cohete --test power_reboot
```

**Null Hypothesis:** Unsafe mode change.

**Rejection Criteria:**
- Mode change applied without reboot when required
- Data loss on reboot

**TPS Principle:** Safe state transitions

**Evidence Required:**
- [ ] Reboot requirement detection
- [ ] State preservation
- [ ] Graceful shutdown

---

### PM-10: Power Monitoring Frequency

**Claim:** cohete samples power at configurable frequency (default 100ms).

**Falsification Test:**
```bash
cargo test --package cohete --test power_sampling
```

**Null Hypothesis:** Sampling too slow.

**Rejection Criteria:**
- Sampling interval >configured
- Spikes missed

**TPS Principle:** Measurement granularity

**Evidence Required:**
- [ ] Timing accuracy
- [ ] Spike detection
- [ ] Resource overhead

---

## Section 5: Storage Management [10 Items]

### SM-01: NVMe Detection

**Claim:** cohete detects NVMe SSD in M.2 slot.

**Falsification Test:**
```bash
cargo test --package cohete --test nvme_detection
```

**Null Hypothesis:** NVMe not detected.

**Rejection Criteria:**
- Present SSD not detected
- Wrong capacity reported

**TPS Principle:** Hardware visibility

**Evidence Required:**
- [ ] Device enumeration
- [ ] Capacity verification
- [ ] Health status

---

### SM-02: NVMe Provisioning

**Claim:** cohete provisions NVMe with ext4 and swap.

**Falsification Test:**
```bash
cargo test --package cohete --test nvme_provision
```

**Null Hypothesis:** Provisioning fails.

**Rejection Criteria:**
- Formatting fails
- Swap not created

**TPS Principle:** Automated setup

**Evidence Required:**
- [ ] Partition creation
- [ ] Filesystem verification
- [ ] Swap activation

---

### SM-03: Mount Persistence

**Claim:** cohete configures mounts to persist across reboots.

**Falsification Test:**
```bash
cargo test --package cohete --test mount_persistence
```

**Null Hypothesis:** Mounts lost after reboot.

**Rejection Criteria:**
- fstab entry missing
- Mount fails after reboot

**TPS Principle:** Configuration persistence

**Evidence Required:**
- [ ] fstab modification
- [ ] UUID usage
- [ ] Reboot verification

---

### SM-04: Storage Health Monitoring

**Claim:** cohete monitors NVMe health via SMART.

**Falsification Test:**
```bash
cargo test --package cohete --test storage_health
```

**Null Hypothesis:** Health not monitored.

**Rejection Criteria:**
- SMART attributes not read
- Wear level not tracked

**TPS Principle:** Predictive maintenance

**Evidence Required:**
- [ ] SMART attribute parsing
- [ ] Threshold alerting
- [ ] Trend analysis

*Reference: [14] NVMe Specification - SMART*

---

### SM-05: Swap Configuration

**Claim:** cohete creates and optimizes swap for ML workloads.

**Falsification Test:**
```bash
cargo test --package cohete --test swap_config
```

**Null Hypothesis:** Swap suboptimal.

**Rejection Criteria:**
- Swappiness not tuned
- Swap size inappropriate

**TPS Principle:** ML workload optimization

**Evidence Required:**
- [ ] Swappiness setting (vm.swappiness=10)
- [ ] Swap size calculation
- [ ] NVMe-backed swap performance

---

### SM-06: Model Storage Layout

**Claim:** cohete organizes model storage in standard directory structure.

**Falsification Test:**
```bash
cargo test --package cohete --test model_storage
```

**Null Hypothesis:** Disorganized storage.

**Rejection Criteria:**
- No standard paths
- Permission issues

**TPS Principle:** Organization

**Evidence Required:**
- [ ] Directory structure
- [ ] Permission settings
- [ ] Quota management

---

### SM-07: Storage Space Monitoring

**Claim:** cohete warns when storage space is low.

**Falsification Test:**
```bash
cargo test --package cohete --test storage_space
```

**Null Hypothesis:** Low space undetected.

**Rejection Criteria:**
- <10% free without warning
- Disk full during model download

**TPS Principle:** Andon — visual warning

**Evidence Required:**
- [ ] Threshold configuration
- [ ] Warning mechanism
- [ ] Automatic cleanup option

---

### SM-08: Model Cache Management

**Claim:** cohete manages model cache with LRU eviction.

**Falsification Test:**
```bash
cargo test --package cohete --test model_cache
```

**Null Hypothesis:** Cache unbounded.

**Rejection Criteria:**
- No eviction when full
- LRU order incorrect

**TPS Principle:** Muda (Inventory) prevention

**Evidence Required:**
- [ ] Cache size limit
- [ ] Eviction policy
- [ ] Access time tracking

---

### SM-09: Docker Storage Migration

**Claim:** cohete can migrate Docker storage to NVMe.

**Falsification Test:**
```bash
cargo test --package cohete --test docker_migration
```

**Null Hypothesis:** Migration fails.

**Rejection Criteria:**
- Data loss during migration
- Docker fails to start

**TPS Principle:** Resource optimization

**Evidence Required:**
- [ ] Migration procedure
- [ ] Data integrity verification
- [ ] Rollback capability

---

### SM-10: Storage Performance Benchmark

**Claim:** cohete benchmarks storage performance.

**Falsification Test:**
```bash
cargo test --package cohete --test storage_benchmark
```

**Null Hypothesis:** Performance unknown.

**Rejection Criteria:**
- No benchmark capability
- Results not stored

**TPS Principle:** Performance visibility

**Evidence Required:**
- [ ] Read/write throughput
- [ ] Latency measurement
- [ ] Baseline comparison

---

## Section 6: Fleet Orchestration [10 Items]

### FO-01: Multi-Device Discovery

**Claim:** cohete discovers all Jetson devices on network.

**Falsification Test:**
```bash
cargo test --package cohete --test fleet_discovery
```

**Null Hypothesis:** Devices missed.

**Rejection Criteria:**
- Reachable device not found
- Discovery takes >30 seconds

**TPS Principle:** Complete visibility

**Evidence Required:**
- [ ] Discovery method coverage
- [ ] Timing measurement
- [ ] Concurrent discovery

---

### FO-02: Load Balancing

**Claim:** cohete distributes inference load across fleet.

**Falsification Test:**
```bash
cargo test --package cohete --test load_balancing
```

**Null Hypothesis:** Load imbalanced.

**Rejection Criteria:**
- One device overloaded while others idle
- No load metric consideration

**TPS Principle:** Heijunka — load leveling

**Evidence Required:**
- [ ] Load distribution algorithm
- [ ] Thermal-aware balancing
- [ ] Utilization metrics

---

### FO-03: Failover Handling

**Claim:** cohete fails over inference to healthy devices.

**Falsification Test:**
```bash
cargo test --package cohete --test failover
```

**Null Hypothesis:** Failover fails.

**Rejection Criteria:**
- Request lost on device failure
- Failover latency >5 seconds

**TPS Principle:** Resilience

**Evidence Required:**
- [ ] Failure detection
- [ ] Request rerouting
- [ ] State recovery

---

### FO-04: Coordinated Model Deployment

**Claim:** cohete deploys models to fleet atomically.

**Falsification Test:**
```bash
cargo test --package cohete --test coordinated_deploy
```

**Null Hypothesis:** Inconsistent deployment.

**Rejection Criteria:**
- Different model versions serving simultaneously
- Partial deployment left in bad state

**TPS Principle:** Atomic deployment

**Evidence Required:**
- [ ] Two-phase commit
- [ ] Rollback capability
- [ ] Version consistency

---

### FO-05: Fleet Health Dashboard

**Claim:** cohete provides fleet-wide health view.

**Falsification Test:**
```bash
cargo test --package cohete --test fleet_dashboard
```

**Null Hypothesis:** No aggregated view.

**Rejection Criteria:**
- Per-device view only
- Stale data in dashboard

**TPS Principle:** Visibility

**Evidence Required:**
- [ ] Aggregate metrics
- [ ] Refresh rate
- [ ] Alert aggregation

---

### FO-06: Scheduled Maintenance

**Claim:** cohete supports rolling maintenance windows.

**Falsification Test:**
```bash
cargo test --package cohete --test maintenance_window
```

**Null Hypothesis:** No maintenance support.

**Rejection Criteria:**
- All devices offline simultaneously
- No drain capability

**TPS Principle:** Availability during maintenance

**Evidence Required:**
- [ ] Drain mechanism
- [ ] Rolling schedule
- [ ] Capacity planning

---

### FO-07: Fleet Configuration Sync

**Claim:** cohete synchronizes configuration across fleet.

**Falsification Test:**
```bash
cargo test --package cohete --test config_sync
```

**Null Hypothesis:** Configuration drift.

**Rejection Criteria:**
- Devices with different settings
- No drift detection

**TPS Principle:** Consistency

**Evidence Required:**
- [ ] Config distribution
- [ ] Drift detection
- [ ] Reconciliation

---

### FO-08: Capacity Planning

**Claim:** cohete recommends fleet sizing for workload.

**Falsification Test:**
```bash
cargo test --package cohete --test capacity_planning
```

**Null Hypothesis:** No sizing guidance.

**Rejection Criteria:**
- No throughput estimation
- Overprovisioning recommendation

**TPS Principle:** Muda (Inventory) prevention

**Evidence Required:**
- [ ] Workload characterization
- [ ] Throughput modeling
- [ ] Cost optimization

---

### FO-09: Geographic Distribution

**Claim:** cohete supports geo-distributed fleets.

**Falsification Test:**
```bash
cargo test --package cohete --test geo_distribution
```

**Null Hypothesis:** Single location only.

**Rejection Criteria:**
- No latency-based routing
- No region awareness

**TPS Principle:** Edge deployment

**Evidence Required:**
- [ ] Region configuration
- [ ] Latency measurement
- [ ] Data residency compliance

---

### FO-10: Fleet Metrics Export

**Claim:** cohete exports fleet metrics in standard format.

**Falsification Test:**
```bash
cargo test --package cohete --test metrics_export
```

**Null Hypothesis:** No metrics export.

**Rejection Criteria:**
- Proprietary format only
- No Prometheus/OpenTelemetry support

**TPS Principle:** Observability

**Evidence Required:**
- [ ] Prometheus exposition
- [ ] OpenTelemetry integration
- [ ] Grafana dashboard

---

## Section 7: Model Quantization [10 Items]

### MQ-01: Quantization Level Support

**Claim:** cohete supports Q4_0, Q4_1, Q5_0, Q5_1, Q8_0 quantization.

**Falsification Test:**
```bash
cargo test --package cohete --test quant_levels
```

**Null Hypothesis:** Quantization levels not supported.

**Rejection Criteria:**
- Any listed level fails
- Output format incompatible

**TPS Principle:** Completeness

**Evidence Required:**
- [ ] Per-level testing
- [ ] Output validation
- [ ] llama.cpp compatibility

*Reference: [15] Dettmers et al. "QLoRA: Efficient Finetuning of Quantized LLMs"*

---

### MQ-02: Perplexity Preservation

**Claim:** cohete quantization maintains perplexity within bounds.

**Falsification Test:**
```bash
cargo test --package cohete --test perplexity_preservation
```

**Null Hypothesis:** Perplexity exceeds bounds.

**Rejection Criteria:**
- Q4_0: >5% perplexity increase
- Q8_0: >1% perplexity increase

**TPS Principle:** Quality preservation

**Evidence Required:**
- [ ] Perplexity measurement
- [ ] Baseline comparison
- [ ] Model-specific bounds

---

### MQ-03: Memory Reduction Accuracy

**Claim:** cohete achieves expected memory reduction from quantization.

**Falsification Test:**
```bash
cargo test --package cohete --test memory_reduction
```

**Null Hypothesis:** Memory reduction less than expected.

**Rejection Criteria:**
- Q4 not ~4x reduction
- Unexpected overhead

**TPS Principle:** Predictable behavior

**Evidence Required:**
- [ ] Memory measurement
- [ ] Reduction ratio
- [ ] Overhead accounting

---

### MQ-04: Auto-Quantization Selection

**Claim:** cohete selects optimal quantization level for memory budget.

**Falsification Test:**
```bash
cargo test --package cohete --test auto_quant_selection
```

**Null Hypothesis:** Suboptimal selection.

**Rejection Criteria:**
- Fits in Q5 but selects Q4
- OOM with selected level

**TPS Principle:** Intelligent automation

**Evidence Required:**
- [ ] Selection algorithm
- [ ] Budget adherence
- [ ] Quality prioritization

---

### MQ-05: Quantization Speed

**Claim:** cohete quantizes models at >100MB/s.

**Falsification Test:**
```bash
cargo test --package cohete --test quant_speed
```

**Null Hypothesis:** Quantization too slow.

**Rejection Criteria:**
- <100MB/s throughput
- Blocking UI during quantization

**TPS Principle:** Muda (Waiting) reduction

**Evidence Required:**
- [ ] Throughput measurement
- [ ] Progress reporting
- [ ] Background operation

---

### MQ-06: Quantization Validation

**Claim:** cohete validates quantized model integrity.

**Falsification Test:**
```bash
cargo test --package cohete --test quant_validation
```

**Null Hypothesis:** Corruption undetected.

**Rejection Criteria:**
- Corrupted output accepted
- No checksum verification

**TPS Principle:** Quality assurance

**Evidence Required:**
- [ ] Checksum generation
- [ ] Output validation
- [ ] Error detection

---

### MQ-07: Mixed Precision Support

**Claim:** cohete supports mixed precision (sensitive layers in higher precision).

**Falsification Test:**
```bash
cargo test --package cohete --test mixed_precision
```

**Null Hypothesis:** Uniform quantization only.

**Rejection Criteria:**
- No per-layer control
- Quality regression on sensitive layers

**TPS Principle:** Optimization flexibility

**Evidence Required:**
- [ ] Layer selection
- [ ] Sensitivity analysis
- [ ] Quality comparison

*Reference: [16] Xiao et al. "SmoothQuant: Accurate and Efficient Post-Training Quantization"*

---

### MQ-08: Quantization Report

**Claim:** cohete generates quantization quality report.

**Falsification Test:**
```bash
cargo test --package cohete --test quant_report
```

**Null Hypothesis:** No quality report.

**Rejection Criteria:**
- No metrics in output
- Missing perplexity delta

**TPS Principle:** Transparency

**Evidence Required:**
- [ ] Report content
- [ ] Metrics included
- [ ] Format specification

---

### MQ-09: Incremental Quantization

**Claim:** cohete supports incremental quantization (only changed layers).

**Falsification Test:**
```bash
cargo test --package cohete --test incremental_quant
```

**Null Hypothesis:** Full requantization required.

**Rejection Criteria:**
- No caching of quantized layers
- Full recomputation on minor change

**TPS Principle:** Muda (Overprocessing) prevention

**Evidence Required:**
- [ ] Layer caching
- [ ] Change detection
- [ ] Speed improvement

---

### MQ-10: Quantization Reproducibility

**Claim:** cohete produces identical quantization output for same input.

**Falsification Test:**
```bash
cargo test --package cohete --test quant_reproducibility
```

**Null Hypothesis:** Non-deterministic output.

**Rejection Criteria:**
- Different output on same input
- No seed control

**TPS Principle:** Reproducibility

**Evidence Required:**
- [ ] Hash comparison
- [ ] Seed documentation
- [ ] Cross-platform consistency

---

## Section 8: Provisioning & Setup [10 Items]

### PS-01: Interactive Setup Wizard

**Claim:** cohete provides interactive setup completing full configuration.

**Falsification Test:**
```bash
cargo test --package cohete --test setup_wizard
```

**Null Hypothesis:** Setup incomplete.

**Rejection Criteria:**
- Missing configuration step
- User left with non-functional state

**TPS Principle:** Complete experience

**Evidence Required:**
- [ ] Step coverage
- [ ] Error recovery
- [ ] State validation

---

### PS-02: YAML Configuration Generation

**Claim:** cohete generates valid YAML from interactive setup.

**Falsification Test:**
```bash
cargo test --package cohete --test yaml_generation
```

**Null Hypothesis:** Invalid YAML generated.

**Rejection Criteria:**
- YAML parse error
- Missing required fields

**TPS Principle:** Declarative configuration

**Evidence Required:**
- [ ] Schema validation
- [ ] Round-trip testing
- [ ] Default completeness

---

### PS-03: SSH Key Setup

**Claim:** cohete configures SSH keys for passwordless access.

**Falsification Test:**
```bash
cargo test --package cohete --test ssh_key_setup
```

**Null Hypothesis:** SSH requires password.

**Rejection Criteria:**
- Key not copied
- Permissions incorrect

**TPS Principle:** Security automation

**Evidence Required:**
- [ ] Key generation
- [ ] Key copy verification
- [ ] Permission check

---

### PS-04: SSH Config Integration

**Claim:** cohete adds entries to ~/.ssh/config.

**Falsification Test:**
```bash
cargo test --package cohete --test ssh_config_integration
```

**Null Hypothesis:** Config not updated.

**Rejection Criteria:**
- Entry missing
- Existing config corrupted

**TPS Principle:** System integration

**Evidence Required:**
- [ ] Config modification
- [ ] Backup creation
- [ ] Conflict resolution

---

### PS-05: NVMe Auto-Setup

**Claim:** cohete automatically sets up NVMe on first detection.

**Falsification Test:**
```bash
cargo test --package cohete --test nvme_auto_setup
```

**Null Hypothesis:** Manual setup required.

**Rejection Criteria:**
- NVMe present but unmounted
- No user prompt for destructive operation

**TPS Principle:** Automation with confirmation

**Evidence Required:**
- [ ] Detection logic
- [ ] User confirmation
- [ ] Setup verification

---

### PS-06: Package Installation

**Claim:** cohete installs required packages on Jetson.

**Falsification Test:**
```bash
cargo test --package cohete --test package_install
```

**Null Hypothesis:** Missing packages.

**Rejection Criteria:**
- Required package not installed
- Broken dependencies

**TPS Principle:** Complete environment

**Evidence Required:**
- [ ] Package list
- [ ] Dependency resolution
- [ ] Version pinning

---

### PS-07: Firmware Version Check

**Claim:** cohete verifies JetPack/L4T firmware version.

**Falsification Test:**
```bash
cargo test --package cohete --test firmware_check
```

**Null Hypothesis:** Version not checked.

**Rejection Criteria:**
- Incompatible firmware not detected
- No upgrade guidance

**TPS Principle:** Compatibility assurance

**Evidence Required:**
- [ ] Version parsing
- [ ] Compatibility matrix
- [ ] Upgrade instructions

---

### PS-08: Network Configuration

**Claim:** cohete configures network for both USB and Ethernet access.

**Falsification Test:**
```bash
cargo test --package cohete --test network_config
```

**Null Hypothesis:** Network misconfigured.

**Rejection Criteria:**
- USB gadget network not working
- Static IP not set

**TPS Principle:** Connectivity

**Evidence Required:**
- [ ] Interface configuration
- [ ] IP assignment
- [ ] Routing verification

---

### PS-09: CUDA Environment Setup

**Claim:** cohete configures CUDA environment variables.

**Falsification Test:**
```bash
cargo test --package cohete --test cuda_env
```

**Null Hypothesis:** CUDA not configured.

**Rejection Criteria:**
- CUDA_HOME not set
- PATH not updated

**TPS Principle:** Development environment

**Evidence Required:**
- [ ] Environment variables
- [ ] Shell profile update
- [ ] Verification command

---

### PS-10: Provisioning Idempotency

**Claim:** cohete provisioning is idempotent (safe to run multiple times).

**Falsification Test:**
```bash
cargo test --package cohete --test provision_idempotent
```

**Null Hypothesis:** Re-run causes problems.

**Rejection Criteria:**
- Duplicate entries
- Data loss on re-run

**TPS Principle:** Safe operations

**Evidence Required:**
- [ ] Multiple run testing
- [ ] State checking
- [ ] Incremental updates

---

## Section 9: Safety & Security [5 Items]

### SS-01: Credential Security

**Claim:** cohete never logs or exposes credentials.

**Falsification Test:**
```bash
cargo test --package cohete --test credential_security
```

**Null Hypothesis:** Credentials exposed.

**Rejection Criteria:**
- Password in logs
- Key in error message

**TPS Principle:** Security by design

**Evidence Required:**
- [ ] Log audit
- [ ] Error message review
- [ ] Memory scrubbing

---

### SS-02: Secure Default Configuration

**Claim:** cohete uses secure defaults (SSH keys, not passwords).

**Falsification Test:**
```bash
cargo test --package cohete --test secure_defaults
```

**Null Hypothesis:** Insecure defaults.

**Rejection Criteria:**
- Password authentication enabled
- Weak cipher allowed

**TPS Principle:** Security by default

**Evidence Required:**
- [ ] Default configuration audit
- [ ] SSH hardening
- [ ] Cipher list

---

### SS-03: Root Access Control

**Claim:** cohete uses sudo only when necessary.

**Falsification Test:**
```bash
cargo test --package cohete --test root_access
```

**Null Hypothesis:** Excessive root usage.

**Rejection Criteria:**
- Root used for read-only operations
- No privilege drop

**TPS Principle:** Least privilege

**Evidence Required:**
- [ ] Privilege audit
- [ ] sudo usage documentation
- [ ] Alternative methods

---

### SS-04: Network Attack Surface

**Claim:** cohete minimizes open ports and network exposure.

**Falsification Test:**
```bash
cargo test --package cohete --test network_surface
```

**Null Hypothesis:** Unnecessary exposure.

**Rejection Criteria:**
- Ports open without need
- No firewall recommendation

**TPS Principle:** Attack surface reduction

**Evidence Required:**
- [ ] Port usage documentation
- [ ] Firewall rules
- [ ] TLS enforcement

---

### SS-05: Update Security

**Claim:** cohete verifies package/model signatures.

**Falsification Test:**
```bash
cargo test --package cohete --test update_security
```

**Null Hypothesis:** Updates unverified.

**Rejection Criteria:**
- Unsigned package accepted
- No signature verification

**TPS Principle:** Supply chain security

**Evidence Required:**
- [ ] Signature verification
- [ ] Key management
- [ ] Revocation handling

---

## Section 10: Architectural Invariants [5 Items] — CRITICAL

*Hard requirements. Any failure = project FAIL. No exceptions.*

### AI-01: Declarative YAML Configuration

**Claim:** cohete offers full functionality via declarative YAML without code.

**Falsification Test:**
```bash
cargo test --package cohete --test declarative_coverage
```

**Null Hypothesis:** Code required for basic functionality.

**Rejection Criteria (CRITICAL):**
- Any core feature unavailable via YAML config
- User must write Rust to use basic functionality

**TPS Principle:** Poka-Yoke — enable non-developers

**Evidence Required:**
- [ ] YAML schema documentation
- [ ] Feature coverage matrix (YAML vs API)
- [ ] No-code quickstart example

**Severity:** CRITICAL — Project FAIL if not met

---

### AI-02: Zero Scripting in Production

**Claim:** No Python/JavaScript/Lua in cohete production runtime.

**Falsification Test:**
```bash
cargo test --package cohete --test zero_scripting
cargo tree --edges no-dev | grep -E "pyo3|napi|mlua"
```

**Null Hypothesis:** Scripting language dependencies exist.

**Rejection Criteria (CRITICAL):**
- Any `.py`, `.js`, `.lua` in src/ or runtime
- pyo3, napi-rs, mlua in non-dev dependencies

**TPS Principle:** Jidoka — type safety, determinism

**Evidence Required:**
- [ ] Dependency audit
- [ ] File extension audit
- [ ] Build artifact inspection

**Severity:** CRITICAL — Project FAIL if not met

---

### AI-03: Pure Rust Implementation

**Claim:** cohete is implemented entirely in safe Rust (except FFI).

**Falsification Test:**
```bash
cargo test --package cohete --test unsafe_audit
```

**Null Hypothesis:** Unsafe code outside FFI.

**Rejection Criteria (CRITICAL):**
- Unsafe block outside src/ffi/
- No safety documentation for FFI

**TPS Principle:** Memory safety

**Evidence Required:**
- [ ] Unsafe block inventory
- [ ] FFI safety documentation
- [ ] Miri verification

**Severity:** CRITICAL — Project FAIL if not met

---

### AI-04: trueno Integration

**Claim:** cohete uses trueno for all SIMD/compute operations.

**Falsification Test:**
```bash
cargo test --package cohete --test trueno_integration
```

**Null Hypothesis:** Custom SIMD implementation.

**Rejection Criteria (CRITICAL):**
- Inline SIMD intrinsics
- Custom ARM NEON code

**TPS Principle:** Code reuse, tested foundation

**Evidence Required:**
- [ ] trueno dependency
- [ ] No inline SIMD
- [ ] Backend selection integration

**Severity:** CRITICAL — Project FAIL if not met

---

### AI-05: Declarative Schema Validation

**Claim:** YAML configs validated against typed schema.

**Falsification Test:**
```bash
cargo test --package cohete --test yaml_schema_validation
```

**Null Hypothesis:** YAML accepted without validation.

**Rejection Criteria (CRITICAL):**
- Invalid YAML silently accepted
- No JSON Schema or serde validation
- Runtime panics on bad config

**TPS Principle:** Poka-Yoke — prevent config errors

**Evidence Required:**
- [ ] JSON Schema or Rust struct with serde
- [ ] Validation error messages
- [ ] Invalid config test cases

**Severity:** CRITICAL — Project FAIL if not met

---

## Part VIII: Evaluation Protocol

### Scoring Methodology

Each item scored as:
- **Pass (1):** Rejection criteria avoided, evidence provided
- **Partial (0.5):** Some evidence, minor issues
- **Fail (0):** Rejection criteria met, claim falsified

**Total Score:** Sum / 100 × 100%

### TPS-Aligned Assessment

| Score | Assessment | TPS Response |
|-------|------------|--------------|
| 95-100% | Toyota Standard | Release |
| 85-94% | Kaizen Required | Beta with documented issues |
| 70-84% | Andon Warning | Significant revision |
| <70% | Stop the Line | Major rework |

### Current Status

| Section | Items | Verified | Score |
|---------|-------|----------|-------|
| Device Discovery & Connection | 15 | 0 | 0% |
| Thermal Monitoring | 15 | 0 | 0% |
| Memory Management | 10 | 0 | 0% |
| Power Management | 10 | 0 | 0% |
| Storage Management | 10 | 0 | 0% |
| Fleet Orchestration | 10 | 0 | 0% |
| Model Quantization | 10 | 0 | 0% |
| Provisioning & Setup | 10 | 0 | 0% |
| Safety & Security | 5 | 0 | 0% |
| **Architectural Invariants** | **5** | **0** | **0%** |
| **Total** | **100** | **0** | **0%** |

---

## Part IX: Peer-Reviewed References

### NVIDIA Jetson Hardware
- [1] NVIDIA. "Jetson Orin Nano Developer Kit." https://developer.nvidia.com/embedded/jetson-orin-nano-developer-kit
- [2] NVIDIA. "Jetson Modules Datasheet." Technical Specifications, 2024.
- [8] NVIDIA. "L4T Developer Guide - tegrastats Utility." L4T Documentation.
- [11] NVIDIA. "Jetson Memory Allocation Guide." CUDA Developer Documentation.
- [12] NVIDIA. "L4T Power Management." nvpmodel Documentation.

### ARM Architecture
- [3] ARM. "Cortex-A78AE Technical Reference Manual." ARM DDI 0487G.a.

### Network Protocols
- [4] USB Implementers Forum. "USB CDC ACM Class Specification." Rev 1.2.
- [5] Cheshire, S., & Krochmal, M. (2013). "Multicast DNS." RFC 6762.
- [6] Ylonen, T., & Lonvick, C. (2006). "SSH Authentication Protocol." RFC 4252.

### Security Standards
- [7] NIST. "Guidelines for the Selection, Configuration, and Use of TLS." SP 800-52 Rev 2.

### Thermal Management
- [9] Wang, Z., et al. (2020). "Thermal Prediction for Embedded ML Systems." *ACM TECS*.

### ML Optimization
- [10] Sheng, Y., et al. (2023). "FlexGen: High-Throughput Generative Inference of Large Language Models with a Single GPU." *ICML*.
- [15] Dettmers, T., et al. (2023). "QLoRA: Efficient Finetuning of Quantized LLMs." *NeurIPS*.
- [16] Xiao, G., et al. (2023). "SmoothQuant: Accurate and Efficient Post-Training Quantization for Large Language Models." *ICML*.

### Storage
- [13] Texas Instruments. "INA3221 Triple-Channel, High-Side Measurement, Shunt and Bus Voltage Monitor." Datasheet.
- [14] NVM Express. "NVM Express Specification." Revision 2.0.

### Toyota Production System
- Liker, J. (2004). *The Toyota Way*. McGraw-Hill.
- Ohno, T. (1988). *Toyota Production System: Beyond Large-Scale Production*. Productivity Press.

### Rust Safety
- Jung, R., et al. (2017). "RustBelt: Securing the Foundations of the Rust Programming Language." *POPL 2018*.

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0-draft | 2026-01-12 | Team | Initial specification |

---

**Status:** SPECIFICATION — Implementation Pending

**Document Philosophy:**
> "The goal is not to just build a product, but to build a capacity to produce." — Toyota Way
