# Component Spec: Test Tiers

**Parent:** [cohete-e2e-demo.md](architecture.md)
**Scope:** Define the 5-tier test framework that cohete runs on jetson nightly

---

## Tier 1: Smoke (Gate)

**Budget:** 10 seconds
**Policy:** ALL must pass. Any failure = entire run is RED.

For every binary in the matrix, run:

```bash
<binary> --version    # must exit 0, print version string
<binary> --help       # must exit 0, print usage
```

**Output: `smoke.json`**

```json
{
  "tier": 1,
  "pass": true,
  "binaries": [
    {
      "name": "forjar",
      "path": "/home/noah/.cargo/bin/forjar",
      "exists": true,
      "executable": true,
      "version": "forjar 0.5.2",
      "help_ok": true
    }
  ]
}
```

**Failure modes:**
- Binary not found at expected path → forjar provision failed
- Binary exists but segfaults → cross-compile produced bad aarch64 binary
- Binary runs but wrong version → stale install, forjar didn't update

---

## Tier 2: Hardware

**Budget:** 15 seconds
**Policy:** Failures logged, do not gate. Hardware state is informational.

| Check | Command | Pass criteria |
|-------|---------|---------------|
| GPU present | `nvidia-smi --query-gpu=name` | Exit 0, lists GPU model |
| CUDA available | `nvidia-smi \| grep 'CUDA Version'` | Parses version string |
| Vulkan available | `vulkaninfo --summary` | Contains deviceName |
| NEON available | `grep neon /proc/cpuinfo` | Match found |
| CPU cores | `nproc` | Returns count |
| Memory | `grep MemTotal /proc/meminfo` | Total >= 7000 MB |
| Disk space | `df -BG /home` | Available >= 10 GB |
| Power mode | `nvpmodel -q` | Extracts NV Power Mode |
| JetPack version | `cat /etc/nv_tegra_release` | Non-empty |

**Output: `hardware.json`**

```json
{
  "tier": 2,
  "pass": true,
  "gpu": {
    "model": "Orin",
    "cuda_version": "12.6"
  },
  "vulkan": "deviceName = NVIDIA Tegra Orin",
  "cpu": {
    "neon": true,
    "cores": 6
  },
  "memory": {
    "total_mb": 7628,
    "available_mb": 5200
  },
  "disk_ok": true,
  "power_mode": "MAXN_SUPER",
  "jetpack": "6.2.2"
}
```

---

## Tier 3: Functional (M1, M5)

**Budget:** 120 seconds
**Policy:** Per-binary pass/fail. Failures do not gate other binaries.

### M1: CLI Inference — `apr`

```bash
# Hardware self-test (requires model file)
apr check ~/data/models/canary/qwen-1.5b-q4k.apr

# Single-shot inference (requires model)
apr run --model ~/data/models/canary/qwen-1.5b-q4k.apr \
    --prompt "1+1=" --max-tokens 8 2>&1
```

**Pass:** `apr check` exits 0. Inference produces tokens.
**Skip:** both tests skipped if model not present.

### M5: Transcription — `whisper-apr`

```bash
# Transcribe short audio on ARM NEON
whisper-apr transcribe ~/data/models/canary/test-2s.wav \
    --model ~/data/models/canary/whisper-tiny.en 2>&1
```

**Pass:** exits 0, produces text output.
**Skip:** if model or audio fixture not present.

### Infrastructure Tools

#### forjar

```bash
forjar plan -f /dev/null 2>&1   # exits with error but proves parser works
```

**Pass:** exits without segfault, prints meaningful error.

#### pmat

```bash
cd /home/noah/src/cohete && pmat query --literal "fn main" --limit 5
```

**Pass:** exits 0, returns results.

#### copia

```bash
mkdir -p /tmp/cohete-test-src /tmp/cohete-test-dst
echo "test" > /tmp/cohete-test-src/file.txt
copia sync -r /tmp/cohete-test-src/ /tmp/cohete-test-dst/
diff /tmp/cohete-test-src/file.txt /tmp/cohete-test-dst/file.txt
```

**Pass:** files match.

#### pzsh

```bash
pzsh status
```

**Pass:** exits 0, prints status and startup time.

#### trueno-rag

```bash
echo '{"text": "Rust is a systems language"}' > /tmp/cohete-test.jsonl
trueno-rag index --sqlite /tmp/cohete-test.db /tmp/cohete-test.jsonl
trueno-rag query --sqlite /tmp/cohete-test.db "systems programming"
```

**Pass:** exits 0, returns search results.

#### batuta

```bash
batuta oracle --rag "test query" 2>&1
```

**Pass:** exits 0, returns results (or "no results" gracefully).
**Skip:** if no oracle index configured.

**Output: `functional.json`**

```json
{
  "tier": 3,
  "pass": true,
  "total": 9,
  "passed": 8,
  "skipped": 1,
  "failed": 0,
  "tests": [
    { "binary": "apr", "test": "check", "modality": null, "status": "pass", "duration_ms": 1200 },
    { "binary": "apr", "test": "inference", "modality": "M1", "status": "pass", "duration_ms": 3400 },
    { "binary": "whisper-apr", "test": "transcribe", "modality": "M5", "status": "pass", "duration_ms": 2100 }
  ]
}
```

