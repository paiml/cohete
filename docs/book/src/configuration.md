# Configuration

Cohete uses declarative YAML configuration as an **Architectural Invariant** - all functionality is accessible without writing Rust code.

## Configuration File

By default, cohete looks for `cohete.yaml` in the current directory:

```rust
use cohete::config::CoheteConfig;

let config = CoheteConfig::load("cohete.yaml")?;
println!("Version: {}", config.version);
```

## Complete Example

```yaml
version: "1.0"

discovery:
  methods:
    - usb
    - mdns
    - static:
        - "192.168.1.101"
        - "192.168.1.102"

fleet:
  name: "ml-inference-cluster"
  devices:
    - id: jetson-01
      connection: ethernet
      ip: "192.168.1.101"
      thermal_policy: conservative
      memory_budget_mb: 6000
    - id: jetson-02
      connection: usb
      thermal_policy: aggressive
      memory_budget_mb: 4000

models:
  - name: llama-7b
    source: "pacha://models/llama-2-7b-chat"
    quantization: q4_0
    devices: all
  - name: phi-2
    source: "pacha://models/phi-2"
    quantization: q8_0
    devices: jetson-01

thermal:
  conservative:
    threshold_c: 65.0
    cooldown_c: 55.0
    check_interval_ms: 500
  aggressive:
    threshold_c: 75.0
    cooldown_c: 65.0
    check_interval_ms: 1000

inference:
  port: 8080
  max_batch_size: 4
  context_length: 2048
  api_compatibility: openai

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
    - tmux
```

## Configuration Sections

### Discovery

Configure how devices are discovered:

```yaml
discovery:
  methods:
    - usb           # Check USB-C (192.168.55.1)
    - mdns          # mDNS service discovery
    - static:       # Known IP addresses
        - "192.168.1.101"
```

### Fleet

Define your device fleet:

```yaml
fleet:
  name: "production-cluster"
  devices:
    - id: jetson-01
      connection: ethernet  # or usb
      ip: "192.168.1.101"   # required for ethernet
      thermal_policy: conservative  # or aggressive
      memory_budget_mb: 6000
```

### Models

Specify models to deploy:

```yaml
models:
  - name: llama-7b
    source: "pacha://models/llama-2-7b-chat"  # pacha:// URL
    quantization: q4_0  # q4_0, q4_1, q5_0, q5_1, q8_0, f16
    devices: all        # or specific device IDs
```

### Thermal

Custom thermal policies:

```yaml
thermal:
  conservative:
    threshold_c: 65.0
    cooldown_c: 55.0
    check_interval_ms: 500
```

### Inference

Server configuration:

```yaml
inference:
  port: 8080
  max_batch_size: 4
  context_length: 2048
  api_compatibility: openai  # OpenAI-compatible API
```

### Provision

Setup and provisioning:

```yaml
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

## Programmatic Configuration

Create and modify configuration in code:

```rust
use cohete::config::CoheteConfig;

// Load from file
let config = CoheteConfig::load("cohete.yaml")?;

// Create default
let config = CoheteConfig::default();

// Parse from string
let yaml = std::fs::read_to_string("cohete.yaml")?;
let config = CoheteConfig::from_yaml(&yaml)?;

// Serialize to YAML
let yaml = config.to_yaml()?;
println!("{}", yaml);

// Save to file
config.save("cohete-new.yaml")?;
```

## Converting to Runtime Types

```rust
use cohete::thermal::ThermalPolicy;

// YAML policy to runtime policy
let policy: ThermalPolicy = config.thermal.conservative.into();
println!("Threshold: {}°C", policy.threshold_c);
```
