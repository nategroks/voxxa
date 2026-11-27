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


# Recorded deviation from ADR-010 Decision 3 (ADR-016, user decision 2026-11-13):
# ADR-009's minimum (section labels, repeat-aware priors, word-token scoring,
# the evidence accumulator) gets its own row for aligner/evidence/ instead of
# raising the conductor row, which stays at 1200 and no longer counts it.
evidence = total(["aligner/evidence/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-028 Decision 2 and amendment 1,
# founder decision FA-54 2026-11-27, "take the number you need and multiply times
# 3"): the phrase-onset trigger (ADR-030: detector, arm/fire rules, break event)
# gets its own row for aligner/onset/, capped at 750 = the 250-line estimate x 3.
# It is split from the follower row's future scope, so the conductor row keeps
# its 1200 cap and no longer counts it. 0 lines at V7 (the path does not exist
# yet; the code lands with V11, planned in V3b). The review trigger fires when
# the row passes its 250 estimate.
onset = total(["aligner/onset/"])
conductor = total(["aligner/"]) - evidence - onset
presenters = total(["presenters/"])
importers = total(["importers/", "planning_center/"])
# Q7: both CLI tools moved from src/bin/ to src/cli/ so the installers leave
# them out (Cargo.toml, [[bin]] entries); the rows count the same files.
# Recorded deviation from ADR-010 Decision 3 (ADR-013, user decision 2026-11-12):
# the voxxad CLI binary gets its own row instead of raising "everything else".
# The listening loop it shares with the GUI lives in the library and counts in
# "everything else", which stays at 2500.
voxxad = total(["cli/voxxad/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-014, user decision 2026-11-12):
# the Whisper engine (transcription/, 251 code lines before Q4) and the new
# recognizer code (Recognizer trait, Hypothesis, the rolling window, all in
# transcription/) get one row instead of raising "everything else", which
# stays at 2500 and still holds the listening loop (listen.rs).
# Recorded deviation from ADR-010 Decision 3 (ADR-028 amendment 1 item 3, founder
# decision FA-54 2026-11-27): vad/ (125 code lines at 0190515, tokei 15.0.0) moves
# into this row, so the cap rises by exactly those 125 lines, 700 -> 825, and
# "everything else" falls by the same 125. A move under the same rule as the
# wiring split, not an increase: both headrooms are unchanged to the line
# (recognizer 268 left before and after, "everything else" 4 left before and
# after). V3b's energy gate (<= 40 lines) lands in vad/ inside this row.
recognizer = total(["transcription/", "vad/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-015, user decision 2026-11-13):
# the voxxa-eval binary (fixture scoring, Q5) gets its own row instead of
# raising "everything else", which stays at 2500.
voxxa_eval = total(["cli/voxxa-eval/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-017, user decision 2026-11-18):
# Simple mode and its first-run flow get their own row for src/simple/ instead
# of raising the GUI row, which stays at 3000 and no longer counts it.
simple = sum(n for f, n in gui.items() if f.startswith("src/simple/"))
# Recorded deviation from ADR-010 Decision 3 (ADR-028 amendment 1 item 2, founder
# decision FA-54 2026-11-27, "x 3"): the Sunday health check page (ADR-034; JS,
# HTML and CSS) gets its own row for src/health/, capped at 450 = the 150-line
# estimate x 3, subtracted from the GUI row the way src/simple/ is. 0 lines at
# V7 (the path does not exist yet; the code lands with V13s). The review trigger
# fires when the row passes its 150 estimate.
health = sum(n for f, n in gui.items() if f.startswith("src/health/"))
# Recorded deviation from ADR-010 Decision 3 (ADR-019 § 10 and its Q9W amendment,
# user decision D5 2026-11-20): discovery (183 code lines at b154a55) gets its own
# row, capped at 250, for finding WorshipTools Presenter by process. "Everything
# else" no longer counts it, so its cap goes down by those 183 lines, 2500 -> 2317.
discovery = total(["discovery/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-020 § 5, founder decision
# 2026-11-23, the D5 precedent of ADR-019 § 10): the operator-initiated updater
# (the comparator, the listening rule and its two commands) gets its own row,
# capped at 150, instead of raising "everything else", which stays at 2317.
updater = total(["updater/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-028 Decision 1 and amendment 1,
# founder decision FA-10 2026-11-26, option (a)): the one approved 150-line
# increase. dispatch.rs (147), listen.rs (159), http_api/ (238) and commands/
# (744), together 1288 code lines at 0190515 (tokei 15.0.0), move out of
# "everything else" into this "wiring" row, capped at 1288 + 150 = 1438. No file
# moves. "Everything else" no longer counts them, so its cap goes down by exactly
# those 1288 lines (and by vad/'s 125 above): 2317 -> 904; its headroom is
# unchanged to the line (2313 / 2317 before, 900 / 904 after, 4 left). Every
# later step's wiring share is listed in ADR-028 amendment 1 (sum <= 135 of
# the 150). Caps still only go down.
wiring = total(["dispatch.rs", "listen.rs", "http_api/", "commands/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-028 amendment 1 item 2, founder
# decision FA-54 2026-11-27, "x 3"): the input chain (high-pass, gain, level and
# hum numbers, the degraded flag; ADR-030 as amended) gets its own row for
# input/, capped at 600 = the 200-line estimate x 3. 0 lines at V7 (the path
# does not exist yet; the code lands with V3b task 6). The review trigger fires
# when the row passes its 200 estimate.
input_chain = total(["input/"])
# Recorded deviation from ADR-010 Decision 3 (ADR-028 amendment 1 and ADR-036
# Decision 10, founder decisions FA-54 and FA-77 2026-11-27, "x 3"): position
# sources (one interface; output-frame hashes, the scoped key hint, read-back
# adapters as data) get their own row for position/, capped at 900 = the
# 300-line estimate x 3. 0 lines at V7 (the path does not exist yet; the code
# lands with V8c). The review trigger fires when the row passes its 300 estimate.
position = total(["position/"])
rest = (
    sum(rust.values())
    - conductor
    - evidence
    - onset
    - presenters
    - importers
    - voxxad
    - recognizer
    - voxxa_eval
    - discovery
    - updater
    - wiring
    - input_chain
    - position
)

checks = [
    ("conductor (aligner/)", conductor, 1200),
    ("conductor evidence (ADR-009 minimum, aligner/evidence/)", evidence, 500),
    ("presenters/ total", presenters, 2000),
    ("importers/ + planning_center/", importers, 1600),
    ("everything else in src-tauri/src", rest, 904),
    ("wiring (dispatch.rs, listen.rs, http_api/, commands/)", wiring, 1438),
    ("discovery (src/discovery/)", discovery, 250),
    ("updater (src/updater/)", updater, 150),
    ("voxxad binary (src/cli/voxxad/)", voxxad, 600),
    ("recognizer (transcription/ + recognizer code + vad/)", recognizer, 825),
    ("voxxa-eval (src/cli/voxxa-eval/)", voxxa_eval, 600),
    ("onset (src-tauri/src/aligner/onset/)", onset, 750),
    ("input chain (src-tauri/src/input/)", input_chain, 600),
    ("position (src-tauri/src/position/)", position, 900),
    ("GUI (src/, index.html, stage.html)", sum(gui.values()) - simple - health, 3000),
    ("Simple mode (src/simple/*)", simple, 600),
    ("health check (src/health/)", health, 450),
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
