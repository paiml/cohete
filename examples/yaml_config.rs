//! YAML Configuration Example
//!
//! Demonstrates declarative configuration (Architectural Invariant).
//!
//! Run with: `cargo run --example yaml_config`

use cohete::{
    config::{CoheteConfig, DiscoveryMethod},
    thermal::ThermalPolicy,
    Result,
};

fn main() -> Result<()> {
    println!("=== Cohete YAML Configuration ===\n");

    // Parse configuration from YAML
    let yaml = r#"
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
    devices: all

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
"#;

    println!("Parsing YAML configuration...\n");
    let config = CoheteConfig::from_yaml(yaml)?;

    // Display parsed configuration
    println!("Configuration Version: {}\n", config.version);

    // Discovery methods
    println!("Discovery Methods:");
    for method in &config.discovery.methods {
        match method {
            DiscoveryMethod::Usb => println!("  - USB CDC"),
            DiscoveryMethod::Mdns => println!("  - mDNS"),
            DiscoveryMethod::Static(ips) => {
                println!("  - Static IPs: {:?}", ips);
            }
        }
    }

    // Fleet configuration
    println!("\nFleet: {}", config.fleet.name);
    println!("Devices:");
    for device in &config.fleet.devices {
        println!("  {}:", device.id);
        println!("    Connection: {}", device.connection);
        if let Some(ip) = &device.ip {
            println!("    IP: {}", ip);
        }
        println!("    Thermal Policy: {}", device.thermal_policy);
        println!("    Memory Budget: {} MB", device.memory_budget_mb);
    }

    // Models
    println!("\nModels:");
    for model in &config.models {
        println!("  {}:", model.name);
        println!("    Source: {}", model.source);
        println!("    Quantization: {:?}", model.quantization);
        println!("    Devices: {}", model.devices);
    }

    // Thermal policies
    println!("\nThermal Policies:");
    println!("  Conservative: {}°C / {}°C",
        config.thermal.conservative.threshold_c,
        config.thermal.conservative.cooldown_c);
    println!("  Aggressive: {}°C / {}°C",
        config.thermal.aggressive.threshold_c,
        config.thermal.aggressive.cooldown_c);

    // Convert YAML thermal to runtime policy
    let policy: ThermalPolicy = config.thermal.conservative.clone().into();
    println!("\n  Converted to runtime: threshold={}°C", policy.threshold_c);

    // Inference settings
    println!("\nInference Server:");
    println!("  Port: {}", config.inference.port);
    println!("  Max Batch Size: {}", config.inference.max_batch_size);
    println!("  Context Length: {}", config.inference.context_length);
    println!("  API Compatibility: {}", config.inference.api_compatibility);

    // Provisioning
    println!("\nProvisioning:");
    println!("  NVMe: {} ({})",
        if config.provision.nvme.enabled { "enabled" } else { "disabled" },
        config.provision.nvme.mount_point);
    println!("  Swap: {} GB", config.provision.nvme.swap_size_gb);
    println!("  SSH Host: {}", config.provision.ssh.config_host);
    println!("  Packages: {:?}", config.provision.packages);

    // Serialize back to YAML
    println!("\n=== Roundtrip Test ===\n");
    let serialized = config.to_yaml()?;
    println!("Serialized {} bytes of YAML", serialized.len());

    // Parse again to verify
    let reparsed = CoheteConfig::from_yaml(&serialized)?;
    println!("Roundtrip successful: version={}", reparsed.version);

    // Create default configuration
    println!("\n=== Default Configuration ===\n");
    let default_config = CoheteConfig::default();
    println!("Default version: {}", default_config.version);
    println!("Default inference port: {}", default_config.inference.port);

    println!("\n=== YAML Configuration Complete ===");
    Ok(())
}
