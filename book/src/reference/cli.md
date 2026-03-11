# CLI Usage

## Synopsis

```
cohete verify [OPTIONS]
```

Cohete has a single command: `verify`. It runs up to 5 tiers of verification
and emits JSON artifacts.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-o, --output <DIR>` | `artifacts/latest` | Directory to write artifact JSON files |
| `--max-tier <N>` | `5` | Only run tiers 1 through N |
| `--stdout` | `false` | Print results to stdout instead of writing files |
| `--allow-missing` | `false` | Continue past tier 1 if some binaries are missing |
| `--model <PATH>` | *(auto)* | Path to LLM model file (`.gguf` or `.apr`) |

## Examples

```bash
# Full run, artifacts to default directory
cohete verify

# Print to terminal, tolerate missing binaries
cohete verify --stdout --allow-missing

# Only run tiers 1-3
cohete verify --max-tier 3

# Specify a model explicitly
cohete verify --model /path/to/model.gguf

# Custom output directory
cohete verify --output artifacts/2026-03-11
```

## Exit Code

- **0** — all executed tiers passed
- **1** — one or more tiers failed

With `--allow-missing`, missing binaries in tier 1 are treated as warnings
(not failures). Only *installed* binaries must pass their smoke tests.

## Artifact Output

When `--stdout` is not set, each tier writes a JSON file to the output directory:

```
artifacts/latest/
├── smoke.json         # tier 1
├── hardware.json      # tier 2
├── functional.json    # tier 3
├── integration.json   # tier 4
├── performance.json   # tier 5
└── summary.json       # overall
```

See [Artifact Schemas](../spec/artifacts.md) for the JSON structure.
