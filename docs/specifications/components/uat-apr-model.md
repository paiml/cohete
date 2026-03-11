# Component Spec: APR Model User Acceptance Testing

**Parent:** [cohete-e2e-demo.md](../cohete-e2e-demo.md)
**Scope:** Prove the APR model solves real-world problems via chat, serve, kernel provability, and task chaining
**Refs:** PMAT-015

---

## Overview

Tiers 3-4 verify that the model *runs*. UAT verifies that the model *works* —
that it solves real-world problems a developer would actually ask. This spec
defines four UAT modalities (U1-U4) that produce falsifiable JSON artifacts.

**Philosophy:** Popperian falsification. Each UAT scenario defines a hypothesis
("model can solve X") and a pass criteria that would *refute* the hypothesis
if unmet. Models pass by surviving severe attempts at refutation.

**Budget:** 120 seconds total, shared with tier 4 (reuses the same `apr serve` instance).

---

## U1: Chat Problem Solving

**Hypothesis:** The model produces correct, usable solutions to real-world coding problems.

Unlike M3 (correctness), which tests simple factual recall ("7 * 8 = 56"),
U1 tests multi-step problem solving requiring reasoning.

### Scenarios

| ID | Scenario | Prompt | Pass Criteria |
|----|----------|--------|---------------|
| U1-001 | Parse CSV | "Write a Rust function that parses CSV" | Contains `fn` + `&str` or `String` + `split` or `csv` |
| U1-002 | Error handling | "Add error handling to this: `fn div(a: i32, b: i32) -> i32 { a / b }`" | Contains `Result` or `Option` + `0` |
| U1-003 | Unit test | "Write a test for: `fn add(a: i32, b: i32) -> i32 { a + b }`" | Contains `#[test]` + `assert` |
| U1-004 | Debug analysis | "This panics: `vec![1,2,3][5]`. Why and how to fix?" | Contains `bounds` or `index` or `len` + `get` or `check` |
| U1-005 | Algorithm | "Write binary search in Python" | Contains `def` + `mid` or `//` or `>>` + `return` |

**Determinism:** All prompts sent with `temperature: 0`, `max_tokens: 256`.

**Scoring:** Each scenario is independent. A model that passes 4/5 is still useful.
UAT reports individual results — no single failure blocks the run.

### Artifact Schema

```json
{
  "U1_chat_problem_solving": {
    "pass": true,
    "total": 5,
    "passed": 5,
    "scenarios": [
      { "id": "U1-001", "pass": true, "prompt_hash": "a1b2c3", "duration_ms": 2400 },
      { "id": "U1-002", "pass": true, "prompt_hash": "d4e5f6", "duration_ms": 1800 }
    ]
  }
}
```

---

## U2: Serve API Validation

**Hypothesis:** The HTTP API is production-ready — correct responses, proper errors, graceful behavior.

Unlike M2 (chat server), which only checks `/health`, U2 validates the full API surface
that a developer would integrate against.

### Scenarios

| ID | Scenario | Method | Pass Criteria |
|----|----------|--------|---------------|
| U2-001 | Predict endpoint | `POST /v1/chat/completions` (valid) | 200, valid JSON, `choices[0].message.content` non-empty |
| U2-002 | Streaming | `POST /v1/chat/completions` with `stream: true` | 200, receives `data:` SSE chunks, final `[DONE]` |
| U2-003 | Invalid JSON | `POST /v1/chat/completions` with `{bad` | 400, JSON error response |
| U2-004 | Missing field | `POST /v1/chat/completions` with `{"messages": []}` | 400 or 200 (graceful), no crash |
| U2-005 | Model info | `GET /v1/models` | 200, JSON with model ID |
| U2-006 | Health check | `GET /health` (10 rapid calls) | All 200, no degradation |

**Determinism:** Request bodies are fixed. Responses validated structurally (not content).

### Artifact Schema

```json
{
  "U2_serve_api": {
    "pass": true,
    "total": 6,
    "passed": 6,
    "scenarios": [
      { "id": "U2-001", "pass": true, "status_code": 200, "duration_ms": 450 },
      { "id": "U2-003", "pass": true, "status_code": 400, "duration_ms": 12 }
    ]
  }
}
```

---

## U3: Kernel Provability

**Hypothesis:** The inference pipeline produces numerically reproducible results —
same model, same prompt, same seed yields identical output.

This is the computational kernel's contract: determinism is provable,
not just hoped for. Without this, no other test is scientifically meaningful.

### Scenarios

| ID | Scenario | Method | Pass Criteria |
|----|----------|--------|---------------|
| U3-001 | Reflexivity | Run same prompt 3x with `seed: 42` | All 3 outputs identical (byte-equal) |
| U3-002 | Cardinality | Request `max_tokens: 8` | Output has <= 8 tokens |
| U3-003 | Format parity | Run prompt on GGUF, then APR | Outputs match (if both formats available) |
| U3-004 | Token stability | Run "2+2=" with `temperature: 0` | Output contains "4", stable across runs |

**Invariants (from apr-model-qa-playbook I-1..I-5):**

| Invariant | Property | What It Catches |
|-----------|----------|-----------------|
| I-1 | Round-trip identity | `inference(convert(model)) == inference(original)` |
| I-2 | Tensor name bijection | Writer names == loader names |
| I-3 | No silent fallbacks | Unknown input → error, never default |

