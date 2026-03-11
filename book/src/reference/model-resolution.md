# Model Resolution

Cohete needs at least one LLM model for tiers 3–5. Models are resolved
in priority order:

## Priority Chain

1. **`--model <path>`** — CLI flag, highest priority
2. **`COHETE_MODEL` env var** — path to a model file
3. **Auto-discovery** — scans `~/.cache/pacha/models/` for `.gguf` and `.apr` files
4. **Legacy paths** — hardcoded fallback paths (Jetson-specific)

If no model is found at any level, tiers 3–5 are skipped.

## Auto-Discovery

When no explicit model is provided, cohete scans `~/.cache/pacha/models/`
and selects the **newest file** (by modification time) of each format:

- Newest `.gguf` file → used for GGUF backend tests
- Newest `.apr` file → used for APR backend tests

Both formats are tested independently in the format × backend matrix.

## Format × Backend Matrix

Cohete proves inference works across all combinations:

|        | GPU (CUDA) | CPU |
|--------|-----------|-----|
| **GGUF** | tested | tested |
| **APR**  | tested | tested |

- GPU tests are skipped on CPU-only machines
- APR tests require a `.apr` model file
- GGUF tests require a `.gguf` model file

## Pulling Models

```bash
# Download a GGUF model (~1 GB)
apr pull hf://Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf

# Create an APR format copy
apr import ~/.cache/pacha/models/*.gguf \
    -o ~/.cache/pacha/models/qwen-1.5b-q4k.apr \
    --preserve-q4k
```

Models are cached in `~/.cache/pacha/models/` and reused across runs.
