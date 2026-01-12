# Memory Management

Jetson devices have unified memory shared between CPU and GPU. Cohete provides budget-aware allocation using the Poka-Yoke (ポカヨケ) pattern to prevent OOM conditions.

## Memory Budgets

Create budgets for specific Jetson models:

```rust
use cohete::memory::MemoryBudget;

// Pre-configured budgets
let budget = MemoryBudget::orin_nano_8gb();   // 8192 MB total, 6144 MB available
let budget = MemoryBudget::orin_nano_4gb();   // 4096 MB total, 3072 MB available
let budget = MemoryBudget::orin_nx_16gb();    // 16384 MB total, 14336 MB available
let budget = MemoryBudget::agx_orin_32gb();   // 32768 MB total, 28672 MB available
let budget = MemoryBudget::agx_orin_64gb();   // 65536 MB total, 57344 MB available

// Custom budget
let budget = MemoryBudget::new(
    8192,   // Total MB
    2048    // Reserved for system MB
);
```

## Budget Queries

```rust
let budget = MemoryBudget::orin_nano_8gb();

println!("Total: {} MB", budget.total_mb());
println!("Usable: {} MB", budget.usable_mb());
println!("Available: {} MB", budget.available_mb());
println!("Allocated: {} MB", budget.allocated_mb());
println!("Utilization: {:.1}%", budget.utilization_percent());
```

## RAII Guards (Poka-Yoke)

Allocations return guards that automatically release memory when dropped:

```rust
let budget = MemoryBudget::orin_nano_8gb();

// Allocate with label for debugging
let weights = budget.allocate(2000, "model_weights")?;
println!("Allocated {} MB", weights.size_mb());

// Check remaining
println!("Available: {} MB", budget.available_mb());

// Memory automatically released on drop
drop(weights);
println!("After drop: {} MB", budget.available_mb());
```

## Preventing Over-Allocation

The budget prevents allocations that would exceed available memory:

```rust
let budget = MemoryBudget::orin_nano_8gb(); // 6144 MB available

// This succeeds
let guard = budget.allocate(5000, "large_model")?;

// This fails - not enough memory
match budget.allocate(2000, "kv_cache") {
    Ok(_) => println!("Allocated"),
    Err(cohete::Error::InsufficientMemory { requested_mb, available_mb }) => {
        println!("Cannot allocate {} MB, only {} MB available",
            requested_mb, available_mb);
    }
    Err(e) => return Err(e),
}
```

## Pre-Check Allocation

Check if allocation would succeed without actually allocating:

```rust
if budget.can_allocate(4000) {
    let guard = budget.allocate(4000, "model")?;
    // Use guard...
} else {
    println!("Not enough memory for this model");
}
```

## Model Memory Estimation

Estimate memory requirements for LLMs:

```rust
use cohete::memory::ModelMemoryEstimate;
use cohete::quantize::QuantLevel;

// From parameter count
let estimate = ModelMemoryEstimate::from_params(7_000_000_000); // 7B params

println!("F16 size: {} MB", estimate.f16_size_mb());
println!("Q8_0 size: {} MB", estimate.quantized_size_mb(QuantLevel::Q8_0));
println!("Q4_0 size: {} MB", estimate.quantized_size_mb(QuantLevel::Q4_0));

// Check if model fits
let budget = MemoryBudget::orin_nano_8gb();
if estimate.fits_in(&budget, 2048) {  // 2048 context length
    println!("Model fits!");
}
```

## Multiple Allocations

Track multiple concurrent allocations:

```rust
let budget = MemoryBudget::orin_nano_8gb();

let weights = budget.allocate(3000, "weights")?;
let kv_cache = budget.allocate(1000, "kv_cache")?;
let activations = budget.allocate(500, "activations")?;

println!("Total allocated: {} MB", budget.allocated_mb());
println!("Utilization: {:.1}%", budget.utilization_percent());

// All released when guards go out of scope
```

## Best Practices

1. **Reserve memory for system** - Default reservations account for JetPack overhead
2. **Use RAII guards** - Never manually track allocations
3. **Check before allocate** - Use `can_allocate()` for optional allocations
4. **Consider context length** - KV cache grows linearly with context
5. **Profile with tegrastats** - Verify actual memory usage matches estimates
