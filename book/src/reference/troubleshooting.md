# Troubleshooting

## UAT Failures

### All U1/U2/U4 fail with 3-5ms duration

**Symptom:** Most UAT scenarios fail instantly (3-5ms) while U3-003
(format parity, which uses CLI `apr run`) passes.

**Cause:** The `apr serve` instance is unhealthy or overloaded. The
`curl -sf` flag causes curl to fail silently on HTTP errors. A 3-5ms
duration means curl connected but got an error response.

**Fix:**
1. Check for stale `apr serve` processes: `pgrep -af 'apr serve'`
2. Kill them: `pkill -f 'apr serve'`
3. Wait 2-3 seconds for GPU memory to free
4. Re-run: `cohete verify --max-tier 4 --stdout --allow-missing`

**Root cause:** Cohete kills the server after each tier 4 run, but GPU
memory release is asynchronous. Back-to-back runs can exhaust VRAM.

### U3-001 reflexivity fails (identical=false)

**Symptom:** Three identical requests produce different outputs despite
`temperature: 0`.

**Cause:** A stale server from a previous run is serving cached KV state
from different prompts. The "same prompt" hits different cache states.

**Fix:** Kill stale servers and re-run.

### U3-003 format parity fails

**Symptom:** GGUF and APR formats produce different outputs for `2+2=`.

**Expected:** Both should contain "4", but exact output may differ.
The check passes if either (a) outputs are byte-identical, or (b) both
contain "4". Only fails if one format doesn't produce "4" at all.

**Cause:** Model conversion issue, or one format is a different
quantization level.

### U4-002/U4-004 intermittent failures

**Symptom:** Refinement or error correction tests fail occasionally.

**Cause:** LLM output varies slightly even at `temperature: 0` due to
floating-point non-determinism in GPU kernels (especially with FP8/Q4_K).
The check heuristics are designed to be broad, but edge cases exist.

**What the checks accept:**
- U4-002 (refinement): `arg`, `clap`, `name`, `--`, `env`, `std::`, or `fn main`
- U4-004 (error correction): contains `2` or `two`, and doesn't affirm `3`

## Server Issues

### M2: server did not become healthy

**Symptom:** Tier 4 skips all tests because `apr serve run` didn't
respond to `/health` within 30 seconds.

**Causes:**
- Model file not found (check `--model` flag or `COHETE_MODEL` env)
- GPU memory exhausted from a previous run
- Port 8090 already in use: `lsof -i :8090`

### M4: load test fails

**Symptom:** One of the two sequential requests fails.

**Cause:** Server ran out of memory or crashed during U1 (which sends
5 prompts with `max_tokens: 256`). Check `dmesg` for OOM kills.

## Model Issues

### No model found

**Symptom:** Tiers 3-5 are skipped with "no model found."

**Resolution order:**
1. `--model /path/to/model.gguf` (CLI flag)
2. `COHETE_MODEL=/path/to/model.gguf` (env var)
3. Auto-discovery: newest `.gguf` and `.apr` in `~/.cache/pacha/models/`

**Quick fix:**
```bash
apr pull hf://Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
```

## Artifact Issues

### generate-status.py: markers not found

**Symptom:** `python3 scripts/generate-status.py` says "markers not found."

**Fix:** Ensure `README.md` contains:
```markdown
<!-- NIGHTLY:BEGIN -->
<!-- NIGHTLY:END -->
```

### Stale artifacts from previous run

**Symptom:** `artifacts/latest/` contains results from a previous run.

**Fix:** Delete and re-run:
```bash
rm -rf artifacts/latest
cohete verify --output artifacts/latest --allow-missing
```
