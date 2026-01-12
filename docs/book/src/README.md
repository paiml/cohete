# Cohete: NVIDIA Jetson Edge Hardware Integration

**Cohete** (Spanish: "rocket") provides safe, pure Rust interfaces to NVIDIA Jetson edge hardware for the Sovereign AI Stack.

## Overview

Cohete enables edge AI deployment on NVIDIA Jetson devices with:

- **Device Discovery**: USB CDC, mDNS, and SSH-based Jetson detection
- **Thermal Monitoring**: tegrastats integration with circuit breakers (Jidoka pattern)
- **Memory Management**: Budget-aware allocation for constrained devices (Poka-Yoke pattern)
- **Power Management**: nvpmodel and jetson_clocks control
- **Fleet Orchestration**: Multi-device management with repartir integration
- **Model Quantization**: Memory-aware quantization for edge deployment

## Supported Hardware

| Model | Memory | CUDA Cores | AI Performance |
|-------|--------|------------|----------------|
| Orin Nano 4GB | 4 GB | 512 | 20 TOPS |
| Orin Nano 8GB | 8 GB | 1024 | 40 TOPS |
| Orin NX 8GB | 8 GB | 1024 | 70 TOPS |
| Orin NX 16GB | 16 GB | 1024 | 100 TOPS |
| AGX Orin 32GB | 32 GB | 2048 | 200 TOPS |
| AGX Orin 64GB | 64 GB | 2048 | 275 TOPS |

## Toyota Way Principles

Cohete embeds Toyota Production System principles:

- **Jidoka (自働化)**: Thermal circuit breakers automatically stop work when temperature exceeds thresholds
- **Poka-Yoke (ポカヨケ)**: Memory guards prevent allocation failures through compile-time and runtime checks
- **Kaizen (改善)**: Continuous monitoring enables iterative optimization
- **Muda (無駄)**: Quantization eliminates memory waste

## Quick Example

```rust
use cohete::{device::JetsonDevice, thermal::TegraMonitor};

#[tokio::main]
async fn main() -> cohete::Result<()> {
    // Discover Jetson devices
    let devices = JetsonDevice::discover_all().await?;

    for device in devices {
        let mut monitor = TegraMonitor::connect(&device)?;
        let stats = monitor.sample()?;
        println!("{}: GPU {}°C, Memory {}MB free",
            device.id(),
            stats.gpu_temp,
            stats.available_memory_mb
        );
    }

    Ok(())
}
```

## Architectural Invariant

> All cohete functionality is accessible via declarative YAML configuration.
> No Rust code is required for standard deployments.

```yaml
fleet:
  devices:
    - id: jetson-01
      connection: usb
      thermal_policy: conservative
      memory_budget_mb: 6000

models:
  - name: llama-7b
    source: "pacha://models/llama-2-7b-chat"
    quantization: q4_0
```

## License

MIT License - See [LICENSE](https://github.com/paiml/cohete/blob/main/LICENSE) for details.
