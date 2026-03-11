#!/usr/bin/env python3
"""Update README nightly section from cohete artifacts.

Reads artifacts/latest/*.json and updates README.md between
<!-- NIGHTLY:BEGIN --> and <!-- NIGHTLY:END --> markers.

Zero external dependencies — stdlib only.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "artifacts" / "latest"
README = ROOT / "README.md"


def load(name):
    p = ARTIFACTS / name
    if not p.exists():
        return None
    return json.loads(p.read_text())


def generate_markdown(summary, functional, integration):
    s = summary
    date = s.get("date", "\u2014")
    overall = s.get("pass", False)
    dur = s.get("duration_s", 0)
    tiers = s.get("tiers", {})
    hw = s.get("hardware") or {}
    met = s.get("metrics") or {}
    bins = s.get("binaries", [])

    status = "PASS" if overall else "FAIL"
    lines = []
    lines.append(f"*Last run: **{date}** \u2014 **{status}** ({dur}s)*")
    lines.append("")

    # Tier summary table
    lines.append("### Tier Results")
    lines.append("")
    lines.append("| Tier | Name | Status | Passed | Failed | Skipped |")
    lines.append("|------|------|--------|--------|--------|---------|")

    tier_names = [
        ("1", "Smoke", "smoke"),
        ("2", "Hardware", "hardware"),
        ("3", "Functional", "functional"),
        ("4", "Integration", "integration"),
        ("5", "Performance", "performance"),
    ]
    for num, label, key in tier_names:
        d = tiers.get(key)
        if d is None:
            lines.append(f"| {num} | {label} | \u2014 | \u2014 | \u2014 | \u2014 |")
            continue
        icon = "\u2705" if d.get("pass") else "\u274c"
        if key == "performance":
            reg = d.get("regressions", 0)
            lines.append(
                f"| {num} | {label} | {icon} | {reg} regressions | \u2014 | \u2014 |"
            )
        else:
            p = d.get("passed", 0)
            f = d.get("failed", 0)
            sk = d.get("skipped", 0)
            lines.append(f"| {num} | {label} | {icon} | {p} | {f} | {sk} |")

    # Binary versions
    lines.append("")
    lines.append("### Binary Versions")
    lines.append("")
    lines.append("| Binary | Version | Status |")
    lines.append("|--------|---------|--------|")
    for b in bins:
        name = f"`{b['name']}`"
        ver = b.get("version") or "\u2014"
        if ver != "\u2014" and ver.startswith(b["name"] + " "):
            ver = ver[len(b["name"]) + 1 :]
        icon = "\u2705" if b.get("installed") else "\u2b1c"
        state = "installed" if b.get("installed") else "missing"
        lines.append(f"| {name} | {ver} | {icon} {state} |")

    # Format x Backend matrix
    if functional:
        inf_tests = [
            t
            for t in functional.get("tests", [])
            if t["test"].startswith("inference_")
        ]
        if inf_tests:
            lines.append("")
            lines.append("### Format x Backend Matrix")
            lines.append("")
            lines.append("|        | GPU | CPU |")
            lines.append("|--------|-----|-----|")
            grid = {}
            for t in inf_tests:
                parts = t["test"].split("_")
                if len(parts) == 3:
                    fmt = parts[1].upper()
                    bk = parts[2].upper()
                    st = t["status"]
                    ms = t["duration_ms"]
                    icon = "\u2705" if st == "pass" else "\u274c" if st == "fail" else "\u26a0\ufe0f"
                    grid[(fmt, bk)] = f"{icon} {ms / 1000:.1f}s"
            for fmt in ["GGUF", "APR"]:
                gpu = grid.get((fmt, "GPU"), "\u2014")
                cpu = grid.get((fmt, "CPU"), "\u2014")
                lines.append(f"| **{fmt}** | {gpu} | {cpu} |")

    # Correctness
    if integration:
        m3 = integration.get("modalities", {}).get("M3_correctness")
        if m3:
            lines.append("")
            lines.append(
                f"### Correctness (M3): {m3['passed']}/{m3['total']} passed"
            )

    # UAT: Real-World Problem Solving
    if integration:
        uat = integration.get("uat")
        if uat:
            lines.append("")
            lines.append("### UAT: Real-World Problem Solving")
            lines.append("")
            lines.append("| Suite | Passed | Total | Status |")
            lines.append("|-------|--------|-------|--------|")
            uat_suites = [
                ("U1 Chat Solving", "U1_chat_problem_solving"),
                ("U2 API Validation", "U2_serve_api"),
                ("U3 Kernel Provability", "U3_kernel_provability"),
                ("U4 Task Chaining", "U4_task_chaining"),
            ]
            for label, key in uat_suites:
                suite = uat.get(key)
                if suite:
                    sp = suite.get("passed", 0)
                    st = suite.get("total", 0)
                    icon = "\u2705" if suite.get("pass") else "\u274c"
                    lines.append(f"| {label} | {sp} | {st} | {icon} |")
                else:
                    lines.append(f"| {label} | \u2014 | \u2014 | \u2014 |")

    # Performance
    lines.append("")
    lines.append("### Performance")
    lines.append("")
    lines.append("| Metric | Value |")
    lines.append("|--------|-------|")
    tok = met.get("inference_tok_s")
    lines.append(f"| Inference | {f'{tok:.1f} tok/s' if tok else chr(0x2014)} |")
    rtf = met.get("whisper_rtf")
    lines.append(f"| Whisper RTF | {f'{rtf:.2f}' if rtf else chr(0x2014)} |")
    rag = met.get("rag_query_ms")
    lines.append(f"| RAG query | {f'{rag:.0f} ms' if rag else chr(0x2014)} |")
    mem = met.get("memory_available_mb")
    lines.append(
        f"| Memory available | {f'{mem // 1024} GB' if mem else chr(0x2014)} |"
    )

    # Hardware
    lines.append("")
    lines.append("### Hardware")
    lines.append("")
    lines.append("| Property | Value |")
    lines.append("|----------|-------|")
    lines.append(f"| GPU | {hw.get('gpu') or chr(0x2014)} |")
    lines.append(f"| CUDA | {hw.get('cuda') or chr(0x2014)} |")
    neon = "yes" if hw.get("neon") else "no"
    lines.append(f"| NEON | {neon} |")
    if hw.get("jetpack"):
        lines.append(f"| JetPack | {hw['jetpack']} |")
    if hw.get("power_mode"):
        lines.append(f"| Power | {hw['power_mode']} |")

    return "\n".join(lines)


BEGIN = "<!-- NIGHTLY:BEGIN -->"
END = "<!-- NIGHTLY:END -->"


def update_readme(md_content):
    if not README.exists():
        print(f"  SKIP: {README} not found", file=sys.stderr)
        return

    text = README.read_text()
    b = text.find(BEGIN)
    e = text.find(END)
    if b == -1 or e == -1:
        print(f"  SKIP: markers not found in README.md", file=sys.stderr)
        return

    new = text[: b + len(BEGIN)] + "\n" + md_content + "\n" + text[e:]
    README.write_text(new)
    print(f"  wrote {README}", file=sys.stderr)


def main():
    summary = load("summary.json")
    if summary is None:
        print("No artifacts/latest/summary.json — run cohete verify first", file=sys.stderr)
        sys.exit(1)

    functional = load("functional.json")
    integration = load("integration.json")

    md = generate_markdown(summary, functional, integration)
    update_readme(md)


if __name__ == "__main__":
    main()
