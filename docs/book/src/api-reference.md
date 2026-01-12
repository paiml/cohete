# API Reference

Full API documentation is available on [docs.rs/cohete](https://docs.rs/cohete).

## Core Types

### JetsonModel

Supported Jetson hardware models:

```rust
pub enum JetsonModel {
    OrinNano4GB,
    OrinNano8GB,
    OrinNX8GB,
    OrinNX16GB,
    AgxOrin32GB,
    AgxOrin64GB,
    Unknown,
}
```

**Methods:**
- `memory_mb() -> u64` - Total memory in MB
- `cuda_cores() -> u32` - Number of CUDA cores
- `tops() -> u32` - AI performance in TOPS

### Error

Cohete error type:

```rust
pub enum Error {
    DeviceNotFound(String),
    ConnectionFailed { host: String, reason: String },
    Ssh(String),
    Usb(String),
    ThermalExceeded { current_c: f32, threshold_c: f32 },
    InsufficientMemory { requested_mb: u64, available_mb: u64 },
    MemoryBudgetExceeded { used_mb: u64, budget_mb: u64 },
    InvalidYaml(String),
    // ... more variants
}
```

### Result

Type alias for cohete operations:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Modules

### cohete::device

- `JetsonDevice` - Device handle
- `DeviceInfo` - Device metadata
- `ConnectionMethod` - USB, Ethernet, mDNS
- `ComputeHint` - trueno backend hints

### cohete::thermal

- `TegraMonitor` - tegrastats interface
- `TegraStats` - Thermal/memory statistics
- `ThermalPolicy` - Temperature thresholds
- `ThermalCircuitBreaker` - Jidoka pattern
- `ThermalZone` - GPU, CPU, SOC, Board

### cohete::memory

- `MemoryBudget` - Budget enforcer
- `MemoryGuard` - RAII allocation guard
- `ModelMemoryEstimate` - Model size estimation

### cohete::power

- `PowerMode` - nvpmodel modes
- `JetsonClocks` - Clock controller
- `PowerProfile` - Preset configurations

### cohete::storage

- `NvmeDevice` - NVMe handle
- `SwapConfig` - Swap configuration
- `StorageLayout` - Directory layout

### cohete::fleet

- `Fleet` - Device collection
- `FleetMember` - Device + policy
- `FleetHealth` - Health status
- `DeploymentConfig` - Deployment settings
- `JetsonExecutor` - repartir integration (batuta feature)

### cohete::quantize

- `QuantLevel` - Quantization levels
- `JetsonQuantizer` - Quantization controller
- `QuantResult` - Quantization results

### cohete::provision

- `ProvisionConfig` - Setup configuration
- `SetupWizard` - Interactive setup
- `SshConfigEntry` - SSH config generation
- `ProvisionResult` - Setup results

### cohete::config

- `CoheteConfig` - Root configuration
- `DiscoveryConfig` - Discovery settings
- `FleetConfig` - Fleet settings
- `ModelConfig` - Model settings
- `ThermalPoliciesConfig` - Thermal settings
- `InferenceConfig` - Server settings

## Feature Flags

| Flag | Description | Dependencies |
|------|-------------|--------------|
| `default` | Core functionality | - |
| `arm-neon` | ARM NEON SIMD | - |
| `tegra-stats` | tegrastats monitoring | - |
| `nvme` | NVMe storage | - |
| `batuta` | Stack integration | repartir, pacha, renacer |
| `cli` | CLI tools | clap, indicatif |
| `full` | All features | - |

## Generating Local Docs

```bash
cargo doc --open
```

With all features:

```bash
cargo doc --all-features --open
```
