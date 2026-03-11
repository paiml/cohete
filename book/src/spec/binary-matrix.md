# Component Spec: Binary Matrix

**Parent:** [cohete-e2e-demo.md](architecture.md)
**Scope:** Which binaries cohete tests, where they come from, how they're installed

---

## Acquisition Path

Cohete does NOT download or install binaries. The chain is:

```
repo nightly workflow (04:00 UTC)
    → builds aarch64-unknown-linux-gnu binary
    → uploads to GitHub Release (tag: nightly)

forjar nightly binary (05:00 UTC)
    → provisions jetson
    → downloads nightly releases for each binary
    → installs to ~/.cargo/bin/
    → syncs models + test data via copia

cohete nightly (06:00 UTC)
    → assumes binaries exist at ~/.cargo/bin/
    → assumes models exist at ~/data/models/canary/
    → tests them all
```

If a binary is missing, cohete reports it as a tier 1 failure. The fix is
upstream: either the repo's nightly didn't build, or forjar's provision
didn't install it.

---

## Binary Inventory

### AI/ML Stack (modality-tested)

| Binary | Repo | GitHub | Nightly | Modalities | Features |
|--------|------|--------|---------|------------|----------|
| `apr` | aprender | [paiml/aprender](https://github.com/paiml/aprender) | [nightly](https://github.com/paiml/aprender/releases/tag/nightly) | M1,M2,M3,M4 | inference |
| `whisper-apr` | whisper.apr | [paiml/whisper.apr](https://github.com/paiml/whisper.apr) | [nightly](https://github.com/paiml/whisper.apr/releases/tag/nightly) | M5,M6 | cli |
| `trueno-rag` | trueno-rag | [paiml/trueno-rag](https://github.com/paiml/trueno-rag) | [nightly](https://github.com/paiml/trueno-rag/releases/tag/nightly) | M6 | — |

These binaries prove the sovereign stack handles inference, transcription,
and retrieval on edge hardware.

### Core Tools (smoke-tested)

| Binary | Repo | GitHub | Nightly | Features |
|--------|------|--------|---------|----------|
| `forjar` | forjar | [paiml/forjar](https://github.com/paiml/forjar) | [nightly](https://github.com/paiml/forjar/releases/tag/nightly) | — |
| `pmat` | pmat | [paiml/paiml-mcp-agent-toolkit](https://github.com/paiml/paiml-mcp-agent-toolkit) | [nightly](https://github.com/paiml/paiml-mcp-agent-toolkit/releases/tag/nightly) | — |
| `copia` | copia | [paiml/copia](https://github.com/paiml/copia) | [nightly](https://github.com/paiml/copia/releases/tag/nightly) | cli |
| `pzsh` | pzsh | [paiml/pzsh](https://github.com/paiml/pzsh) | [nightly](https://github.com/paiml/pzsh/releases/tag/nightly) | — |
| `batuta` | batuta | [paiml/batuta](https://github.com/paiml/batuta) | [nightly](https://github.com/paiml/batuta/releases/tag/nightly) | native,rag |

These must run on every nightly. No exceptions.

### Not Tested by Cohete

| Binary | Repo | Why excluded |
|--------|------|-------------|
| `trueno-zram` | trueno-zram | System-level, requires root |
| `entrenar` | entrenar | Training workload, too heavy |
| `repartir-worker` | repartir | Needs cluster |
| `faro` | faro | Needs metrics source |
| `rmedia` | rmedia | Needs heavy inputs |
| `simular` | simular | Not edge-relevant |
| `pacha` | pacha | Needs registry network |
| Others | — | Not edge-relevant |

---

## Model & Test Data Dependencies

| Data | Path | Size | Provisioned by | Required for |
|------|------|------|----------------|-------------|
| Qwen 1.5B Q4K | ~/data/models/canary/qwen-1.5b-q4k.apr | ~1 GB | forjar | M1,M2,M3,M4 (tier 3/4/5) |
| Whisper tiny.en | ~/data/models/canary/whisper-tiny.en | ~75 MB | forjar | M5,M6 (tier 3/4) |
| Test audio (2s) | ~/data/models/canary/test-2s.wav | ~32 KB | forjar | M5,M6 (tier 3/4) |
| Correctness tests | cohete binary (built-in) | — | cohete build | M3 (tier 4) |
| Oracle index | ~/.batuta-private.toml → SQLite | varies | forjar | batuta smoke |

Missing models/data cause individual test **skips**, not failures. Cohete
distinguishes "binary broken" (fail) from "test data missing" (skip).

---

## Version Tracking

Cohete records every installed binary's version in `smoke.json`. This
creates a nightly audit trail:

```
artifacts/history/2026-03-10.json → apr 0.28.0, whisper-apr 0.2.5, ...
artifacts/history/2026-03-11.json → apr 0.28.1, whisper-apr 0.2.5, ...
```

**Version bump detection:** If a version changes from yesterday, cohete
logs it in `summary.json` under `"version_changes"`. This correlates
regressions with specific upstream releases.

---

## forjar Integration

Forjar's jetson `forjar.yaml` must include resources that:

1. Download aarch64 nightly binaries from GitHub Releases (tag: `nightly`)
2. Install them to `~/.cargo/bin/`
3. Sync model files to `~/data/models/canary/`
4. Deploy test fixtures (audio WAV)

```yaml
# forjar.yaml — github_release resources (FJ-034)
install-forjar:
  type: github_release
  machine: jetson
  repo: paiml/forjar
  tag: nightly
  asset_pattern: "*aarch64-unknown-linux-gnu*"
  binary: forjar
  install_dir: /home/noah/.cargo/bin

install-apr:
  type: github_release
  machine: jetson
  repo: paiml/aprender
  tag: nightly
  asset_pattern: "*aarch64-unknown-linux-gnu*"
  binary: apr
  install_dir: /home/noah/.cargo/bin

install-whisper-apr:
  type: github_release
  machine: jetson
  repo: paiml/whisper.apr
  tag: nightly
  asset_pattern: "*aarch64-unknown-linux-gnu*"
  binary: whisper-apr
  install_dir: /home/noah/.cargo/bin

# ... same pattern for copia, pmat, pzsh, trueno-rag, batuta
```

The `github_release` resource type (FJ-034) is implemented in forjar.
It uses `gh release download` to fetch assets, extracts tarballs/zips
automatically, installs the binary, and verifies with `--version`.
State is tracked via BLAKE3 hash of version + file size for drift detection.
