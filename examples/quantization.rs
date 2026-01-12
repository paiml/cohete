//! Model Quantization Example
//!
//! Demonstrates memory-aware quantization selection for edge deployment.
//!
//! Run with: `cargo run --example quantization`

use cohete::{
    memory::MemoryBudget,
    quantize::{JetsonQuantizer, QuantLevel, QuantResult},
    Result,
};

fn main() -> Result<()> {
    println!("=== Cohete Model Quantization ===\n");

    // Show all quantization levels
    println!("Available Quantization Levels:\n");
    let levels = [
        QuantLevel::Q4_0,
        QuantLevel::Q4_1,
        QuantLevel::Q5_0,
        QuantLevel::Q5_1,
        QuantLevel::Q8_0,
        QuantLevel::F16,
        QuantLevel::F32,
    ];

    println!("{:<8} {:>6} {:>12} {:>15}",
        "Level", "Bits", "Memory Factor", "Perplexity Delta");
    println!("{}", "-".repeat(45));

    for level in levels {
        println!("{:<8} {:>6} {:>12.4} {:>14.1}%",
            level.as_str(),
            level.bits_per_param(),
            level.memory_factor(),
            level.perplexity_delta_percent()
        );
    }

    // Memory budget comparison
    println!("\n=== Memory Budget by Jetson Model ===\n");

    let budgets = [
        ("Orin Nano 4GB", MemoryBudget::orin_nano_4gb()),
        ("Orin Nano 8GB", MemoryBudget::orin_nano_8gb()),
        ("Orin NX 16GB", MemoryBudget::orin_nx_16gb()),
        ("AGX Orin 32GB", MemoryBudget::agx_orin_32gb()),
        ("AGX Orin 64GB", MemoryBudget::agx_orin_64gb()),
    ];

    println!("{:<16} {:>12} {:>12}", "Device", "Total MB", "Available MB");
    println!("{}", "-".repeat(42));

    for (name, budget) in &budgets {
        println!("{:<16} {:>12} {:>12}", name, budget.total_mb(), budget.available_mb());
    }

    // Model fitting analysis
    println!("\n=== Model Fitting Analysis ===\n");
    println!("Which models fit on which devices?\n");

    let models = [
        ("Llama 2 7B", 14000),   // F16 size in MB
        ("Llama 2 13B", 26000),
        ("Mistral 7B", 14000),
        ("Phi-2 2.7B", 5400),
        ("Gemma 2B", 4000),
    ];

    let devices = [
        ("Nano 8GB", MemoryBudget::orin_nano_8gb()),
        ("NX 16GB", MemoryBudget::orin_nx_16gb()),
        ("AGX 64GB", MemoryBudget::agx_orin_64gb()),
    ];

    // Header
    print!("{:<15}", "Model");
    for (name, _) in &devices {
        print!(" {:>12}", name);
    }
    println!();
    println!("{}", "-".repeat(15 + 13 * devices.len()));

    // Model rows
    for (model_name, f16_size) in models {
        print!("{:<15}", model_name);
        for (_, budget) in &devices {
            let level = JetsonQuantizer::select_for_budget(f16_size, budget);
            print!(" {:>12}", level.as_str());
        }
        println!();
    }

    // Quantization demo
    println!("\n=== Quantization Demo ===\n");

    let quantizer = JetsonQuantizer::new(QuantLevel::Q4_0)
        .with_target_memory_mb(4000);

    println!("Quantizer Configuration:");
    println!("  Level: {}", quantizer.level());

    // Perform quantization
    let result: QuantResult = quantizer.quantize(&[])?;

    println!("\nQuantization Result:");
    println!("  Original Size:  {} MB", result.original_size_mb);
    println!("  Quantized Size: {} MB", result.quantized_size_mb);
    println!("  Compression:    {:.2}x", result.compression_ratio());
    println!("  Est. Perplexity Delta: {:.1}%", result.estimated_perplexity_delta);

    // Best quantization for specific scenario
    println!("\n=== Scenario: Deploy Llama 2 7B on Orin Nano 8GB ===\n");

    let budget = MemoryBudget::orin_nano_8gb();
    let llama_7b_f16_mb = 14000;

    println!("Constraints:");
    println!("  Model F16 size: {} MB", llama_7b_f16_mb);
    println!("  Available memory: {} MB", budget.available_mb());

    let optimal = JetsonQuantizer::select_for_budget(llama_7b_f16_mb, &budget);
    let quantized_size = (llama_7b_f16_mb as f32 * optimal.memory_factor()) as u64;

    println!("\nRecommendation:");
    println!("  Quantization: {} ({})", optimal, optimal.as_str());
    println!("  Expected size: {} MB", quantized_size);
    println!("  Perplexity impact: +{:.1}%", optimal.perplexity_delta_percent());
    println!("  Fits in budget: {}", quantized_size <= budget.available_mb());

    println!("\n=== Quantization Complete ===");
    Ok(())
}
