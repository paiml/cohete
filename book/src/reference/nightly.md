# Nightly Workflow

Cohete runs nightly as part of the sovereign AI stack's continuous verification
pipeline.

## Schedule

```
04:00 UTC — 20 repos cross-compile aarch64 nightly binaries
05:00 UTC — forjar provisions Jetson, installs binaries + models
06:00 UTC — cohete verifies everything works → artifacts committed
```

## Pipeline

The nightly E2E workflow (`.github/workflows/nightly-e2e.yml`) runs on the
Jetson itself as a self-hosted GitHub Actions runner:

1. **Checkout** — clone cohete repo
2. **Build** — `cargo build --release`
3. **Ensure model** — pull GGUF if missing, create APR copy if missing
4. **Verify** — `cohete verify --output artifacts/latest --allow-missing`
5. **Snapshot** — copy results to `artifacts/history/YYYY-MM-DD.json`
6. **Update README** — `python3 scripts/generate-status.py` regenerates the
   nightly status section between `<!-- NIGHTLY:BEGIN -->` / `<!-- NIGHTLY:END -->`
7. **Commit** — push artifacts and updated README with message
   `nightly: YYYY-MM-DD [pass/FAIL]`

## Cross-Platform Builds

A separate workflow (`.github/workflows/nightly.yml`) runs at 04:00 UTC and
cross-compiles cohete for 5 targets:

| Target | Runner |
|--------|--------|
| `x86_64-unknown-linux-gnu` | Ubuntu (native) |
| `aarch64-unknown-linux-gnu` | Ubuntu (cross) |
| `x86_64-apple-darwin` | macOS (native) |
| `aarch64-apple-darwin` | macOS (native) |
| `x86_64-pc-windows-msvc` | Windows (native) |

Binaries are packaged as `.tar.gz` (unix) or `.zip` (windows) with SHA256
checksums and published as a nightly prerelease on GitHub.

## Jetson Runner

The self-hosted runner is a Jetson Orin Nano (8GB, 15W, CUDA 12.6).

**Constraints:**
- Shared CPU/GPU memory (8GB total, ~5.9GB available)
- GPU JIT kernel compilation consumes ~1.5GB VRAM
- Full 5-tier verification takes ~12 minutes
- Job timeout: 25 minutes

**Known behaviors:**
- GPU inference may fail (Warn) due to VRAM exhaustion — CPU inference proves correctness
- `apr pull` requires `--skip-contract` to bypass PMAT-237 tensor naming QA
- Back-to-back runs need 2s GPU cooldown between server kills

See [Troubleshooting](./troubleshooting.md) for common issues.

## History & Regression

Daily snapshots in `artifacts/history/` enable tier 5 regression detection.
The performance tier compares current `tok/s` against a rolling baseline
derived from historical runs.

## Status Generation

`scripts/generate-status.py` reads artifact JSON files and generates:

- Tier results table (pass/fail/skip counts)
- Binary versions table
- Format × backend matrix with latencies
- Correctness test counts
- UAT results (19 scenarios across U1-U4 suites)
- Performance metrics
- Hardware summary

This content is injected into `README.md` between the nightly markers,
keeping the README always current without manual updates.
