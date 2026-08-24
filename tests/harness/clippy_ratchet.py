#!/usr/bin/env python3
"""Clippy warning ratchet for the custom quality gate (QA-001).

Enforces that the number of DISTINCT clippy warnings never exceeds the
recorded per-platform baseline. Warnings are deduplicated by primary
span (file, line, column) plus lint name/message, because --all-targets
reports the same source warning once per compiled target.

The baseline reflects the upstream v0.9.6 state; custom code must not
add warnings. When a platform has no baseline entry yet (first run on a
new platform), the measured value is written back as the initial ceiling
instead of failing.

Usage:
    python tests/harness/clippy_ratchet.py [--update]

Exit codes: 0 = pass, 1 = gate violation, 2 = environment error.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

BASELINE_PATH = Path(__file__).resolve().parent / "clippy-baseline.json"


def platform_key() -> str:
    if sys.platform.startswith("win"):
        return "windows"
    if sys.platform == "darwin":
        return "macos"
    return "linux"


def warning_key(msg: dict) -> tuple | None:
    """Dedup key from a compiler-message diagnostic, or None if not a warning."""
    if msg.get("level") != "warning":
        return None
    spans = msg.get("spans") or []
    primary = next((s for s in spans if s.get("is_primary")), spans[0] if spans else None)
    if primary is None:
        # e.g. command-line warnings without span; fall back to message text
        return ("-", msg.get("message", ""))
    file_name = primary.get("file_name", "-")
    line = primary.get("line_start", 0)
    col = primary.get("column_start", 0)
    code = None
    code_obj = msg.get("code")
    if isinstance(code_obj, dict):
        code = code_obj.get("code")
    # prefer stable identity: file:line:col + lint code, message as tiebreaker
    return (file_name, line, col, code or "", msg.get("message", ""))


def collect_warnings() -> set[tuple]:
    """Run clippy over all targets and return the set of distinct warnings."""
    proc = subprocess.run(
        ["cargo", "clippy", "--all-targets", "--message-format=json"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if proc.returncode != 0:
        print(proc.stderr[-4000:], file=sys.stderr)
        print("[ratchet] cargo clippy itself failed", file=sys.stderr)
        raise SystemExit(2)

    warnings: set[tuple] = set()
    for line in (proc.stdout or "").splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            payload = json.loads(line)
        except json.JSONDecodeError:
            continue
        if payload.get("reason") != "compiler-message":
            continue
        msg = payload.get("message") or {}
        key = warning_key(msg)
        if key is not None:
            warnings.add(key)
    return warnings


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update", action="store_true",
                        help="overwrite the platform ceiling with the current count")
    args = parser.parse_args()

    baseline = json.loads(BASELINE_PATH.read_text(encoding="utf-8"))
    key = platform_key()
    warnings = collect_warnings()
    current = len(warnings)
    print(f"[ratchet] distinct clippy warnings ({key}): {current}")

    entry = baseline.get(key)
    if entry is None:
        baseline[key] = {"max_warnings": current}
        BASELINE_PATH.write_text(json.dumps(baseline, indent=2) + "\n", encoding="utf-8")
        print(f"[ratchet] no baseline for '{key}' -> initialized at {current}")
        raise SystemExit(0)

    ceiling = int(entry["max_warnings"])
    if args.update:
        baseline[key]["max_warnings"] = current
        BASELINE_PATH.write_text(json.dumps(baseline, indent=2) + "\n", encoding="utf-8")
        print(f"[ratchet] baseline for '{key}' updated to {current}")
        raise SystemExit(0)

    if current > ceiling:
        print(
            f"[ratchet] FAIL: {current} warnings exceed ceiling {ceiling} "
            f"for '{key}'. New offenders:",
            file=sys.stderr,
        )
        known = set()
        # best effort: list warnings not present in a stored signature dump
        sig_path = BASELINE_PATH.with_name(f"clippy-signatures-{key}.txt")
        if sig_path.exists():
            known = {l.rstrip("\\n") for l in sig_path.read_text(encoding="utf-8").splitlines() if l}
        for w in sorted(warnings):
            sig = repr(w)
            if sig not in known:
                print(f"  - {sig}", file=sys.stderr)
        print(
            "Fix the new warnings. Raising the ceiling requires owner approval.",
            file=sys.stderr,
        )
        raise SystemExit(1)

    if current < ceiling:
        print(
            f"[ratchet] PASS: {current} < ceiling {ceiling}. Consider lowering "
            f"tests/harness/clippy-baseline.json['{key}']['max_warnings']."
        )
    else:
        print(f"[ratchet] PASS: {current} <= ceiling {ceiling}")

    # persist signatures so future violations can be diffed precisely
    sig_path = BASELINE_PATH.with_name(f"clippy-signatures-{key}.txt")
    sig_path.write_text(
        "\n".join(sorted(repr(w) for w in warnings)) + "\n", encoding="utf-8"
    )
    raise SystemExit(0)


if __name__ == "__main__":
    main()
