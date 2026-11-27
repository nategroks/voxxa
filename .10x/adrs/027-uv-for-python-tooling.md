# ADR-027: uv runs voxxa's Python tools; PEP 723 headers, no root project, no CI change; the dead Python prototype is deleted

**Status:** Proposed (2026-11-24) · **Date:** 2026-11-24 · **Feature:** church-autoadvance · **Reversibility:** Easy (comment headers; a deleted folder stays in git history) · **Owner:** nategroks · **Author:** 10x-Team synthesiser, from drafts 12, 14 and review 21

## Context
- The founder asked to "make this work with uv package manager" (brief § 1). The booth PC has Python 3.9 and no uv (brief § 3).
- Tracked Python on `origin/main` (`git ls-tree`): `scripts/loc_budget.py`, `scripts/qa/mock_pro7.py`, `scripts/release/{test_release_scripts,updater_manifest,winget_manifest}.py`, `src-tauri/tests/fixtures/pdf/make_pdfs.py`, all standard library only; plus the May prototype `main.py`, `voxxa/*.py` and `requirements.txt` (unpinned faster-whisper, rapidfuzz, pyautogui, numpy, sounddevice), which nothing runs (`.10x/decisions/cto/_index.md:8`; draft 12 § 1.1).
- CI runs `python3 scripts/loc_budget.py` and the release script tests with the runner's Python (`.github/workflows/ci.yml:149`, `:153`); release jobs hold `contents: write`.
- The private experiment tools (follower prototype, eval scripts, tap helpers) run on Python 3.9 without a lock and may contain private material **[UNVERIFIED: not read]** (review 20 S-43).
- The INTEGRATIONS foot-pedal and MIDI bridge examples import `requests` for one POST (draft 14 finding 15).

## Decision
1. **Booth:** the founder installs uv with `winget install --id astral-sh.uv -e`, then `uv python install 3.12`. System Python 3.9 stays untouched.
2. **Repo scripts:** each stdlib script gets a PEP 723 header (`requires-python = ">=3.12"`, `dependencies = []`); they still run with plain `python3`. `uv run scripts/loc_budget.py` is the documented way. **No root `pyproject.toml`, no `.python-version` at the root, no uv in CI.**
3. **Scripts with dependencies** get a PEP 723 header with pinned lower bounds (e.g. the INTEGRATIONS bridges: `hidapi`, or `mido` + `python-rtmidi`); `requests` is replaced by `urllib.request`.
4. **Delete** `main.py`, `voxxa/` and `requirements.txt` (337 lines, 5 packages) in one PR (founder item FA-09). `.gitignore` adds `.venv/` and `**/.venv/`.
5. **Private lab** (never public): `C:\dev\voxxa-church-private\lab\` on the founder's development booth (shared private tools, outside every `church_root`, ADR-029 § 2) becomes a uv project (`pyproject.toml` with no build system, `requires-python = ">=3.12,<3.14"`, `[tool.uv] exclude-newer = "7 days"` **[UNVERIFIED duration syntax in pyproject form]**, `uv.lock` kept in the private folder). Before any private script is edited by an agent or moved anywhere, the lyric 4-gram scan (ADR-029) runs on it; the prototype never moves to the public repo; its equivalence test against the Rust port stays private.
6. **Kit tools** (`kit/tools/songcheck.py`, `kit/tools/kit_check.py`) are stdlib PEP 723 scripts, run with `uv run` at churches and with `python3` in CI.
7. **Supply chain, separate S ticket:** pin `ci.yml` and `pages.yml` actions by SHA like `build.yml` (draft 12 § 1.5). Not part of this ADR's step.

## Alternatives Considered
| Alternative | Pros | Cons | Why Not |
|---|---|---|---|
| Root `pyproject.toml` + `uv.lock` (repo as a Python project) | one lock | the repo is Rust/Tauri; `uv run` would sync a project for stdlib scripts | nothing to lock |
| `astral-sh/setup-uv` in the CI budgets job (draft 12) | CI mirrors the booth | a new third-party action for zero dependencies | buys nothing today |
| Public `tools/voxxa-lab/` uv project with torch, Demucs, wav2vec2 (draft 10) | draft truth, faster tapping | 2–3 GB; accuracy on singing unproven; private-content leak risk (S-43) | deferred until tapping is the measured bottleneck |
| Keep the prototype "as reference" | history at hand | the only code that needs pip; a wrong "how to run voxxa" path | git history keeps it |

## Consequences
### Positive
- Every Python tool runs the same way on any church PC: `uv run <script>`.
- Lean: −337 lines and −5 unpinned dependencies.
### Negative
- Churches must install uv (one winget line in the kit).
### Risks
- `uv` versions change behaviour; the kit pins a minimum version in its README and the founder updates with `winget upgrade`.

## Dependencies
- Depends on: ADR-010 (the Rust port superseded the prototype).
- Constrains: ADR-026 and ADR-029 (kit tools).
- Plan steps: V1.

## Review trigger
A repo tool needs a third-party package in CI; the public draft-truth lab is approved; uv changes PEP 723 handling.
