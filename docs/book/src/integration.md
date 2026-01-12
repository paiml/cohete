# Sovereign AI Stack Integration

Cohete integrates with other components of the Sovereign AI Stack for complete edge AI deployment.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Sovereign AI Stack                        │
├─────────────────────────────────────────────────────────────┤
│  batuta (Conductor) - Orchestration                         │
│    ├── repartir - Task distribution & load balancing        │
│    ├── pacha - Secure model deployment (Ed25519 signed)     │
│    ├── realizar - Inference server                          │
│    └── renacer - Tracing & observability                    │
├─────────────────────────────────────────────────────────────┤
│  trueno (Thunder) - Compute                                 │
│    └── ARM NEON SIMD backend for Jetson                     │
├─────────────────────────────────────────────────────────────┤
│  cohete (Rocket) - Hardware                                 │
│    ├── Device discovery                                     │
│    ├── Thermal management                                   │
│    ├── Memory budgets                                       │
│    └── Fleet orchestration                                  │
└─────────────────────────────────────────────────────────────┘
```

## trueno Integration

Cohete provides compute hints for trueno backend selection:

```rust
use cohete::device::JetsonDevice;

let device = JetsonDevice::discover_usb().await?;
let hint = device.compute_hint();

// Use with trueno
println!("Prefer NEON: {}", hint.prefer_neon);
println!("Memory budget: {} MB", hint.memory_budget_mb);
println!("CUDA available: {}", hint.cuda_available);
```

trueno automatically selects ARM NEON SIMD on Jetson when available.

## batuta Integration

Enable with the `batuta` feature:

```toml
[dependencies]
cohete = { version = "0.1", features = ["batuta"] }
```

### repartir (Task Distribution)

Register Jetson devices as remote executors:

```rust
#[cfg(feature = "batuta")]
use cohete::fleet::JetsonExecutor;

let executor = JetsonExecutor::new("192.168.1.101")
    .with_thermal_policy(ThermalPolicy::conservative())
    .with_memory_budget_mb(4000);
```

### pacha (Model Deployment)

Models are referenced via `pacha://` URLs in configuration:

```yaml
models:
  - name: llama-7b
    source: "pacha://models/llama-2-7b-chat"
    quantization: q4_0
```

Pacha handles:
- Ed25519 signature verification
- Secure transport
- Model caching on NVMe

### renacer (Observability)

Cohete emits tracing spans compatible with renacer:

```rust
use tracing::{info, instrument};

#[instrument]
async fn run_inference() {
    info!(temp_c = 65.0, "Thermal status");
    // ...
}
```

## Feature Flags

| Feature | Dependencies |
|---------|--------------|
| `batuta` | repartir, pacha, renacer |
| `arm-neon` | (trueno auto-detection) |
| `full` | All integrations |

## Deployment Flow

1. **Discovery**: Cohete discovers Jetson devices
2. **Registration**: Devices registered with repartir
3. **Deployment**: pacha deploys models securely
4. **Inference**: realizar runs inference with trueno
5. **Monitoring**: renacer collects thermal/performance traces

## Without Batuta

Cohete works standalone without batuta integration:

```rust
// Core functionality always available
let devices = JetsonDevice::discover_all().await?;
let monitor = TegraMonitor::new();
let budget = MemoryBudget::orin_nano_8gb();
```
