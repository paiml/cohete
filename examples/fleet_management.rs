//! Fleet Management Example
//!
//! Demonstrates multi-device fleet orchestration.
//!
//! Run with: `cargo run --example fleet_management`

use cohete::{
    device::{ConnectionMethod, DeviceInfo, JetsonDevice},
    fleet::{DeploymentConfig, Fleet},
    thermal::ThermalPolicy,
    JetsonModel, Result,
};

fn make_device(id: &str, model: JetsonModel, ip: Option<&str>) -> JetsonDevice {
    JetsonDevice {
        info: DeviceInfo {
            id: id.to_string(),
            model,
            connection: match ip {
                Some(addr) => ConnectionMethod::Ethernet(addr.parse().unwrap()),
                None => ConnectionMethod::Usb,
            },
            jetpack_version: Some("5.1.2".to_string()),
            hostname: Some(format!("{}.local", id)),
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Cohete Fleet Management ===\n");

    // Create a fleet
    let mut fleet = Fleet::new();
    println!("Created empty fleet");
    println!("  Devices: {}", fleet.len());
    println!("  Is empty: {}\n", fleet.is_empty());

    // Add devices to fleet
    println!("Adding devices to fleet...\n");

    let devices = [
        ("jetson-01", JetsonModel::OrinNano8GB, Some("192.168.1.101")),
        ("jetson-02", JetsonModel::OrinNano8GB, Some("192.168.1.102")),
        ("jetson-03", JetsonModel::OrinNX16GB, Some("192.168.1.103")),
        ("jetson-usb", JetsonModel::OrinNano4GB, None),
    ];

    for (id, model, ip) in devices {
        let device = make_device(id, model, ip);
        let policy = if model == JetsonModel::OrinNX16GB {
            ThermalPolicy::aggressive()
        } else {
            ThermalPolicy::conservative()
        };

        fleet.add_device(device, policy)?;
        println!("  Added: {} ({}, {:?})",
            id, model,
            if ip.is_some() { "Ethernet" } else { "USB" }
        );
    }

    println!("\nFleet Status:");
    println!("  Total devices: {}", fleet.len());
    println!("  Enabled: {}", fleet.enabled_count());

    // Fleet health
    println!("\n=== Fleet Health ===\n");
    let health = fleet.health_status();
    println!("Total:    {} devices", health.total_devices);
    println!("Enabled:  {} devices", health.enabled_devices);
    println!("Healthy:  {} devices", health.healthy_devices);
    println!("Degraded: {} devices", health.degraded_devices);
    println!("Offline:  {} devices", health.offline_devices);
    println!("Health:   {:.1}%", health.health_percent());

    // Iterate over devices
    println!("\n=== Device Details ===\n");
    for member in fleet.devices() {
        let device = &member.device;
        let policy = &member.policy;
        println!("{}:", device.id());
        println!("  Model: {}", device.model());
        println!("  Memory: {} MB", device.model().memory_mb());
        println!("  Thermal Policy: {}°C threshold", policy.threshold_c);
        println!("  Enabled: {}", member.enabled);
        println!();
    }

    // Get specific device
    if let Some(member) = fleet.get("jetson-03") {
        println!("jetson-03 compute hints:");
        let hint = member.device.compute_hint();
        println!("  Memory Budget: {} MB", hint.memory_budget_mb);
        println!("  CUDA: {}", hint.cuda_available);
    }

    // Deployment configuration
    println!("\n=== Deployment Configuration ===\n");
    let config = DeploymentConfig {
        target_devices: vec!["jetson-01".to_string(), "jetson-02".to_string()],
        quantization: Some("q4_0".to_string()),
        memory_budget_mb: 4000,
        thermal_policy: ThermalPolicy::conservative(),
    };

    println!("Deployment Config:");
    println!("  Targets: {:?}", config.target_devices);
    println!("  Quantization: {:?}", config.quantization);
    println!("  Memory Budget: {} MB", config.memory_budget_mb);
    println!("  Thermal: {}°C threshold", config.thermal_policy.threshold_c);

    // Deploy model
    println!("\n=== Model Deployment ===\n");
    println!("Deploying model to fleet...");
    fleet.deploy_model(&[/* model bytes */]).await?;
    println!("  Model deployed successfully");

    // Start inference servers
    println!("\nStarting inference servers...");
    fleet.start_inference_servers().await?;
    println!("  Servers started on all devices");

    // Remove a device
    println!("\nRemoving jetson-usb from fleet...");
    if let Some(removed) = fleet.remove_device("jetson-usb") {
        println!("  Removed: {}", removed.device.id());
    }
    println!("  Fleet size: {}", fleet.len());

    println!("\n=== Fleet Management Complete ===");
    Ok(())
}
