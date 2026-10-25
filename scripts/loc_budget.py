#!/usr/bin/env python3
"""Fail if code exceeds the line budgets in .10x/specs/2026-10-24-software-strategy.md (Maintenance model).
Usage: tokei must be on PATH. Run from the repo root."""
import json, subprocess, sys

BUDGETS = [
    # (label, tokei args, languages counted, max code lines)
    ("core Rust (src-tauri/src minus adapters)", ["src-tauri/src", "-e", "adapters"], ["Rust"], 3000),
    ("GUI (src/)", ["src"], ["JavaScript", "HTML", "CSS"], 800),
]

def code_lines(args, langs):
    out = subprocess.run(["tokei", "-o", "json", *args], check=True, capture_output=True, text=True).stdout
    data = json.loads(out)
    return sum(data.get(l, {}).get("code", 0) for l in langs)

failed = False
for label, args, langs, limit in BUDGETS:
    n = code_lines(args, langs)
    status = "OK " if n <= limit else "OVER"
    failed |= n > limit
    print(f"{status} {label}: {n} / {limit}")
sys.exit(1 if failed else 0)
