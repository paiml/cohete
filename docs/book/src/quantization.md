# Model Quantization

Quantization reduces model memory footprint at the cost of some accuracy. Cohete helps select the optimal quantization level for your Jetson's memory constraints.

## Quantization Levels

Cohete supports llama.cpp-compatible quantization levels:

| Level | Bits | Memory Factor | Perplexity Delta |
|-------|------|---------------|------------------|
| Q4_0 | 4 | 0.25x | +5.0% |
| Q4_1 | 4 | 0.25x | +4.0% |
| Q5_0 | 5 | 0.3125x | +3.0% |
| Q5_1 | 5 | 0.3125x | +2.5% |
| Q8_0 | 8 | 0.50x | +1.0% |
| F16 | 16 | 1.0x | 0% |
| F32 | 32 | 2.0x | 0% |

```rust
use cohete::quantize::QuantLevel;

let level = QuantLevel::Q4_0;
println!("Bits: {}", level.bits_per_param());
println!("Memory factor: {}", level.memory_factor());
println!("Perplexity delta: {}%", level.perplexity_delta_percent());
```

## Automatic Selection

Let cohete select the best quantization for your memory budget:

```rust
use cohete::memory::MemoryBudget;
use cohete::quantize::JetsonQuantizer;

let budget = MemoryBudget::orin_nano_8gb();
let model_f16_size_mb = 14000; // 7B model

let level = JetsonQuantizer::select_for_budget(model_f16_size_mb, &budget);
println!("Recommended: {}", level); // q5_1 for Orin Nano 8GB
```

The selection algorithm tries levels from highest quality (F16) to lowest (Q4_0), returning the first that fits.

## Manual Quantization

Create a quantizer with a specific level:

```rust
use cohete::quantize::{JetsonQuantizer, QuantLevel};

let quantizer = JetsonQuantizer::new(QuantLevel::Q4_0)
    .with_target_memory_mb(4000);

let result = quantizer.quantize(&model_bytes)?;

println!("Original: {} MB", result.original_size_mb);
println!("Quantized: {} MB", result.quantized_size_mb);
println!("Compression: {:.2}x", result.compression_ratio());
println!("Est. perplexity delta: {}%", result.estimated_perplexity_delta);
```

## Model Fitting by Device

Here's what fits on each Jetson:

| Model | Orin Nano 8GB | Orin NX 16GB | AGX Orin 64GB |
|-------|---------------|--------------|---------------|
| Llama 2 7B (14GB F16) | Q5_1 | F16 | F16 |
| Llama 2 13B (26GB F16) | Q4_0 | Q8_0 | F16 |
| Mistral 7B (14GB F16) | Q5_1 | F16 | F16 |
| Phi-2 2.7B (5.4GB F16) | F16 | F16 | F16 |
| Gemma 2B (4GB F16) | F16 | F16 | F16 |

## YAML Configuration

Specify quantization in your deployment config:

```yaml
models:
  - name: llama-7b
    source: "pacha://models/llama-2-7b-chat"
    quantization: q4_0
    devices: all

  - name: phi-2
    source: "pacha://models/phi-2"
    quantization: q8_0  # Higher quality for smaller model
    devices: all
```

## Quality vs Memory Tradeoffs

**Rule of thumb:**
- **Q4_0/Q4_1**: Maximum memory savings, noticeable quality loss
- **Q5_0/Q5_1**: Good balance for constrained devices
- **Q8_0**: Minimal quality loss, 50% memory savings
- **F16**: Best quality, baseline memory usage

For chat applications, Q4_1 or Q5_1 are often acceptable. For code generation or complex reasoning, prefer Q8_0 or higher.

## Best Practices

1. **Start with auto-selection** - Let `select_for_budget()` choose
2. **Test quality** - Benchmark perplexity on your use case
3. **Leave headroom** - KV cache grows with context length
4. **Consider batching** - Multiple concurrent requests need more memory
5. **Profile in production** - Memory usage varies with input length
