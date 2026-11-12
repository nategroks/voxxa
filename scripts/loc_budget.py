#!/usr/bin/env python3
"""Fail if code exceeds the per-module line budgets of ADR-010 Decision 3, as amended by A8.

Unit: tokei "code" lines (no blanks, no comments). CSS/JS embedded in an HTML file
(tokei "blobs") is added to that HTML file's count, so moving GUI code inline does not
hide it. Budgets can only go down.
Usage: tokei must be on PATH. Run from the repo root.
"""
import json
import subprocess
import sys

RUST = "src-tauri/src/"
GUI_PATHS = ["src", "index.html", "stage.html"]
GUI_LANGS = ["JavaScript", "HTML", "CSS"]
PER_DRIVER_LIMIT = 400


def per_file(paths, langs):
    """{normalised path: tokei code lines} for the given paths and languages."""
    out = subprocess.run(["tokei", "-o", "json", *paths], check=True, capture_output=True, text=True).stdout
    data = json.loads(out)
    files = {}
    for lang in langs:
        for report in data.get(lang, {}).get("reports", []):
            name = report["name"].replace("\\", "/")
            name = name[2:] if name.startswith("./") else name
            stats = report["stats"]
            code = stats["code"] + sum(b["code"] for b in stats.get("blobs", {}).values())
            files[name] = files.get(name, 0) + code
    return files


rust = per_file([RUST], ["Rust"])
gui = per_file(GUI_PATHS, GUI_LANGS)


def total(prefixes):
    return sum(n for f, n in rust.items() if any(f.startswith(RUST + p) for p in prefixes))


conductor = total(["aligner/"])
presenters = total(["presenters/"])
importers = total(["importers/", "planning_center/"])
rest = sum(rust.values()) - conductor - presenters - importers

checks = [
    ("conductor (aligner/)", conductor, 1200),
    ("presenters/ total", presenters, 2000),
    ("importers/ + planning_center/", importers, 1600),
    ("everything else in src-tauri/src", rest, 2500),
    ("GUI (src/, index.html, stage.html)", sum(gui.values()), 3000),
]
for f, n in sorted(rust.items()):
    if f.startswith(RUST + "presenters/") and not f.endswith("/mod.rs"):
        checks.append((f"driver {f[len(RUST):]}", n, PER_DRIVER_LIMIT))

failed = False
for label, n, limit in checks:
    over = n > limit
    failed |= over
    print(f"{'OVER' if over else 'OK  '} {label}: {n} / {limit}")

with open("package.json", encoding="utf-8") as fh:
    runtime_deps = json.load(fh).get("dependencies", {})
if runtime_deps:
    failed = True
    print(f"OVER runtime npm dependencies: {sorted(runtime_deps)} (the GUI must have none)")
else:
    print("OK   runtime npm dependencies: none")

sys.exit(1 if failed else 0)
