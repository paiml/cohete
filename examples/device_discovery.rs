//! Device Discovery Example
//!
//! Demonstrates discovering and connecting to Jetson devices.
//!
//! Run with: `cargo run --example device_discovery`

use cohete::{device::JetsonDevice, JetsonModel, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Cohete Device Discovery ===\n");

    // Discover all devices (USB + mDNS)
    println!("Discovering Jetson devices...");
    let devices = JetsonDevice::discover_all().await?;

    if devices.is_empty() {
        println!("No devices found.");
        println!("\nTip: Connect a Jetson via USB-C or ensure mDNS is enabled.");
    } else {
        println!("Found {} device(s):\n", devices.len());

        for device in &devices {
            let info = device.info();
            println!("  Device: {}", device.id());
            println!("  Model:  {}", info.model);
            println!("  Connection: {:?}", info.connection);

            if info.model != JetsonModel::Unknown {
                println!("  Memory: {} MB", info.model.memory_mb());
                println!("  CUDA Cores: {}", info.model.cuda_cores());
                println!("  AI Performance: {} TOPS", info.model.tops());
            }

            // Get compute hints for trueno integration
            let hint = device.compute_hint();
            println!("\n  Compute Hints:");
            println!("    Prefer NEON: {}", hint.prefer_neon);
            println!("    Memory Budget: {} MB", hint.memory_budget_mb);
            println!("    CUDA Available: {}", hint.cuda_available);
            println!();
        }
    }

    // Try direct USB connection
    println!("Attempting direct USB connection...");
    match JetsonDevice::discover_usb().await {
        Ok(device) => {
            println!("  USB device found: {}", device.id());
            let mem = device.available_memory_mb().await?;
            println!("  Available memory: {} MB", mem);
        }
        Err(e) => println!("  USB connection failed: {}", e),
    }

    println!("\n=== Discovery Complete ===");
    Ok(())
}
