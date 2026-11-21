# Ponytail lane — Q9V (2026-11-20)

Independent pass, as the user asked (process rule, 2026-11-11). Q9V is a code step (Phase 4 + 5: the private "See a Victory" fixture and the media guard; ADR-019 §§ 7-9, ADR-015). It ran like this:
- **Two build lanes** started from 84a3966 (`main` after the Q9W merge, PR #144) in worktrees that could not see each other's work: the **10x lane** (`q9v/10x`, `C:/dev/q9v-10x`) and the **ponytail lane** (`q9v/ponytail`, `C:/dev/q9v-pony`).
- **Cross-review** of each lane, then a **comparator**, then the **finisher** on `q9v/base` (`C:/dev/q9v-base`). The verifier rounds come after this record.

**Outcome:** base = **10x lane**, merged as c847eab, with four grafts from the ponytail lane and two fixes from the 10x review, one commit each, plus one README sentence the finisher's own run showed was needed. Every Q9V exit criterion is met on `q9v/base` (§ 7). `q9v/10x` (c5ffb23) and `q9v/ponytail` (ce65275) stay local for reference. Nothing is pushed yet, and CI has not run Q9V.

No lyrics and no recognised words are quoted here. Tests and scratch fixtures use placeholder words ("alpha bravo") and titles ("Song A"). No agent searched for, opened, copied, hashed or converted the .mp4 or any church media, read anything under `%APPDATA%\Presenter`, touched Presenter, opened a microphone or started the GUI app.

## 1. The task, in short
Pack block Q9V (`specs/2026-11-11-prompts-10x-team.md:85-95`; handoff "From: Q9W → To: Q9V"):
1. **The repo refuses media:** a test reads `git ls-files -z`, lower-cases each path, and fails naming any path that ends in one of nine audio and video extensions or sits under `fixtures/private/`; exactly the 15 public-domain WAVs in `src-tauri/tests/fixtures/voxxad/` are allowed by name; the pure check is tested with a fake list; the same check runs in `ci.yml`.
2. **`voxxa-eval --full` prints numbers only:** `voxxad`'s stderr to null; errors name the exit code or the line number; a test proves a failing run prints no hypothesis text. Replay (CI) unchanged.
3. **Founder docs** in `fixtures/README.md` for `fixtures/private/see-a-victory/`: the ffmpeg command, a BOM-free PowerShell 5.1 setlist snippet, a way to make `truth.jsonl`.
4. **Founder checklist item** to run the eval and record one row in `fixtures/LICENSED_RUNS.md`, numbers only.

**R0's "M16" is M17.** R0's record (`reviews/2026-11-20-ponytail-r0.md:219`) calls the private-fixture eval "M16". Q9W took M15 and M16, so this check is **M17** (handoff, "Still open from earlier steps").

## 2. The two lanes

| | 10x lane (`q9v/10x` @ c5ffb23) | ponytail lane (`q9v/ponytail` @ ce65275) |
|---|---|---|
| Commits on 84a3966 | 5: b73a9a9 (guard and CI job), 7f71681 (numbers-only `--full`), 5a4bcea (founder steps, `tap-truth.ps1`), 2584f00 (ADR-019 amendment), c5ffb23 (SDE, QA, security records) | 3: 1783b88 (guard and CI job), b3a4e1e (numbers-only `--full`), ce65275 (founder steps) |
| Size | 14 files, +616 / −22 | 8 files, +283 / −23 |
| Guard | `media_guard.rs`: pure `refused()`, three tests (real tree, fake list, ci.yml in step); CI job `media` in bash with its own copy of the 15 paths | `media_guard.rs`; CI job reads the 15 paths out of the test file with `grep` |
| `--full` | `voxxad(args, cwd, full)`; a fake `voxxad` test binary (`eval_numbers_only.rs`, `harness = false`), 5 checks; the setlist error by line and column in both modes | the `full_dir` argument switches stderr to null; tests in `voxxa_eval.rs`; the setlist error by line in `--full` only |
| Truth route | `fixtures/tools/tap-truth.ps1` (first Enter is slide 0, `b`, `n`, numbers) or by hand | an inline PowerShell snippet (Enter, `b`, `q`) or by hand |
| Docs | README "Making the See a Victory fixture", 9 steps; runbook unchanged | README 6 steps; runbook paragraph updated |
| Budget | voxxa-eval 469 → 488 / 600 | within the voxxa-eval row |

## 3. Cross-reviews, as the comparator used them
**Review of the 10x lane.**
- Should-fix 1: step 4 quoted serde's "expected value at line 1 column 1" for a setlist with a BOM, but the new `main.rs:77-84` prints "not a setlist (Syntax error at line 1, column 1)". **Applied** (1931c59).
- Should-fix 2: the lane proposed M17 in its report and left it for the recorder. **Applied** in this commit: M17 is in `handoff.md` after M16, with four sub-steps (make the fixture, check the setup, run the eval, record the row). The lane's report text was not in the worktree; the finisher wrote the four sub-steps from the lane's README steps 1-9, which they follow.
- Nit 3: one allowlist file read by both the test and the CI job. **Not applied** (comparator): the test `ci_runs_the_same_check_with_the_same_list` already turns drift into a red build, and a CI job that depends on no test file is the stronger kind of independence for § 8 (it still runs if the test file is broken or deleted).

**Review of the ponytail lane.** The comparator took should-fix 2(b) as a graft into the 10x docs (§ 4, graft 2).

## 4. The comparator's ruling and the finisher's commits
Base: the 10x lane (pure check tested with a richer fake list, CI job independent of the test file, a fake-`voxxad` proof of numbers only, a reusable truth tool, the ADR amendment first). One commit each on `q9v/base`:

| Commit | What | From |
|---|---|---|
| c847eab | merge `q9v/10x` (comparator's base) | 10x |
| 2b365b8 | `docs/runbooks/eval-regression.md`: full mode prints numbers only, do not add `2>/dev/null`; `docs/releases/voxxad-cli-preview.md`: `--full` now discards Whisper's log, so the stderr filter is redundant | ponytail graft 1 |
| ddb96b8 | README step 7: if step 6 worked but this run stops with "voxxad failed with exit code: 1", `audio.wav` is missing or not 16 kHz mono 16-bit, so redo step 3 | ponytail graft 2 (its review's 2(b)) |
| 4b53c67 | ci.yml `media` job on `ubuntu-latest` (bash 4 or later only; the check job's AppArmor reason for 22.04 does not apply); ADR-019 Q9V amendment item 2 to match | ponytail graft 3 |
| e17fe19 | `fixtures/privateer.md` and `docs/wave.md` in the guard test's fine list | ponytail graft 4 |
| 1931c59 | README step 4 quotes the BOM error as printed | 10x review should-fix 1 |
| 325a467 | README step 6: a `FAIL total` line is normal when one fixture runs | finisher (seen in § 7's real run) |
| this commit | M17 in `handoff.md`; `status.md`; this record | 10x review should-fix 2 |

**Checks on the grafts.**
- Graft 2: `voxxad` exits 1 on any error (`cli/voxxad/main.rs:53-56`), and `read_wav` refuses anything but 16 kHz mono 16-bit PCM or 32-bit float. Run on scratch fixtures with placeholder words: an 8 kHz dummy WAV and a missing `audio.wav` each printed only `voxxa-eval: voxxad failed with exit code: 1`, exit 2.
- Graft 4: two mutants were applied by hand and reverted. Extension matched as a substring (`lower.contains("wav")`): the fake-list test fails on `docs/wave.md` (and `docs/audio.wav.md`). `fixtures/private` without its slash, with `fixtures/private-notes.md` taken out: it fails on `fixtures/privateer.md`.
- Should-fix 1: `voxxa-eval` was run, in replay and `--full`, on a scratch setlist written with a BOM. Both printed `...setlist.json: not a setlist (Syntax error at line 1, column 1)`, exit 2.
- None of the grafts was wrong, so none was skipped. Nit 3 was not applied, by ruling.

## 5. Docs and ADR checked against the final code
- ADR-019 §§ 7-9 and the "Amendment (Q9V)": item 1 (lower-cased matching, exact-path allowlist, the three tests), item 2 (`ubuntu-latest`, own job), item 3 (`--full`: stderr null, exit status only, error lines by number; bad lines by number and the setlist by line and column in both modes), item 4 (BOM-free snippet, `tap-truth.ps1`, exit 1 as a report), item 5 (voxxa-eval 488 / 600) all match the code at this head.
- `fixtures/README.md`: the "Full" bullet, § Private fixtures and steps 1-9 match. Steps 4, 6 and 7 now quote what the binaries print.
- `full-eval.yml` and the runbook's command lines keep the `2> >(grep '^voxxa-eval' >&2)` filter. It is redundant now, and harmless.

## 6. Gates (finisher, Windows 11, rustc 1.99.1, `CARGO_TARGET_DIR=C:/dev/q3-10x-target`)
All pass at 325a467 (this records-only commit changes no code):
1. `npm ci && npm run build`
2. `cargo fmt --check`
3. `cargo clippy --all-targets -D warnings`, without and with `--features cli`
4. `cargo test --features cli -j 2`: 240 passed, 0 failed, 1 ignored, plus `eval_numbers_only` 5 checks (`media_guard` 3/3)
5. cargo-deny: advisories, bans and licences ok
6. `python scripts/loc_budget.py`: all rows OK (voxxa-eval 488 / 600, everything else 2315 / 2317, voxxad 552 / 600). No cap raised.
7. `git grep -i dictat -- . ':!.10x'`: empty.

Then the CI eval step as ci.yml runs it: 21/21 hits, precision and recall 1.00, median 2,310 ms, p95 4,870 ms, "OK: no metric is worse". The ci.yml media step, taken from the file by a script and run with `bash --noprofile --norc -eo pipefail` (bash 5.3): "media guard: 286 tracked paths, no media outside the 15 allowed recordings", exit 0. `npm run test:gui` 9/9; `npm run test:gui:e2e` 188/188. The control-byte grep (`git grep -lIP '[\x00\x0b]'`) matches nothing. README step 9's command, `cargo test --manifest-path src-tauri/Cargo.toml --test media_guard` (without `--features cli`), passes 3/3. No process started by the finisher is still running.

## 7. Exit criteria (re-taken by the finisher)
- **The guard passes on the real tree:** `media_guard` 3/3 and the ci.yml step, exit 0 (§ 6).
- **It catches a planted file in a scratch run:** a throwaway `git clone` of `q9v/base` in the scratchpad, with no remote and on branch `scratch-plant-never-push`. Three dummies were staged with `git add -f`: `X.WAV` and `Músicas de adoración/canción.mp4` (four zero bytes each) and an empty `fixtures/private/see-a-victory/dummy.txt`. The ci.yml step (byte-identical to the base's) printed one `::error::` line naming each path and exited 1. `cargo test --test media_guard`, built from the clone, failed `no_tracked_path_is_media_or_under_fixtures_private` with `["Músicas de adoración/canción.mp4", "X.WAV", "fixtures/private/see-a-victory/dummy.txt"]`. Then `git rm --cached` was run and the clone was deleted. Nothing was committed or pushed.
- **`--full` prints numbers only:** `eval_numbers_only` (a fake `voxxad` printing "alpha bravo" on stderr and stdout) passes 5/5. A real `--full` run with Whisper Tiny on the public `prayer-then-reprise` printed the mode line, the table and `FAIL total: median_ms is 2900, baseline needs <= 2500` (one fixture against the six-fixture total), exit 1, and **nothing on stderr** (0 lines). The scratch failure runs in § 4 printed only the exit status.
- **`git log --all -- fixtures/private`** prints nothing, and `git ls-files fixtures/private` lists nothing.
- **The docs give the founder every command:** `fixtures/README.md` steps 1-9 (ffmpeg, the folder, the setlist snippet, `tap-truth.ps1` or by hand, the setup check, the eval, the row, the pre-commit guard) and M17 in the handoff.

## 8. What is open
- The verifier rounds, then the push, the draft PR, CI and the merge (pre-approved, D6). CI has not run the `media` job yet.
- Proposed tickets (10x SDE and security records): the guard matches names, not content (renamed audio, other extensions); lyrics copied out of `fixtures/private/` are not caught; make "media guard" a required check on `main`; a killed `--full` run leaves its temporary session log; nothing refuses `--record` on a private fixture (docs only).
- Founder: M17 on LHMEZPC01, and the church's CCLI License # in `LICENSED_RUNS.md`.

## 9. Lessons for the next step
- Re-run a doc's quoted error against the binary. Step 4's quote was right before the lane changed the error, and wrong after.
- Building a test from a scratch clone with a shared `CARGO_TARGET_DIR` rebuilds `voxxa-eval` with the clone's `CARGO_MANIFEST_DIR` baked in. Rebuild from the worktree before using `target/debug/voxxa-eval` again.
