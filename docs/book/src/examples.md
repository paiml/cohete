# Examples

Cohete includes several runnable examples demonstrating key functionality.

## Running Examples

```bash
# Clone the repository
git clone https://github.com/paiml/cohete
cd cohete

# Run any example
cargo run --example <example_name>
```

## Available Examples

### device_discovery

Discover and connect to Jetson devices:

```bash
cargo run --example device_discovery
```

Demonstrates:
- USB-C device discovery
- mDNS discovery
- Device information retrieval
- Compute hints for trueno

### thermal_monitoring

Monitor thermal status and circuit breakers:

```bash
cargo run --example thermal_monitoring
```

Demonstrates:
- tegrastats sampling
- Thermal policies (conservative, aggressive, custom)
- Circuit breaker pattern (Jidoka)
- Guarded work execution

### memory_budget

Memory budget management:

```bash
cargo run --example memory_budget
```

Demonstrates:
- Memory budgets for different Jetson models
- RAII allocation guards (Poka-Yoke)
- Over-allocation prevention
- Model memory estimation

### fleet_management

Multi-device fleet orchestration:

```bash
cargo run --example fleet_management
```

Demonstrates:
- Creating and managing a fleet
- Adding/removing devices
- Fleet health monitoring
- Model deployment
- Deployment configuration

### quantization

Model quantization selection:

```bash
cargo run --example quantization
```

Demonstrates:
- All quantization levels and their properties
- Memory budgets by Jetson model
- Automatic quantization selection
- Model fitting analysis

### yaml_config

Declarative YAML configuration:

```bash
cargo run --example yaml_config
```

Demonstrates:
- Parsing YAML configuration
- All configuration sections
- Converting to runtime types
- Roundtrip serialization

## Example Output

### quantization

```
=== Cohete Model Quantization ===

Available Quantization Levels:

Level      Bits Memory Factor Perplexity Delta
---------------------------------------------
q4_0          4       0.2500            5.0%
q4_1          4       0.2500            4.0%
q5_0          5       0.3125            3.0%
q5_1          5       0.3125            2.5%
q8_0          8       0.5000            1.0%
f16          16       1.0000            0.0%
f32          32       2.0000            0.0%

=== Model Fitting Analysis ===

Model               Nano 8GB      NX 16GB     AGX 64GB
------------------------------------------------------
Llama 2 7B              q5_1          f16          f16
Llama 2 13B             q4_0         q8_0          f16
Mistral 7B              q5_1          f16          f16
Phi-2 2.7B               f16          f16          f16
Gemma 2B                 f16          f16          f16
```

## Creating Your Own Examples

```rust
//! My Example
//!
//! Run with: `cargo run --example my_example`

use cohete::{device::JetsonDevice, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let devices = JetsonDevice::discover_all().await?;
    for device in devices {
        println!("Found: {}", device.id());
    }
    Ok(())
}
```

Save as `examples/my_example.rs` and run with:

```bash
cargo run --example my_example
```
