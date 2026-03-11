#!/usr/bin/env python3
"""Generate nightly status SVG and update README from cohete artifacts.

Reads artifacts/latest/*.json, produces docs/status.svg and updates
README.md between <!-- NIGHTLY:BEGIN --> and <!-- NIGHTLY:END --> markers.

Zero external dependencies — stdlib only.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ARTIFACTS = ROOT / "artifacts" / "latest"
SVG_OUT = ROOT / "docs" / "status.svg"
README = ROOT / "README.md"

# ── Colors ────────────────────────────────────────────────────────

C_PASS = "#1a7f37"
C_FAIL = "#cf222e"
C_WARN = "#9a6700"
C_SKIP = "#656d76"
C_HEADER = "#24292f"
C_FOOTER = "#f6f8fa"
C_BORDER = "#d0d7de"
C_TEXT = "#1f2328"
C_MUTED = "#656d76"
C_WHITE = "#ffffff"
C_LIGHT = "#8b949e"


def load(name):
    p = ARTIFACTS / name
    if not p.exists():
        return None
    return json.loads(p.read_text())


def tier_color(data):
    if data is None:
        return C_SKIP
    return C_PASS if data.get("pass") else C_FAIL


def esc(s):
    return str(s).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


# ── SVG Generation ───────────────────────────────────────────────

def generate_svg(summary, functional):
    W, HDR, TIER_H, FTR = 880, 44, 120, 36
    H = HDR + TIER_H + FTR
    COL = W // 5

    date = summary.get("date", "\u2014")
    overall = summary.get("pass", False)
    dur = summary.get("duration_s", 0)
    tiers = summary.get("tiers", {})
    hw = summary.get("hardware") or {}
    met = summary.get("metrics") or {}

    status_lbl = "PASS" if overall else "FAIL"
    status_c = C_PASS if overall else C_FAIL

    # Extract format x backend from functional tests
    fmt_backend = {}
    if functional:
        for t in functional.get("tests", []):
            if t["test"].startswith("inference_"):
                parts = t["test"].split("_")  # inference_gguf_gpu
                if len(parts) == 3:
                    fmt_backend[(parts[1].upper(), parts[2].upper())] = (
                        t["status"],
                        t["duration_ms"],
                    )

    tier_info = [
        ("SMOKE", "1", tiers.get("smoke")),
        ("HARDWARE", "2", tiers.get("hardware")),
        ("FUNCTIONAL", "3", tiers.get("functional")),
        ("INTEGRATION", "4", tiers.get("integration")),
        ("PERFORMANCE", "5", tiers.get("performance")),
    ]

    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
        '<defs><style>',
        '  text { font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace; }',
        '</style></defs>',
        # outer rect
        f'<rect width="{W}" height="{H}" rx="8" fill="{C_WHITE}" stroke="{C_BORDER}"/>',
        # header
        f'<rect width="{W}" height="{HDR}" rx="8" fill="{C_HEADER}"/>',
        f'<rect y="{HDR - 8}" width="{W}" height="8" fill="{C_HEADER}"/>',
        f'<text x="16" y="28" fill="{C_WHITE}" font-size="15" font-weight="700">'
        f'\u25c6 COHETE</text>',
        f'<text x="120" y="28" fill="{C_LIGHT}" font-size="11">'
        f'Sovereign AI Stack \u00b7 Nightly Verification</text>',
        # status badge
        f'<rect x="{W - 220}" y="12" width="56" height="20" rx="10" fill="{status_c}"/>',
        f'<text x="{W - 192}" y="26" fill="{C_WHITE}" font-size="10" '
        f'font-weight="600" text-anchor="middle">{status_lbl}</text>',
        f'<text x="{W - 148}" y="27" fill="{C_LIGHT}" font-size="11">{esc(date)}</text>',
        f'<text x="{W - 14}" y="27" fill="{C_LIGHT}" font-size="10" '
        f'text-anchor="end">{dur}s</text>',
    ]

    ty = HDR  # tier row y origin

    for i, (name, num, data) in enumerate(tier_info):
        x = i * COL
        cx = x + COL // 2
        c = tier_color(data)

        # separator line
        if i > 0:
            lines.append(
                f'<line x1="{x}" y1="{ty}" x2="{x}" y2="{ty + TIER_H}" '
                f'stroke="{C_BORDER}"/>'
            )

        # colored top accent
        lines.append(
            f'<rect x="{x}" y="{ty}" width="{COL}" height="3" fill="{c}"/>'
        )

        # tier number + name
        lines.append(
            f'<text x="{cx}" y="{ty + 20}" fill="{C_MUTED}" font-size="9" '
            f'text-anchor="middle">TIER {num}</text>'
        )
        lines.append(
            f'<text x="{cx}" y="{ty + 36}" fill="{C_TEXT}" font-size="12" '
            f'font-weight="600" text-anchor="middle">{name}</text>'
        )

        # status circle + counts
        if data is not None:
            lines.append(
                f'<circle cx="{cx - 16}" cy="{ty + 54}" r="4" fill="{c}"/>'
            )
            if name == "PERFORMANCE":
                reg = data.get("regressions", 0)
                count = f"{reg} reg"
            else:
                p = data.get("passed", 0)
                t = data.get("total", 0)
                count = f"{p}/{t}"
            lines.append(
                f'<text x="{cx - 8}" y="{ty + 58}" fill="{C_TEXT}" '
                f'font-size="12">{count}</text>'
            )
        else:
            lines.append(
                f'<text x="{cx}" y="{ty + 58}" fill="{C_SKIP}" '
                f'font-size="12" text-anchor="middle">\u2014</text>'
            )

        # detail lines
        details = _tier_details(name, data, met, fmt_backend, hw)
        for j, d in enumerate(details[:3]):
            lines.append(
                f'<text x="{cx}" y="{ty + 76 + j * 14}" fill="{C_MUTED}" '
                f'font-size="9" text-anchor="middle">{esc(d)}</text>'
            )

    # footer
    fy = ty + TIER_H
    lines.append(f'<rect y="{fy}" width="{W}" height="{FTR}" rx="8" fill="{C_FOOTER}"/>')
    lines.append(f'<rect y="{fy}" width="{W}" height="8" fill="{C_FOOTER}"/>')
    lines.append(f'<line x1="0" y1="{fy}" x2="{W}" y2="{fy}" stroke="{C_BORDER}"/>')

    parts = []
    if hw.get("gpu"):
        gpu_short = hw["gpu"].replace("NVIDIA GeForce ", "")
        parts.append(gpu_short)
    if hw.get("cuda"):
        parts.append(f"CUDA {hw['cuda']}")
    if hw.get("neon"):
        parts.append("NEON")
    if hw.get("jetpack"):
        parts.append(f"JP {hw['jetpack']}")
    parts.extend(["Qwen 1.5B Q4K", "GGUF + APR"])
    footer_text = " \u00b7 ".join(parts)
    lines.append(
        f'<text x="{W // 2}" y="{fy + 23}" fill="{C_MUTED}" font-size="10" '
        f'text-anchor="middle">{esc(footer_text)}</text>'
    )

    lines.append("</svg>")
    return "\n".join(lines)


def _tier_details(name, data, metrics, fmt_backend, hw):
    if data is None:
        return []
    if name == "SMOKE":
        f = data.get("failed", 0)
        p = data.get("passed", 0)
        if f:
            return [f"{f} missing", f"{p} installed"]
        return [f"all {p} OK"]
    if name == "HARDWARE":
        parts = []
        if hw.get("gpu"):
            parts.append(hw["gpu"].replace("NVIDIA GeForce ", ""))
        if hw.get("cuda"):
            parts.append(f"CUDA {hw['cuda']}")
        return parts
    if name == "FUNCTIONAL":
        parts = []
        # format x backend summary
        fmts = {}
        for (fmt, bk), (st, ms) in fmt_backend.items():
            sym = "\u2713" if st == "pass" else "\u2717" if st == "fail" else "~"
            fmts.setdefault(fmt, []).append(f"{bk}{sym}")
        for fmt, bks in sorted(fmts.items()):
            parts.append(f"{fmt}: {' '.join(bks)}")
        w = data.get("warned", 0)
        if w:
            parts.append(f"{w} warn (parity)")
        return parts
    if name == "INTEGRATION":
        return ["M2 chat  M3 correct", "M4 load  M6 RAG"]
    if name == "PERFORMANCE":
        parts = []
        tok = metrics.get("inference_tok_s")
        if tok:
            parts.append(f"{tok:.1f} tok/s")
        mem = metrics.get("memory_available_mb")
        if mem:
            parts.append(f"{mem // 1024} GB avail")
        return parts
    return []


# ── Markdown Generation ──────────────────────────────────────────

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
    lines.append(f"*Last run: **{date}** — **{status}** ({dur}s)*")
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
        # Strip binary name prefix from version string (e.g. "apr 0.4.10" -> "0.4.10")
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


# ── README Update ────────────────────────────────────────────────

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


# ── Main ─────────────────────────────────────────────────────────

def main():
    summary = load("summary.json")
    if summary is None:
        print("No artifacts/latest/summary.json — run cohete verify first", file=sys.stderr)
        sys.exit(1)

    functional = load("functional.json")
    integration = load("integration.json")

    # Generate SVG
    SVG_OUT.parent.mkdir(parents=True, exist_ok=True)
    svg = generate_svg(summary, functional)
    SVG_OUT.write_text(svg)
    print(f"  wrote {SVG_OUT}", file=sys.stderr)

    # Generate markdown and update README
    md = generate_markdown(summary, functional, integration)
    update_readme(md)


if __name__ == "__main__":
    main()
