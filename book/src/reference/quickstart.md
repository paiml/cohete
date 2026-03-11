# Quick Start

## Install

```bash
cargo install --git https://github.com/paiml/cohete
```

## Pull a Model

Cohete needs at least one LLM model. Download a GGUF model (~1 GB):

```bash
apr pull hf://Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
```

Optionally, create an APR format copy to verify both formats:

```bash
apr import ~/.cache/pacha/models/*.gguf \
    -o ~/.cache/pacha/models/qwen-1.5b-q4k.apr \
    --preserve-q4k
```

## Run

```bash
cohete verify --stdout --allow-missing
```

This runs all 5 tiers and prints JSON results to stdout.

## What Happens

1. **Model Resolution** — cohete auto-discovers models from `~/.cache/pacha/models/`
2. **Tier 1: Smoke** — checks all 8 binaries exist and respond to `--version`
3. **Tier 2: Hardware** — probes GPU, CUDA, memory, disk
4. **Tier 3: Functional** — runs inference on each format x backend combination
5. **Tier 4: Integration** — starts chat server, runs correctness + load tests
6. **Tier 5: Performance** — benchmarks tok/s, checks for regressions

## Output

Each tier produces a JSON artifact. With `--stdout`, they print to terminal.
Without it, they write to `artifacts/latest/`.

## Common Options

```bash
# Write artifacts to a directory
cohete verify --output artifacts/latest

# Only run tiers 1-3
cohete verify --max-tier 3

# Specify a model explicitly
cohete verify --model /path/to/model.gguf

# Continue even if some binaries are missing
cohete verify --allow-missing
```
