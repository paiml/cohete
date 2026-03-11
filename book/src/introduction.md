# Cohete

**Cohete** (Spanish: "rocket") is a nightly E2E verification tool that proves
the sovereign AI stack works on NVIDIA Jetson edge hardware. One binary, five
tiers, falsifiable JSON artifacts.

```
20 repos build nightly  →  forjar provisions  →  cohete verifies  →  artifacts prove it
```

## What It Does

Cohete runs on the Jetson itself (as a self-hosted GitHub Actions runner) and
executes 5 tiers of tests against pre-installed binaries:

| Tier | Name | What It Proves |
|------|------|----------------|
| 1 | **Smoke** | All 8 binaries installed and respond to `--version` |
| 2 | **Hardware** | GPU, CUDA, Vulkan, NEON present and functional |
| 3 | **Functional** | Inference works across GGUF/APR formats on GPU/CPU |
| 4 | **Integration** | Chat server, correctness, load test, RAG pipeline |
| 5 | **Performance** | tok/s baseline, regression detection |

Total runtime: **< 5 minutes**.

## What It Proves

Six modalities of the sovereign AI stack on edge:

| # | Modality | What It Proves |
|---|----------|----------------|
| M1 | CLI Inference | GGUF + APR formats produce correct output on GPU and CPU |
| M2 | Chat Server | OpenAI-compatible `/v1/chat/completions` API works |
| M3 | Correctness | 6 deterministic tests (math, code, SQL, JSON) pass |
| M4 | Load Test | Concurrent requests without OOM |
| M5 | Transcription | Audio to text on ARM NEON |
| M6 | RAG Pipeline | Transcribe, index, query end-to-end |

## What It Does NOT Do

- Build binaries (each repo owns its nightly release)
- Install binaries (forjar owns provisioning)
- Fix upstream bugs (cohete only reports failures)
- Provision the Jetson base system (forjar owns that)

## Format x Backend Matrix

Cohete proves inference works across all combinations:

|        | GPU (CUDA) | CPU |
|--------|-----------|-----|
| **GGUF** | tested | tested |
| **APR**  | tested | tested |

GPU tests are skipped on CPU-only machines. APR tests require a `.apr` model.