---

## Tier 4: Integration (M2, M3, M4, M6)

**Budget:** 120 seconds
**Policy:** Failures logged. These test cross-binary pipelines.

### M2 + M3: Chat Server + Correctness

```bash
# 1. Start chat server (background)
apr serve --model ~/data/models/canary/qwen-1.5b-q4k.apr --port 8090 &
SERVER_PID=$!

# 2. Wait for health
for i in $(seq 1 30); do
  curl -sf http://localhost:8090/health && break
  sleep 1
done

# 3. Run 6 correctness tests (cohete built-in curl, temperature=0)
# Tests: basic_math, python_fibonacci, rust_hello, json_output,
#        code_explanation, sql_query
# Each sends a POST to /v1/chat/completions with max_tokens=128
curl -sf -X POST http://localhost:8090/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"messages":[{"role":"user","content":"What is 7 * 8?"}],"max_tokens":128,"temperature":0}'
# Pass: response contains "56"
# (repeat for all 6 tests)

# 4. Stop server (after M4)
```

**Pass:** All 6 tests pass (basic_math, python_fibonacci, rust_hello,
json_output, code_explanation, sql_query).

### M4: Load Test

```bash
# Server still running from M2
# 2 concurrent curl requests
for i in 1 2; do
  curl -sf -X POST http://localhost:8090/v1/chat/completions \
    -H 'Content-Type: application/json' \
    -d '{"messages":[{"role":"user","content":"Hello"}],"max_tokens":16,"temperature":0}' &
done
wait

# Now stop server
kill $SERVER_PID 2>/dev/null
pkill -f "apr serve" 2>/dev/null || true
```

**Pass:** No crashes, no OOM. Both requests complete successfully.

### M6: RAG Pipeline (Transcribe → Index → Query)

```bash
# 1. Transcribe audio to text
whisper-apr transcribe ~/data/models/canary/test-2s.wav \
    --model ~/data/models/canary/whisper-tiny.en \
    -o /tmp/transcript.txt

# 2. Index the transcript
echo "{\"text\": \"$(cat /tmp/transcript.txt)\"}" > /tmp/transcript.jsonl
trueno-rag index --sqlite /tmp/pipeline.db /tmp/transcript.jsonl

# 3. Query the index
trueno-rag query --sqlite /tmp/pipeline.db "what was said"
```

**Pass:** query returns results from the transcribed content.
**Skip:** if whisper model or audio not present.

**Output: `integration.json`**

```json
{
  "tier": 4,
  "pass": true,
  "total": 4,
  "passed": 4,
  "failed": 0,
  "skipped": 0,
  "modalities": {
    "M2_chat_server": { "pass": true, "detail": "health endpoint OK" },
    "M3_correctness": { "pass": true, "total": 6, "passed": 6 },
    "M4_load_test": { "pass": true, "detail": "2 concurrent requests, 1240ms" },
    "M6_rag_pipeline": { "pass": true, "detail": "transcribe→index→query complete" }
  }
}
```

---

## Tier 5: Performance

**Budget:** 30 seconds
**Policy:** No gate. Baselines recorded for regression tracking.

| Metric | Command | Unit | Modality |
|--------|---------|------|----------|
| Inference tok/s | `apr bench --model ... --tokens 32` | tokens/sec | M1 |
| Whisper RTF | `whisper-apr bench --model ... --input test.wav` | real-time factor | M5 |
| RAG query latency | `trueno-rag query --sqlite ... "test" --timing` | ms | M6 |
| Memory watermark | `cat /proc/meminfo \| grep MemAvailable` | MB | — |

**Regression detection:** Compare against `artifacts/history/` — if a metric
deviates more than 20% from 7-day rolling average, flag as regression warning.

**Output: `performance.json`**

```json
{
  "tier": 5,
  "pass": true,
  "regressions": 0,
  "metrics": {
    "inference_tok_s": 12.4,
    "whisper_rtf": 0.3,
    "rag_query_ms": 28,
    "memory_available_mb": 5100
  },
  "baselines": {
    "inference_tok_s_7d_avg": 11.8,
    "whisper_rtf_7d_avg": 0.32,
    "rag_query_ms_7d_avg": 31
  }
}
```

---

## Execution Order

```
tier 1 (gate)
    │ if fail → abort, emit summary.json with pass=false
    ▼
tier 2 (hardware)
    │ always continues
    ▼
tier 3 (functional) — M1: CLI inference, M5: transcription
    │ always continues
    ▼
tier 4 (integration) — M2: server, M3: correctness, M4: load, M6: RAG pipeline
    │ always continues
    ▼
tier 5 (performance) — baselines for M1, M5, M6
    │
    ▼
emit summary.json
```

Tier 1 is the only hard gate. All other tiers run regardless of individual
test failures, so we always get a complete picture of the stack's health.
