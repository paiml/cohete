# Cohete: Nightly Edge E2E Verification

**Cohete** (Spanish: "rocket") proves the sovereign AI stack works on NVIDIA Jetson
edge hardware every night. One repo, one binary, one CI job.

## Quick Start

```bash
# 1. Install cohete
cargo install --git https://github.com/paiml/cohete

# 2. Pull a model (one-time ~1 GB download, cached in ~/.cache/pacha/models/)
apr pull hf://Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf

# 3. (Optional) Create an .apr format copy to verify both formats
apr import ~/.cache/pacha/models/*.gguf -o ~/.cache/pacha/models/qwen-1.5b-q4k.apr --preserve-q4k

# 4. Run — auto-discovers models from cache
cohete verify --stdout --allow-missing
```

Cohete auto-discovers models from `~/.cache/pacha/models/`. You can also specify
a model explicitly: `cohete verify --model /path/to/model.gguf` or via the
`COHETE_MODEL` environment variable.

## How It Works

```
20 repos build nightly aarch64 binaries
    → forjar installs them on Jetson
        → cohete verifies they work
            → artifacts prove it
```

Cohete runs on **any machine with `apr` installed** — dev workstations (x86_64),
CI runners, and the Jetson itself (self-hosted GitHub Actions runner).
It does not build or deploy — it only tests.

## Hardware

| Property | Value |
|----------|-------|
| Device | Jetson Orin Nano 8GB |
| CPU | Cortex-A78AE, 6 cores, ARM NEON |
| GPU | 1024 CUDA cores, compute 8.7 |
| Memory | 7.4 GB unified |
| JetPack | 6.2.2 (CUDA 12.6) |

## Model

