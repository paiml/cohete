# Component Spec: Falsification Artifacts

**Parent:** [cohete-e2e-demo.md](../cohete-e2e-demo.md)
**Scope:** JSON artifact schemas, storage layout, regression detection

---

## Storage Layout

```
artifacts/
├── latest/                 # symlinks or copies of most recent run
│   ├── smoke.json          # tier 1
│   ├── hardware.json       # tier 2
│   ├── functional.json     # tier 3
│   ├── integration.json    # tier 4
│   ├── performance.json    # tier 5
│   └── summary.json        # overall
└── history/                # daily snapshots (committed to git)
    ├── 2026-03-08.json
    ├── 2026-03-09.json
    └── 2026-03-10.json
```

**`latest/`** is overwritten every run. **`history/`** is append-only — one
file per day, committed to git by the nightly workflow. This creates a
permanent, auditable record.

**Git policy:** Only `history/` is committed. `latest/` is gitignored
(it's a working copy for the current run).

---

## Summary Schema

`summary.json` is the single source of truth for a nightly run.

```json
{
  "schema_version": 1,
  "date": "2026-03-10",
  "timestamp": "2026-03-10T06:04:32Z",
  "cohete_version": "0.1.0",
  "pass": true,
  "duration_s": 187,
  "tiers": {
    "smoke":       { "pass": true, "total": 8, "passed": 8, "failed": 0, "skipped": 0 },
    "hardware":    { "pass": true, "total": 1, "passed": 1, "failed": 0, "skipped": 0 },
    "functional":  { "pass": true, "total": 12, "passed": 10, "failed": 0, "skipped": 2 },
    "integration": { "pass": true, "total": 4, "passed": 4, "failed": 0, "skipped": 0 },
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
    "power_mode": "MAXN_SUPER"
  }
}
```

**Fields:**
- `pass`: true only if tier 1 passes AND no tier 3/4 failures (skips are OK)
- `version_changes`: compared against previous day's history file
- `skipped`: tests that couldn't run due to missing data (not counted as failures)

---

## History File Schema

`history/YYYY-MM-DD.json` is a copy of `summary.json` for that day.
Same schema, same content. One file per day.

**Retention:** Keep 90 days in git. Older files can be pruned via CI.

---

## Regression Detection

Cohete compares today's `performance.json` against the 7-day rolling average
from `history/`.

**Algorithm:**

```
for each metric in [inference_tok_s, rag_query_ms]:
    avg = mean(last 7 days from history/)
    today = current run value
    delta = (today - avg) / avg * 100

    if metric is "higher is better" (tok/s):
        regression if delta < -20%
    if metric is "lower is better" (latency):
        regression if delta > +20%
```

**Threshold:** 20% deviation triggers a regression warning in `performance.json`.
This is a warning, not a gate — performance can vary on shared hardware.

**No history?** First 7 days of operation have no baseline. Cohete records
metrics but skips regression detection until 7 daily snapshots exist.

---

## Artifact Integrity

Each artifact file includes:
- `schema_version`: for forward compatibility
- `timestamp`: ISO 8601 UTC
- `cohete_version` + `cohete_sha`: which cohete produced this

If the schema changes, bump `schema_version`. Cohete should be able to
read older schema versions for regression comparison.

---

## CI Commit Protocol

The nightly workflow commits artifacts after a run:

```bash
cd artifacts/history
cp ../latest/summary.json "$(date +%Y-%m-%d).json"
git add "$(date +%Y-%m-%d).json"
git commit -m "nightly: $(date +%Y-%m-%d) [pass/fail]"
git push
```

**Commit message format:** `nightly: YYYY-MM-DD [pass]` or `nightly: YYYY-MM-DD [FAIL]`

This keeps the git log scannable:
```
nightly: 2026-03-10 [pass]
nightly: 2026-03-09 [pass]
nightly: 2026-03-08 [FAIL]
nightly: 2026-03-07 [pass]
```
