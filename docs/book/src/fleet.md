# Fleet Orchestration

Cohete supports managing multiple Jetson devices as a fleet for distributed inference.

## Creating a Fleet

```rust
use cohete::fleet::Fleet;
use cohete::thermal::ThermalPolicy;

let mut fleet = Fleet::new();
println!("Fleet size: {}", fleet.len());
println!("Is empty: {}", fleet.is_empty());
```

## Adding Devices

```rust
use cohete::device::JetsonDevice;

// Add devices with their thermal policies
let device = JetsonDevice::discover_usb().await?;
fleet.add_device(device, ThermalPolicy::conservative())?;

// Add multiple devices
for ip in ["192.168.1.101", "192.168.1.102", "192.168.1.103"] {
    let device = JetsonDevice::connect(ip.parse()?).await?;
    fleet.add_device(device, ThermalPolicy::aggressive())?;
}

println!("Fleet size: {}", fleet.len());
println!("Enabled devices: {}", fleet.enabled_count());
```

## Fleet Health

Monitor overall fleet status:

```rust
let health = fleet.health_status();

println!("Total devices: {}", health.total_devices);
println!("Enabled: {}", health.enabled_devices);
println!("Healthy: {}", health.healthy_devices);
println!("Degraded: {}", health.degraded_devices);
println!("Offline: {}", health.offline_devices);
println!("Health: {:.1}%", health.health_percent());
```

## Iterating Over Devices

```rust
for member in fleet.devices() {
    let device = &member.device;
    let policy = &member.policy;

    println!("Device: {}", device.id());
    println!("  Model: {}", device.model());
    println!("  Enabled: {}", member.enabled);
    println!("  Thermal threshold: {}°C", policy.threshold_c);
}
```

## Getting Specific Devices

```rust
if let Some(member) = fleet.get("jetson-01") {
    println!("Found: {}", member.device.id());
    println!("Memory: {} MB", member.device.model().memory_mb());
}
```

## Removing Devices

```rust
if let Some(removed) = fleet.remove_device("jetson-usb") {
    println!("Removed: {}", removed.device.id());
}
```

## Model Deployment

Deploy models to all fleet devices:

```rust
// Load model bytes
let model_bytes = std::fs::read("model.gguf")?;

// Deploy to all devices
fleet.deploy_model(&model_bytes).await?;

// Start inference servers
fleet.start_inference_servers().await?;
```

## Deployment Configuration

Configure deployments precisely:

```rust
use cohete::fleet::DeploymentConfig;

let config = DeploymentConfig {
    target_devices: vec!["jetson-01".to_string(), "jetson-02".to_string()],
    quantization: Some("q4_0".to_string()),
    memory_budget_mb: 4000,
    thermal_policy: ThermalPolicy::conservative(),
};
```

## YAML Configuration

Define your fleet in `cohete.yaml`:

```yaml
fleet:
  name: "ml-inference-cluster"
  devices:
    - id: jetson-01
      connection: ethernet
      ip: "192.168.1.101"
      thermal_policy: conservative
      memory_budget_mb: 6000

    - id: jetson-02
      connection: ethernet
      ip: "192.168.1.102"
      thermal_policy: aggressive
      memory_budget_mb: 6000

    - id: jetson-usb
      connection: usb
      thermal_policy: conservative
      memory_budget_mb: 4000
```

## Load Balancing

With repartir integration (batuta feature):

```rust
#[cfg(feature = "batuta")]
use cohete::fleet::JetsonExecutor;

let executor = JetsonExecutor::new("192.168.1.101")
    .with_thermal_policy(ThermalPolicy::conservative())
    .with_memory_budget_mb(4000);

// Register with repartir for load balancing
```

## Best Practices

1. **Use conservative policies** for heterogeneous fleets
2. **Monitor health regularly** - offline devices affect throughput
3. **Balance memory budgets** - account for model + KV cache
4. **Plan for failures** - N+1 redundancy recommended
5. **Centralize configuration** - use YAML for reproducibility
