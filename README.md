# Cohete: NVIDIA Jetson Edge Hardware Integration

[![Crates.io](https://img.shields.io/crates/v/cohete.svg)](https://crates.io/crates/cohete)
[![Documentation](https://docs.rs/cohete/badge.svg)](https://docs.rs/cohete)
[![Book](https://img.shields.io/badge/book-paiml.github.io%2Fcohete-blue)](https://paiml.github.io/cohete)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<p align="center">
  <img src="docs/book/src/assets/hero.svg" alt="Cohete - NVIDIA Jetson Edge Hardware Integration" width="600">
</p>

**Cohete** (Spanish: "rocket") provides safe, pure Rust interfaces to NVIDIA Jetson edge hardware for the [Sovereign AI Stack](https://github.com/paiml/batuta).

## Features

- **Device Discovery**: USB CDC, mDNS, and SSH-based Jetson detection
- **Thermal Monitoring**: tegrastats integration with Jidoka circuit breakers
- **Memory Management**: Budget-aware allocation for 8GB constrained devices
- **Power Management**: nvpmodel and jetson_clocks control
- **Fleet Orchestration**: Multi-device management with repartir integration
- **Model Quantization**: Memory-aware Q4/Q5/Q8 quantization for edge deployment
- **Declarative Config**: Full YAML configuration (Architectural Invariant)

## Supported Hardware

| Device | Memory | CUDA Cores | AI TOPS |
|--------|--------|------------|---------|
| Jetson Orin Nano 4GB | 4GB | 512 | 20 |
| Jetson Orin Nano 8GB | 8GB | 1024 | 40 |
| Jetson Orin NX 8GB | 8GB | 1024 | 70 |
| Jetson Orin NX 16GB | 16GB | 1024 | 100 |
| Jetson AGX Orin 32GB | 32GB | 2048 | 200 |
| Jetson AGX Orin 64GB | 64GB | 2048 | 275 |

## Quick Start

```rust
use cohete::{device::JetsonDevice, thermal::TegraMonitor};

#[tokio::main]
async fn main() -> cohete::Result<()> {
    // Discover Jetson devices
    let devices = JetsonDevice::discover_all().await?;

    for device in devices {
        let mut monitor = TegraMonitor::connect(&device)?;
        let stats = monitor.sample()?;
        println!("{}: GPU {}°C, {} MB free",
            device.id(),
            stats.gpu_temp,
            stats.available_memory_mb
        );
    }

    Ok(())
}
```

## Declarative Configuration

Full functionality via YAML (Architectural Invariant - no code required):

```yaml
# cohete.yaml
version: "1.0"

fleet:
  name: "edge-cluster"
  devices:
    - id: jetson-01
      connection: usb
      thermal_policy: conservative
      memory_budget_mb: 6000

models:
  - name: mistral-7b-instruct
    source: pacha://models.sovereign.ai/mistral-7b
    quantization: q4_0
    devices: all

thermal:
  conservative:
    threshold_c: 65
    cooldown_c: 55
    check_interval_ms: 500

inference:
  port: 8080
  max_batch_size: 4
  context_length: 2048
  api_compatibility: openai
```

```bash
# Deploy from YAML
cohete deploy --config cohete.yaml
```

## Integration with Sovereign AI Stack

```
┌─────────────────────────────────────────────────────────────┐
│                  BATUTA ORCHESTRATION                       │
├──────────┬──────────┬──────────┬────────────────────────────┤
│ realizar │ repartir │  pacha   │         cohete             │
│ (infer)  │ (sched)  │ (models) │  (Jetson edge hardware)    │
├──────────┴──────────┴──────────┴────────────────────────────┤
│                    trueno (ARM NEON)                        │
└─────────────────────────────────────────────────────────────┘
```

### trueno Integration

```rust
use cohete::device::JetsonDevice;
use trueno::BackendSelector;

let device = JetsonDevice::discover_usb().await?;
let selector = BackendSelector::new()
    .with_device_hint(device.compute_hint());  // Prefers ARM NEON
```

### repartir Integration

```rust
use cohete::fleet::JetsonExecutor;
use repartir::Pool;

let pool = Pool::builder()
    .add_executor(JetsonExecutor::new("192.168.55.1")
        .with_thermal_policy(ThermalPolicy::conservative())
        .with_memory_budget_mb(6000))
    .build()?;
```

## Thermal Safety (Jidoka)

Automatic circuit breaker stops inference when temperature exceeds threshold:

```rust
use cohete::thermal::{ThermalCircuitBreaker, TegraMonitor, ThermalPolicy};

let monitor = TegraMonitor::new().with_policy(ThermalPolicy::conservative());
let mut breaker = ThermalCircuitBreaker::new(monitor);

// Work is automatically paused if GPU temp > 65°C
breaker.guard(async {
    // inference work here
    Ok(())
}).await?;
```

## Memory Management (Poka-Yoke)

Budget-aware allocation prevents OOM on constrained devices:

```rust
use cohete::memory::MemoryBudget;

let budget = MemoryBudget::orin_nano_8gb();  // 8GB total, 2GB reserved

// Fails gracefully if model won't fit
let guard = budget.try_allocate(5000)?;  // 5GB allocation
println!("Available: {} MB", budget.available_mb());
```

## Model Quantization

Memory-aware quantization for edge deployment:

```rust
use cohete::quantize::{JetsonQuantizer, QuantLevel};
use cohete::memory::MemoryBudget;

let budget = MemoryBudget::orin_nano_8gb();

// Auto-select best quantization level for budget
let level = JetsonQuantizer::select_for_budget(14000, &budget);
assert_eq!(level, QuantLevel::Q4_0);  // 7B F16 needs Q4 on 8GB device
```

## Installation

```toml
[dependencies]
cohete = "0.1"

# With full Batuta stack integration
cohete = { version = "0.1", features = ["batuta"] }

# With CLI
cohete = { version = "0.1", features = ["cli"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `arm-neon` | ARM NEON SIMD via trueno (default) |
| `tegra-stats` | tegrastats monitoring |
| `cuda-restricted` | Limited Jetson CUDA support |
| `nvme` | NVMe storage management |
| `batuta` | Full stack integration (repartir, pacha, renacer) |
| `cli` | Command-line interface |
| `full` | All features |

## CLI Usage

```bash
# Discover devices
cohete discover

# Check device status
cohete status

# Interactive setup
cohete setup --interactive

# Deploy from config
cohete deploy --config cohete.yaml

# Monitor fleet
cohete status --fleet
```

## Documentation

- [API Documentation](https://docs.rs/cohete)
- [Specification](docs/specifications/cohete.md) - 100-point Popperian falsification checklist
- [Sovereign AI Stack](https://github.com/paiml/batuta)

## Quality Standards

Following Sovereign AI Stack quality requirements:

- **100% safe Rust** (except FFI quarantine zone)
- **Declarative YAML** - full functionality without code
- **Zero scripting** - no Python/JavaScript in runtime
- **Popperian falsification** - 100-point verification checklist
- **Toyota Way** - Jidoka (circuit breakers), Poka-Yoke (error prevention)

## License

MIT License - See [LICENSE](LICENSE)

## Contributing

1. Fork the repository
2. Create your feature branch from `main`
3. Run `cargo test && cargo clippy -- -D warnings`
4. Ensure specification tests pass
5. Submit a pull request

See the [specification](docs/specifications/cohete.md) for detailed requirements.
