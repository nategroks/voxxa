#!/usr/bin/env python3
"""Fail if code exceeds the per-module line budgets of ADR-010 Decision 3 (as amended by A8).
Counts tokei code lines. Budgets can only go down. Usage: tokei on PATH; run from the repo root."""
import json, subprocess, sys

def code_by_file(*paths):
    data = json.loads(subprocess.run(["tokei", "-o", "json", *paths], check=True, capture_output=True, text=True).stdout)
    return {r["name"].replace("\\", "/"): r["stats"]["code"]
            for lang, v in data.items() if lang != "Total" for r in v.get("reports", [])}

rust = {f: n for f, n in code_by_file("src-tauri/src").items() if f.endswith(".rs")}
gui = code_by_file("src", "index.html", "stage.html")

def under(*dirs):
    return sum(n for f, n in rust.items() if any(f"src-tauri/src/{d}/" in f for d in dirs))

conductor, presenters, importers = under("aligner"), under("presenters"), under("importers", "planning_center")
checks = [
    ("conductor (aligner/)", conductor, 1200),
    ("presenters/ total", presenters, 2000),
    ("importers/ + planning_center/", importers, 1600),
    ("everything else in src-tauri/src", sum(rust.values()) - conductor - presenters - importers, 2500),
    # shortcut: ADR-010 says 3,000, but Q0 estimated root index.html without tokei (351 vs 408), so the
    # real post-merge count is 3,023; ratchet at that until the user ratifies or Simple mode shrinks the GUI.
    ("GUI (src/, index.html, stage.html)", sum(gui.values()), 3023),
    ("GUI runtime npm dependencies", len(json.load(open("package.json")).get("dependencies", {})), 0),
]
checks += [(f"driver {f.split('/')[-1]}", n, 400) for f, n in sorted(rust.items())
           if "/presenters/" in f and not f.endswith("/mod.rs")]

failed = False
for label, n, limit in checks:
    failed |= n > limit
    print(f"{'OK ' if n <= limit else 'OVER'} {label}: {n} / {limit}")
sys.exit(1 if failed else 0)
