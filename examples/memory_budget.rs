//! Memory Budget Example
//!
//! Demonstrates memory budget management with Poka-Yoke patterns.
//!
//! Run with: `cargo run --example memory_budget`

use cohete::{
    memory::{MemoryBudget, ModelMemoryEstimate},
    quantize::{JetsonQuantizer, QuantLevel},
    Result,
};

fn main() -> Result<()> {
    println!("=== Cohete Memory Budget Management ===\n");

    // Create budget for Orin Nano 8GB
    let budget = MemoryBudget::orin_nano_8gb();
    println!("Orin Nano 8GB Memory Budget:");
    println!("  Total:     {} MB", budget.total_mb());
    println!("  Available: {} MB", budget.available_mb());
    println!("  Utilization: {:.1}%\n", budget.utilization_percent());

    // Demonstrate allocation tracking (Poka-Yoke)
    println!("=== Memory Allocation Demo ===\n");

    println!("Allocating 2000 MB for model weights...");
    let guard1 = budget.allocate(2000, "model_weights")?;
    println!("  Allocated: {} MB", guard1.size_mb());
    println!("  Available: {} MB\n", budget.available_mb());

    println!("Allocating 1000 MB for KV cache...");
    let guard2 = budget.allocate(1000, "kv_cache")?;
    println!("  Allocated: {} MB", guard2.size_mb());
    println!("  Available: {} MB", budget.available_mb());
    println!("  Utilization: {:.1}%\n", budget.utilization_percent());

    // Try to over-allocate (demonstrates Poka-Yoke)
    println!("Attempting to allocate 5000 MB (should fail)...");
    match budget.allocate(5000, "too_large") {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Correctly prevented: {}\n", e),
    }

    // RAII: guards automatically release on drop
    println!("Dropping KV cache allocation...");
    drop(guard2);
    println!("  Available after drop: {} MB\n", budget.available_mb());

    // Model memory estimation
    println!("=== Model Memory Estimation ===\n");

    let models = [
        ("Llama 2 7B", 7_000_000_000u64),
        ("Llama 2 13B", 13_000_000_000u64),
        ("Mistral 7B", 7_000_000_000u64),
    ];

    for (name, params) in models {
        println!("{}:", name);
        let estimate = ModelMemoryEstimate::from_params(params);
        println!("  F16:  {} MB", estimate.f16_size_mb());
        println!("  Q8_0: {} MB", estimate.quantized_size_mb(QuantLevel::Q8_0));
        println!("  Q4_0: {} MB", estimate.quantized_size_mb(QuantLevel::Q4_0));
        println!();
    }

    // Automatic quantization selection
    println!("=== Auto-Quantization Selection ===\n");

    let budget = MemoryBudget::orin_nano_8gb();
    println!("Budget: {} MB available\n", budget.available_mb());

    let model_sizes = [
        ("Small (4GB F16)", 4000),
        ("Medium (8GB F16)", 8000),
        ("Large (14GB F16)", 14000),
        ("XL (20GB F16)", 20000),
    ];

    for (name, size_mb) in model_sizes {
        let level = JetsonQuantizer::select_for_budget(size_mb, &budget);
        let quantized = (size_mb as f32 * level.memory_factor()) as u64;
        println!("{}: {} -> {} ({} MB)",
            name, level, level.as_str(), quantized);
    }

    // Drop remaining guard
    drop(guard1);

    println!("\n=== Memory Budget Complete ===");
    Ok(())
}