**Implementation:** U3-003 (format parity) is cohete's implementation of invariant I-1.
It's the most powerful single test — catches all conversion corruption.

### Artifact Schema

```json
{
  "U3_kernel_provability": {
    "pass": true,
    "total": 4,
    "passed": 4,
    "scenarios": [
      { "id": "U3-001", "pass": true, "runs": 3, "all_identical": true },
      { "id": "U3-003", "pass": true, "gguf_output": "4", "apr_output": "4", "match": true }
    ]
  }
}
```

---

## U4: Task Chaining

**Hypothesis:** The model can participate in multi-step workflows where each step
depends on the previous output.

This is the gap identified across the Sovereign AI Stack — models are tested
in isolation but never in chains. Real-world usage is always chained:
user asks → model responds → tool runs → model synthesizes.

### Scenarios

| ID | Scenario | Steps | Pass Criteria |
|----|----------|-------|---------------|
| U4-001 | Two-turn context | Turn 1: "Remember: x=42" → Turn 2: "What is x?" | Turn 2 contains "42" |
| U4-002 | Iterative refinement | Turn 1: "Write hello world in Rust" → Turn 2: "Now add a CLI flag --name" | Turn 2 contains `clap` or `args` or `--name` |
| U4-003 | RAG-augmented | Query trueno-rag → inject context → ask model to summarize | Summary references retrieved content |
| U4-004 | Error correction | Turn 1: "1+1=3, right?" → model should correct | Contains "2" and not affirm "3" |

**Implementation:** U4-001 and U4-002 use multi-turn chat completions (message array).
U4-003 chains trueno-rag query output into the model prompt.
U4-004 validates the model doesn't blindly agree.

### Artifact Schema

```json
{
  "U4_task_chaining": {
    "pass": true,
    "total": 4,
    "passed": 4,
    "scenarios": [
      { "id": "U4-001", "pass": true, "turns": 2, "duration_ms": 3200 },
      { "id": "U4-003", "pass": true, "rag_query": "test", "rag_hit": true, "duration_ms": 4100 }
    ]
  }
}
```

---

## Integration with Existing Tiers

UAT runs inside tier 4 (integration), sharing the `apr serve` instance:

```
tier 4: integration
  ├── M2: chat server startup        (existing)
  ├── M3: correctness (6 tests)      (existing)
  ├── M4: load test                   (existing)
  ├── U1: chat problem solving (5)    (NEW)
  ├── U2: serve API validation (6)    (NEW)
  ├── U3: kernel provability (4)      (NEW — may use CLI, not server)
  ├── U4: task chaining (4)           (NEW)
  └── M6: RAG pipeline               (existing)
```

**Server lifecycle:** M2 starts server → M3+U1+U2+U4 use it → M4 load tests → kill server.
U3 may run `apr run` directly (CLI mode) for format parity checks.

---

## Forjar Orchestration

UAT scenarios are defined as forjar `task` resources for reproducible execution:

```yaml
# forjar-uat.yaml — see project root
resources:
  uat-pipeline:
    type: task
    task_mode: pipeline
    stages:
      - name: chat-solve     # U1
      - name: api-validate   # U2
      - name: kernel-prove   # U3
      - name: chain-tasks    # U4
```

This ensures:
1. **Reproducibility** — forjar records BLAKE3 hashes of inputs/outputs
2. **Deterministic ordering** — DAG execution respects dependencies
3. **Provenance** — `forjar trace` shows exactly what ran and when

---

## Artifact Output

UAT results are emitted as part of `integration.json` under a new `uat` key:

```json
{
  "tier": 4,
  "pass": true,
  "modalities": {
    "M2_chat_server": { ... },
    "M3_correctness": { ... },
    "M4_load_test": { ... },
    "M6_rag_pipeline": { ... }
  },
  "uat": {
    "pass": true,
    "total": 19,
    "passed": 19,
    "failed": 0,
    "U1_chat_problem_solving": { ... },
    "U2_serve_api": { ... },
    "U3_kernel_provability": { ... },
    "U4_task_chaining": { ... }
  }
}
```

---

## README Integration

`scripts/generate-status.py` renders UAT results between the existing markers:

```markdown
### UAT: Real-World Problem Solving

| Suite | Passed | Total | Status |
|-------|--------|-------|--------|
| U1 Chat Solving | 5 | 5 | PASS |
| U2 API Validation | 6 | 6 | PASS |
| U3 Kernel Provability | 4 | 4 | PASS |
| U4 Task Chaining | 4 | 4 | PASS |
```

---

## Scientific Reproducibility

Every UAT run satisfies three properties:

1. **Deterministic inputs** — fixed prompts, temperature=0, fixed seeds
2. **Falsifiable outputs** — JSON artifacts with exact pass/fail criteria
3. **Auditable provenance** — forjar trace + git-committed history snapshots

A third party can reproduce any UAT result by:

```bash
# 1. Check out the cohete commit
git log --oneline -1

# 2. Run the same UAT
cohete verify --max-tier 4

# 3. Compare artifacts
diff artifacts/latest/integration.json expected.json
```

No network calls, no API keys, no randomness. Pure local inference
with fixed seeds. This is the definition of scientifically reproducible.