| Property | Value |
|----------|-------|
| LLM | [Qwen2.5-Coder-1.5B-Instruct](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct), Q4_K (~1 GB) |
| Whisper | [whisper-tiny.en](https://huggingface.co/openai/whisper-tiny.en) (~75 MB) |

Both `.gguf` (community standard) and `.apr` (native format with embedded tokenizer
and Q4K fused kernels) are tested independently on GPU and CPU.

## Modality Matrix

| # | Modality | Binary | Nightly Binary | What It Proves |
|---|----------|--------|----------------|----------------|
| M1 | CLI Inference | `apr run` | [aprender nightly](https://github.com/paiml/aprender/releases/tag/nightly) | GGUF + APR formats produce correct output on GPU and CPU |
| M2 | Chat Server | `apr serve run` | [aprender nightly](https://github.com/paiml/aprender/releases/tag/nightly) | HTTP server responds with OpenAI-compatible API |
| M3 | Correctness | `cohete` + `apr` | [aprender nightly](https://github.com/paiml/aprender/releases/tag/nightly) | Correct code/math/SQL generation (6 tests, temperature=0) |
| M4 | Load Test | `cohete` + `apr` | [aprender nightly](https://github.com/paiml/aprender/releases/tag/nightly) | Concurrent requests without OOM |
| M5 | Transcription | `whisper-apr` | [whisper.apr nightly](https://github.com/paiml/whisper.apr/releases/tag/nightly) | Audio → text on ARM NEON |
| M6 | RAG Pipeline | `whisper-apr` + `trueno-rag` | [trueno-rag nightly](https://github.com/paiml/trueno-rag/releases/tag/nightly) | Transcribe → index → query end-to-end |

## Binary Matrix

### AI/ML Stack

| Binary | Repo | Nightly | Modalities |
|--------|------|---------|------------|
| `apr` | aprender | [download](https://github.com/paiml/aprender/releases/tag/nightly) | M1, M2, M3, M4 |
| `whisper-apr` | whisper.apr | [download](https://github.com/paiml/whisper.apr/releases/tag/nightly) | M5, M6 |
| `trueno-rag` | trueno-rag | [download](https://github.com/paiml/trueno-rag/releases/tag/nightly) | M6 |

### Core Tools

| Binary | Repo | Nightly | Test |
|--------|------|---------|------|
| `forjar` | forjar | [download](https://github.com/paiml/forjar/releases/tag/nightly) | smoke |
| `pmat` | pmat | [download](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/nightly) | smoke |
| `copia` | copia | [download](https://github.com/paiml/copia/releases/tag/nightly) | smoke |
| `pzsh` | pzsh | [download](https://github.com/paiml/pzsh/releases/tag/nightly) | smoke |
| `batuta` | batuta | [download](https://github.com/paiml/batuta/releases/tag/nightly) | smoke |

## Test Tiers

| Tier | Name | Budget | Gate? |
|------|------|--------|-------|
| 1 | Smoke (`--version` all 8 binaries) | 10s | **YES** |
| 2 | Hardware (GPU, CUDA, Vulkan, NEON) | 15s | no |
| 3 | Functional (M1 inference GPU+CPU, format parity, M5 transcription) | 120s | no |
| 4 | Integration (M2 server, M3 correctness, M4 load, M6 RAG) | 120s | no |
| 5 | Performance (tok/s, whisper RTF, RAG latency) | 30s | no |

Total: **< 5 minutes**.

## Format x Backend Matrix (Tier 3)

Cohete proves inference works across all combinations of model format and compute backend:

| Test | Format | Backend | Pass Criteria |
|------|--------|---------|---------------|
| `inference_gguf_gpu` | GGUF | GPU (CUDA) | "What is 7*8?" → contains "56" |
| `inference_gguf_cpu` | GGUF | CPU | "What is 7*8?" → contains "56" |
| `inference_apr_gpu` | APR | GPU (CUDA) | "What is 7*8?" → contains "56" |
| `inference_apr_cpu` | APR | CPU | "What is 7*8?" → contains "56" |
| `gpu_cpu_parity` | any | both | `apr parity --assert` (informational) |

GPU tests are skipped on CPU-only machines. APR tests require an `.apr` model in cache
(create one with `apr import model.gguf -o model.apr --preserve-q4k`).

## Correctness Tests (M3)

| Test | Prompt | Pass Criteria |
|------|--------|---------------|
| basic_math | "What is 7 * 8?" | Contains "56" |
| python_fibonacci | "Write fibonacci" | Contains "def fib" |
| rust_hello | "Write hello world in Rust" | Contains "fn main" |
| json_output | "Return JSON with name Alice" | Contains "name" AND "Alice" |
| code_explanation | "What does map do on a vector?" | Contains double/multiply/transform/2 |
| sql_query | "Write SQL for top 5 users by orders" | Contains SELECT + ORDER BY + LIMIT |

All tests: temperature=0, deterministic, via `/v1/chat/completions` API.

## Nightly Schedule

```
04:00 UTC — 20 repos build aarch64 nightly binaries
05:00 UTC — forjar provisions Jetson, installs binaries + models
06:00 UTC — cohete verifies everything works
```

## Artifacts

Each nightly produces JSON in `artifacts/`:

```
artifacts/
├── latest/
│   ├── smoke.json         # tier 1: all binary versions
│   ├── hardware.json      # tier 2: GPU/CUDA/NEON state
│   ├── functional.json    # tier 3: inference + transcription
│   ├── integration.json   # tier 4: server + correctness + load + RAG
│   ├── performance.json   # tier 5: baselines + regression detection
│   └── summary.json       # overall pass/fail + metrics
└── history/
    └── YYYY-MM-DD.json    # daily snapshots (for regression tracking)
```

## Ownership

| What | Who |
|------|-----|
| Build nightly binaries | Each repo |
| Install on Jetson | forjar |
| Verify they work | **cohete** |
| Fix bugs | Upstream repo |

## Model Resolution

Cohete discovers models automatically. Priority order:

1. `--model <path>` CLI flag
2. `COHETE_MODEL` environment variable
3. Auto-discovery from `~/.cache/pacha/models/` (newest `.gguf` and `.apr` files)
4. Legacy Jetson path (`/home/noah/data/models/canary/`)

When both `.gguf` and `.apr` files exist in cache, cohete tests both formats
independently. Whisper models use `COHETE_WHISPER_MODEL` and `COHETE_TEST_AUDIO`
environment variables.

## Specification

- [Main spec](docs/specifications/cohete-e2e-demo.md)
- [Test tiers](docs/specifications/components/test-tiers.md)
- [Binary matrix](docs/specifications/components/binary-matrix.md)
- [Artifact schemas](docs/specifications/components/artifacts.md)

## License

MIT
