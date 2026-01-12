//! Thermal Monitoring Example
//!
//! Demonstrates thermal monitoring and circuit breaker patterns (Jidoka).
//!
//! Run with: `cargo run --example thermal_monitoring`

use cohete::{
    thermal::{TegraMonitor, ThermalCircuitBreaker, ThermalPolicy},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Cohete Thermal Monitoring ===\n");

    // Create monitor with conservative policy
    let mut monitor = TegraMonitor::new();
    println!("Created thermal monitor with conservative policy:");
    let policy = ThermalPolicy::conservative();
    println!("  Threshold: {}°C", policy.threshold_c);
    println!("  Cooldown:  {}°C", policy.cooldown_c);
    println!("  Interval:  {}ms\n", policy.check_interval_ms);

    // Sample current stats
    println!("Sampling tegrastats...");
    let stats = monitor.sample()?;
    println!("  GPU Temperature: {}°C", stats.gpu_temp);
    println!("  CPU Temperature: {}°C", stats.cpu_temp);
    println!("  SOC Temperature: {}°C", stats.soc_temp);
    println!("  GPU Utilization: {}%", stats.gpu_utilization);
    println!("  CPU Utilization: {}%", stats.cpu_utilization);
    println!("  Power: {} W", stats.power_watts);
    println!("  Memory: {}/{} MB used ({} MB free)\n",
        stats.used_memory_mb,
        stats.total_memory_mb,
        stats.available_memory_mb
    );

    // Check thermal status
    let throttled = monitor.is_throttled()?;
    println!("Thermal Status: {}\n",
        if throttled { "THROTTLED" } else { "OK" }
    );

    // Demonstrate circuit breaker pattern (Jidoka)
    println!("=== Circuit Breaker Demo (Jidoka Pattern) ===\n");

    let monitor = TegraMonitor::new().with_policy(ThermalPolicy::aggressive());
    let mut breaker = ThermalCircuitBreaker::new(monitor);

    println!("Circuit breaker status: {}",
        if breaker.is_open()? { "OPEN (too hot)" } else { "CLOSED (safe)" }
    );

    // Guard work with thermal protection
    println!("\nExecuting guarded work...");
    let result = breaker.guard(async {
        // Simulate inference work
        println!("  Running inference batch...");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok::<_, cohete::Error>(42)
    }).await?;

    println!("  Work completed with result: {}\n", result);

    // Show different policies
    println!("=== Available Thermal Policies ===\n");

    let policies = [
        ("Conservative", ThermalPolicy::conservative()),
        ("Aggressive", ThermalPolicy::aggressive()),
        ("Custom (80°C)", ThermalPolicy::custom(80.0, 70.0, 250)),
    ];

    for (name, policy) in policies {
        println!("{}:", name);
        println!("  Threshold: {}°C, Cooldown: {}°C, Interval: {}ms",
            policy.threshold_c, policy.cooldown_c, policy.check_interval_ms);
    }

    println!("\n=== Thermal Monitoring Complete ===");
    Ok(())
}
