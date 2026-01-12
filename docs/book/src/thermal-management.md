# Thermal Management

Thermal management is critical for edge AI on Jetson devices. Cohete provides comprehensive thermal monitoring and automatic work suspension via the Jidoka (自働化) pattern.

## tegrastats Integration

The `TegraMonitor` provides access to tegrastats data:

```rust
use cohete::thermal::TegraMonitor;

let mut monitor = TegraMonitor::new();
let stats = monitor.sample()?;

println!("GPU Temperature: {}°C", stats.gpu_temp);
println!("CPU Temperature: {}°C", stats.cpu_temp);
println!("SOC Temperature: {}°C", stats.soc_temp);
println!("GPU Utilization: {}%", stats.gpu_utilization);
println!("Power Consumption: {} W", stats.power_watts);
println!("Memory: {}/{} MB", stats.used_memory_mb, stats.total_memory_mb);
```

## Thermal Policies

Cohete provides pre-defined thermal policies:

### Conservative (Default)

```rust
use cohete::thermal::ThermalPolicy;

let policy = ThermalPolicy::conservative();
// Threshold: 65°C
// Cooldown: 55°C
// Check Interval: 500ms
```

Safe for sustained workloads in enclosed environments.

### Aggressive

```rust
let policy = ThermalPolicy::aggressive();
// Threshold: 75°C
// Cooldown: 65°C
// Check Interval: 1000ms
```

For maximum performance with good cooling.

### Custom

```rust
let policy = ThermalPolicy::custom(
    70.0,   // Threshold °C
    60.0,   // Cooldown °C
    250     // Check interval ms
);
```

## Circuit Breaker Pattern (Jidoka)

The `ThermalCircuitBreaker` automatically stops work when temperature exceeds the threshold:

```rust
use cohete::thermal::{TegraMonitor, ThermalCircuitBreaker, ThermalPolicy};

let monitor = TegraMonitor::new()
    .with_policy(ThermalPolicy::conservative());
let mut breaker = ThermalCircuitBreaker::new(monitor);

// Check if circuit is open (too hot)
if breaker.is_open()? {
    println!("Device is thermal throttled!");
}

// Guard work with thermal protection
let result = breaker.guard(async {
    // Your inference work here
    run_inference_batch().await
}).await?;
```

The `guard()` method:
1. Checks temperature before starting
2. Waits for cooldown if too hot
3. Executes the work
4. Returns the result

## Continuous Monitoring

For long-running processes:

```rust
loop {
    let throttled = monitor.is_throttled()?;

    if throttled {
        println!("Throttled! Waiting for cooldown...");
        monitor.wait_for_cooldown().await?;
        println!("Cooled down, resuming work");
    }

    // Do work
    process_batch().await?;
}
```

## YAML Configuration

Configure thermal policies in `cohete.yaml`:

```yaml
thermal:
  conservative:
    threshold_c: 65.0
    cooldown_c: 55.0
    check_interval_ms: 500
  aggressive:
    threshold_c: 75.0
    cooldown_c: 65.0
    check_interval_ms: 1000

fleet:
  devices:
    - id: jetson-01
      thermal_policy: conservative
    - id: jetson-02
      thermal_policy: aggressive
```

## Best Practices

1. **Use conservative policy** for 24/7 deployments
2. **Monitor GPU temperature** - it's the primary heat source during inference
3. **Ensure adequate cooling** - the Orin Nano's heatsink is undersized for sustained loads
4. **Set swappiness low** (10 or less) to avoid thermal-inducing swap thrashing
5. **Consider fan control** via `PowerProfile::max_performance()` for critical workloads
