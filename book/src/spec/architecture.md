# Cohete: Nightly Edge E2E Verification

**Status:** Draft
**Owner:** cohete repo
**Scope:** Prove the sovereign AI stack works on Jetson edge hardware every night

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Architecture](#2-architecture)
3. [Ownership Model](#3-ownership-model)
4. [Model & Modalities](#4-model--modalities)
5. [Binary Matrix](#5-binary-matrix)
6. [Test Tiers](#6-test-tiers)
7. [Nightly CI Workflow](#7-nightly-ci-workflow)
8. [Falsification Artifacts](#8-falsification-artifacts)
9. [Success Criteria](#9-success-criteria)

Sub-specifications (in `components/`):
- [test-tiers.md](test-tiers.md) — Detailed 5-tier test framework
- [binary-matrix.md](binary-matrix.md) — Binary inventory, features, versions
- [artifacts.md](artifacts.md) — JSON schema, storage, regression detection

---

## 1. Problem Statement

We ship nightly binaries from 20 sovereign stack repos. We have a Jetson Orin Nano 8GB
with a self-hosted GitHub Actions runner. Nothing connects these two facts.

**Today's gaps:**

| Gap | Impact |
|-----|--------|
| No nightly verification that binaries run on aarch64 | Ship broken ARM builds silently |
| No proof inference works on Jetson GPU | Vulkan/CUDA regressions go undetected |
| No proof transcription works on ARM NEON | whisper.apr regressions go undetected |
| No integration test across tools | copia + trueno-rag + whisper.apr never tested together |
| No chat server verification | Can't prove model serves requests on edge |
| Canary pipeline lives in 3 repos | Nobody owns it end-to-end |

**Cohete fills this gap.** One repo, one binary, one nightly CI job. It runs on the
Jetson itself and produces falsifiable JSON artifacts proving the stack works.

## 2. Architecture

```
20 repos ──nightly──▶ GitHub Releases (aarch64 binaries)
                              │
                              ▼
                     forjar nightly binary
                     (provisions jetson,
                      installs all binaries,
                      syncs model + test data)
                              │
                              ▼
                    ┌─────────────────────┐
                    │   JETSON ORIN NANO  │
                    │   (self-hosted GHA) │
                    │                     │
                    │   cohete nightly:   │
                    │   1. verify install │
                    │   2. run 5 tiers    │
                    │   3. emit artifacts │
                    └─────────────────────┘
                              │
                              ▼
                     artifacts/*.json
                     committed to repo
```

**Separation of concerns:**

| Repo | Owns | Does NOT own |
|------|------|--------------|
| 20 stack repos | Build + release nightly aarch64 binaries | Jetson deployment |
| forjar | Provision jetson, install binaries, sync models | Testing them |
| **cohete** | **Verify everything works on jetson** | Building or deploying binaries |

Cohete assumes binaries are already installed at `~/.cargo/bin/` by forjar.
It also checks PATH as a fallback (for dev machines). It only tests.
If a test fails, the bug is upstream — cohete reports it.

**CLI:**

```
cohete verify [OPTIONS]
  -o, --output <DIR>       Write artifacts here [default: artifacts/latest]
      --max-tier <N>       Only run tiers 1..N (1-5)
      --stdout             Print JSON to stdout instead of writing files
      --allow-missing      Continue past tier 1 gate if binaries are missing
      --model <PATH>       Path to model file (overrides COHETE_MODEL and auto-discovery)
```

## 3. Ownership Model

**Cohete owns the Jetson for E2E verification.** This means:

- The jetson self-hosted runner runs cohete's nightly workflow
- cohete's `forjar.yaml` may add verification-specific resources (test data, models)
- cohete does NOT provision base system — forjar owns that
- cohete does NOT cross-compile — each repo owns its own nightly release
- cohete DOES own the test suite, artifact schema, and pass/fail criteria

**Dependency chain:**

```
repo nightlies build (04:00 UTC)
    └─▶ forjar provisions jetson (05:00 UTC)
        └─▶ cohete verifies (06:00 UTC)
```

Staggered schedule ensures binaries exist before forjar installs them,
and forjar finishes before cohete tests.

## 4. Model & Modalities

### Model

**Qwen2.5-Coder-1.5B-Instruct** quantized to **Q4_K** (~1 GB).

| Property | Value |
|----------|-------|
| Base model | Qwen/Qwen2.5-Coder-1.5B-Instruct |
| Quantization | Q4_K (4-bit, best quality/size for Jetson) |
| Disk size | ~1 GB |
| Runtime memory | ~1.5 GB (model + KV cache) |
| Headroom | ~6 GB free on 7.4 GB unified memory |
| Format | `.apr` (aprender native) |
| Path | `~/data/models/canary/qwen-1.5b-q4k.apr` |
| Provisioned by | forjar (imported via `apr import` + `apr quantize` on intel) |

**Why this model:** Coder variant matches the sovereign stack's purpose (code tools).
1.5B params is the largest that fits comfortably in 8 GB unified memory with room
for concurrent whisper + RAG workloads.

### Whisper Model

**whisper-tiny.en** (~75 MB) for speech-to-text on ARM NEON.

| Property | Value |
|----------|-------|
| Model | OpenAI whisper-tiny.en |
| Size | ~75 MB |
| Runtime | CPU-only (ARM NEON SIMD) |
| Path | `~/data/models/canary/whisper-tiny.en` |
| Provisioned by | forjar |

### Modalities

Cohete proves **6 modalities** of the sovereign stack on edge:

| # | Modality | Binary | What it proves | Artifact |
|---|----------|--------|----------------|----------|
| M1 | **CLI inference** | `apr run` | Model loads, tokenizes, generates on aarch64 | `functional.json` |
| M2 | **Chat server** | `apr serve` | HTTP server starts, responds to requests | `integration.json` |
| M3 | **Correctness** | `cohete` + `apr` | Model produces correct code/math/SQL (6 tests) | `integration.json` |
| M4 | **Load test** | `cohete` + `apr` | Handles concurrent requests without OOM/crash | `integration.json` |
| M5 | **Transcription** | `whisper-apr` | Audio → text on ARM NEON (no GPU required) | `functional.json` |
| M6 | **RAG pipeline** | `whisper-apr` + `trueno-rag` | Transcribe → index → query end-to-end | `integration.json` |

**M1: CLI Inference** — Single-shot generation. Proves the binary loads the model
and produces tokens. Fastest feedback loop.

```bash
apr run ~/data/models/canary/qwen-1.5b-q4k.apr \
    --prompt "What is 7 * 8?" --max-tokens 16
```

**M2: Chat Server** — Start `apr serve run <model> --port 8090`, verify `/health` endpoint,
send a chat completion request via `runner::curl_post()`. Proves the HTTP stack works on ARM.

**M3: Correctness** — 6 deterministic tests (temperature=0) via cohete's built-in
curl requests against the running chat server:

| Test | Prompt | Pass criteria |
|------|--------|--------------|
| basic_math | "What is 7 * 8?" | Contains "56" |
| python_fibonacci | "Write fibonacci" | Contains "def fib" |
| rust_hello | "Write hello world in Rust" | Contains "fn main" |
| json_output | "Return JSON with name Alice" | Regex `"name".*"Alice"` |
| code_explanation | "What does vec map do?" | Regex `(double\|multiply\|2)` |
| sql_query | "Top 5 users by orders" | Regex `SELECT.*ORDER BY.*LIMIT` |

**M4: Load Test** — 2 sequential `runner::curl_post()` requests. Jetson has 8 GB shared
memory; this proves inference doesn't OOM under light load.

**M5: Transcription** — `whisper-apr transcribe test.wav` on a short audio clip.
Proves ARM NEON SIMD path works. No GPU needed — whisper runs CPU-only.

**M6: RAG Pipeline** — Full chain: transcribe audio → index transcript → query
the index. Proves whisper.apr + trueno-rag interoperate on ARM.

### Compiled Binary (Future — M7)

`apr compile` bakes model weights into a single self-contained aarch64 binary
(~1 GB, llamafile-style). This is the ultimate falsification artifact. Blocked
on aprender's compile feature being cross-compile-ready.

## 5. Binary Matrix

Cohete tests a subset of the 20 nightly repos — only those that matter on edge.

| Binary | Repo | Modalities | Tier 1 | Tier 2 | Tier 3 | Tier 4 | Tier 5 |
|--------|------|-----------|--------|--------|--------|--------|--------|
| `apr` | aprender | M1,M2,M3,M4 | --version | GPU probe | inference | serve+correctness+load | tok/s |
| `whisper-apr` | whisper.apr | M5,M6 | --version | NEON detect | transcribe | pipeline | — |
| `trueno-rag` | trueno-rag | M6 | --version | — | index+query | pipeline | latency |
| `forjar` | forjar | — | --version | — | plan (smoke) | — | — |
| `pmat` | pmat | — | --version | — | query (smoke) | — | — |
| `copia` | copia | — | --version | — | sync (local) | — | — |
| `pzsh` | pzsh | — | --version | — | eval (smoke) | — | — |
| `batuta` | batuta | — | --version | — | oracle (smoke) | — | — |

See [binary-matrix.md](binary-matrix.md) for full details.

## 6. Test Tiers

Five tiers, increasing integration depth. Total budget: **< 5 minutes**.

| Tier | Name | What | Budget |
|------|------|------|--------|
| 1 | Smoke | `--version` and `--help` for all 8 binaries | 10s |
| 2 | Hardware | GPU enumeration, CUDA, Vulkan, NEON | 15s |
| 3 | Functional | Each binary does its core job (M1, M5) | 120s |
| 4 | Integration | Chat server + correctness + UAT (19) + load + RAG (M2-M4, M6) | 120s |
| 5 | Performance | Baselines: tok/s, whisper RTF, RAG latency, memory | 30s |

**Tier 1 is the gate.** If any binary fails `--version`, the entire run is red.
Higher tiers can have individual failures without blocking the overall run.

See [test-tiers.md](test-tiers.md) for test definitions.

## 7. Nightly CI Workflow

```yaml
# .github/workflows/nightly-e2e.yml
name: Nightly E2E
on:
  schedule:
    - cron: "0 6 * * *"    # 06:00 UTC (after forjar at 05:00)
  workflow_dispatch:

runs-on: [self-hosted, jetson]

steps:
  1. checkout cohete
  2. cargo build --release  # build cohete binary (on jetson, it's small)
  3. cohete verify           # run all 5 tiers + 6 modalities
  4. commit artifacts/       # push results back to repo
```

**Self-hosted runner requirements:**
- Rust toolchain (installed by forjar)
- All sovereign stack binaries at `~/.cargo/bin/` (installed by forjar)
- Qwen model at `~/data/models/canary/qwen-1.5b-q4k.apr` (synced by forjar)
- Whisper model at `~/data/models/canary/whisper-tiny.en` (synced by forjar)
- Test audio at `~/data/models/canary/test-2s.wav` (deployed by forjar)
- Network access to GitHub (for checkout + push)

**Manual dispatch:** Any developer can trigger via `workflow_dispatch` to validate
after an ad-hoc forjar provision.

## 8. Falsification Artifacts

Each nightly run produces JSON artifacts in `artifacts/`:

```
artifacts/
├── latest/
│   ├── smoke.json          # tier 1: binary versions
│   ├── hardware.json       # tier 2: GPU, CUDA, NEON
│   ├── functional.json     # tier 3: per-binary test results (M1, M5)
│   ├── integration.json    # tier 4: server + pipeline results (M2-M4, M6)
│   ├── performance.json    # tier 5: baselines
│   └── summary.json        # overall pass/fail + metadata
└── history/
    └── 2026-03-10.json     # daily snapshots for regression tracking
```

**`summary.json` schema:**

```json
{
  "schema_version": 1,
  "date": "2026-03-10",
  "timestamp": "2026-03-10T06:04:32Z",
  "cohete_version": "0.1.0",
  "pass": true,
  "duration_s": 187,
  "tiers": {
    "smoke": { "pass": true, "total": 8, "passed": 8, "failed": 0, "skipped": 0 },
    "hardware": { "pass": true, "total": 1, "passed": 1, "failed": 0, "skipped": 0 },
    "functional": { "pass": true, "total": 13, "passed": 10, "failed": 0, "skipped": 2 },
    "integration": { "pass": true, "total": 23, "passed": 22, "failed": 0, "skipped": 1 },
    "performance": { "pass": true, "regressions": 0 }
  },
  "binaries": [
    { "name": "apr", "version": "0.28.0", "installed": true },
    { "name": "whisper-apr", "version": "0.2.5", "installed": true },
    { "name": "trueno-rag", "version": "0.2.3", "installed": true },
    { "name": "forjar", "version": "0.5.2", "installed": true },
    { "name": "pmat", "version": "0.9.1", "installed": true },
    { "name": "copia", "version": "0.3.0", "installed": true },
    { "name": "pzsh", "version": "0.2.4", "installed": true },
    { "name": "batuta", "version": "0.6.7", "installed": true }
  ],
  "version_changes": [
    { "binary": "forjar", "from": "0.5.1", "to": "0.5.2" }
  ],
  "hardware": {
    "gpu": "Orin",
    "cuda": "12.6",
    "neon": true,
    "power_mode": "MAXN_SUPER",
    "jetpack": "R36.4.3"
  }
}
```

See [artifacts.md](artifacts.md) for full schemas.

## 9. Success Criteria

**Cohete is done when:**

1. Nightly CI runs on jetson self-hosted runner at 06:00 UTC
2. All 8 binaries pass tier 1 (smoke)
3. All 6 modalities are tested (M1–M6)
4. Correctness tests (M3) pass for all 6 prompts
5. UAT (19 scenarios across U1-U4) validates real-world problem solving
6. whisper-apr transcription (M5) produces output on ARM NEON
7. RAG pipeline (M6) completes transcribe → index → query
8. `summary.json` is committed to `artifacts/` after each run
9. README contains modality matrix with links to nightly binaries

**Cohete is NOT responsible for:**
- Building binaries (repos own this)
- Installing binaries (forjar owns this)
- Fixing upstream bugs (repos own this)
- Provisioning jetson base system (forjar owns this)

**Falsification test:** If cohete reports green, any developer can inspect
`artifacts/latest/summary.json` and verify each claim independently by
SSHing to jetson and running the same commands.
