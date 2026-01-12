# Getting Started

This guide will help you get started with cohete for NVIDIA Jetson edge AI deployment.

## Prerequisites

- Rust 1.75 or later
- NVIDIA Jetson device (Orin Nano, Orin NX, or AGX Orin)
- USB-C cable or Ethernet connection to Jetson

## Installation

Add cohete to your `Cargo.toml`:

```toml
[dependencies]
cohete = "0.1"
tokio = { version = "1.42", features = ["full"] }
```

For full functionality including CLI tools:

```toml
[dependencies]
cohete = { version = "0.1", features = ["full"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `default` | Core functionality |
| `arm-neon` | ARM NEON SIMD (auto-detected by trueno) |
| `tegra-stats` | tegrastats monitoring |
| `nvme` | NVMe storage management |
| `batuta` | Sovereign AI Stack integration |
| `cli` | Command-line tools |
| `full` | All features enabled |

## Basic Usage

### 1. Discover Devices

```rust
use cohete::device::JetsonDevice;

#[tokio::main]
async fn main() -> cohete::Result<()> {
    let devices = JetsonDevice::discover_all().await?;

    for device in devices {
        println!("Found: {} ({})", device.id(), device.model());
    }

    Ok(())
}
```

### 2. Monitor Thermals

```rust
use cohete::thermal::{TegraMonitor, ThermalPolicy};

let mut monitor = TegraMonitor::new()
    .with_policy(ThermalPolicy::conservative());

let stats = monitor.sample()?;
println!("GPU: {}°C", stats.gpu_temp);
```

### 3. Manage Memory

```rust
use cohete::memory::MemoryBudget;

let budget = MemoryBudget::orin_nano_8gb();
println!("Available: {} MB", budget.available_mb());

// Allocation is tracked with RAII guards
let guard = budget.allocate(2000, "model_weights")?;
// Memory automatically released when guard drops
```

## Running Examples

Cohete includes several runnable examples:

```bash
# Device discovery
cargo run --example device_discovery

# Thermal monitoring
cargo run --example thermal_monitoring

# Memory management
cargo run --example memory_budget

# Fleet management
cargo run --example fleet_management

# Model quantization
cargo run --example quantization

# YAML configuration
cargo run --example yaml_config
```

## Next Steps

- [Hardware Requirements](getting-started/hardware.md)
- [Device Discovery](device-discovery.md)
- [Thermal Management](thermal-management.md)
