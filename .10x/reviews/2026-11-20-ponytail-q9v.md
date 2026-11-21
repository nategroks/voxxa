# Ponytail lane — Q9V (2026-11-20)

Independent pass, as the user asked (process rule, 2026-11-11). Q9V is a code step (Phase 4 + 5: the private "See a Victory" fixture and the media guard; ADR-019 §§ 7-9, ADR-015). It ran like this:
- **Two build lanes** started from 84a3966 (`main` after the Q9W merge, PR #144) in worktrees that could not see each other's work: the **10x lane** (`q9v/10x`, `C:/dev/q9v-10x`) and the **ponytail lane** (`q9v/ponytail`, `C:/dev/q9v-pony`).
- **Cross-review** of each lane, then a **comparator**, then the **finisher** on `q9v/base` (`C:/dev/q9v-base`), then **two verifier rounds**.

**Outcome:** base = **10x lane**, merged as c847eab, with four grafts from the ponytail lane, two fixes from the 10x review and one README sentence the finisher's own run showed was needed. Verifier round 1 found test gaps in the guard and the eval test; the finisher closed them with three test-only commits. **Round 2 at be4819b: both verifiers passed.** Every Q9V exit criterion is met on `q9v/base`. `q9v/10x` (c5ffb23) and `q9v/ponytail` (ce65275) stay local for reference. Nothing is pushed yet, and CI has not run Q9V.

No lyrics and no recognised words are quoted here. Tests and scratch fixtures use placeholder words ("alpha bravo") and titles ("Song A"). No agent searched for, opened, copied, hashed or converted the .mp4 or any church media, read anything under `%APPDATA%\Presenter`, touched Presenter, opened a microphone or started the GUI app. Nothing went to a network service or Google Drive.

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
| Guard test | `media_guard.rs`: pure `refused()`, three tests (real tree, which also fails when an allowed WAV is no longer tracked; fake list; ci.yml in step) | `media_guard.rs`: pure `refused()`, two tests (real tree, which checks it ran in the right repo; fake list) |
| CI job | `media`, its own job, bash: `declare -A` allowlist of the 15 exact paths, `read -d ''` over `git ls-files -z`, `${path,,}`; fails when git lists fewer than 100 paths | "no media in the repo", its own job: `git ls-files -z \| grep -zaiE …`, allowlist grepped out of `media_guard.rs`; fails closed if the test file is deleted |
| `fixtures/private/` | refused at the root and nested | refused at the root only, like the root `.gitignore` |
| `--full` | `voxxad(args, cwd, full)`; a fake `voxxad` test binary (`eval_numbers_only.rs`, `harness = false`), 5 checks with a replay control | `voxxad(args, full_dir)`; a fake `voxxad` built with `rustc` at test time, in `voxxa_eval.rs` |
| Replay error text | truth, setlist and bad `voxxad` lines by number in both modes | bad `voxxad` lines by number in both modes; setlist by line in `--full` only; truth lines unchanged |
| Truth route | `fixtures/tools/tap-truth.ps1` (first Enter is slide 0, a number, `b`, `n`, `q`; refuses to overwrite) or by hand | an inline PowerShell snippet (Enter, `b`, `q`) or by hand |
| Docs | README "Making the See a Victory fixture", 9 steps; runbook unchanged | README 6 steps; runbook paragraph updated |
| Budget (voxxa-eval) | 469 → 488 / 600 | 469 → 479 / 600 |

Both lanes passed every gate on their own head (240 passed, 0 failed, 1 ignored with `--features cli`; the CI eval step 21/21, "OK: no metric is worse"; `test:gui` 9/9; e2e 188/188), and both ran the planted-file check in a throwaway clone (§ 7). Neither lane wrote `.10x/` state files beyond its own decisions records (the 10x lane) or none at all (ponytail), as each was told.

## 3. Cross-reviews

**Review of the 10x lane: fix 1 first; no bug, leak or safety problem.** The reviewer re-ran the three test files (3/3, "5 checks passed", 26/26), ran the CI step on the real tree (exit 0), and planted `Músicas/canción.M4A`, `a/fixtures/private/x.json` and `fixtures/Private/n.txt` in its own scratch clone: the step named all three and exited 1.
- Should-fix 1: README step 4 quoted serde's "expected value at line 1 column 1" for a setlist with a BOM, but the new `main.rs:77-84` prints "not a setlist (Syntax error at line 1, column 1)". **Applied** (1931c59).
- Should-fix 2: the lane proposed M17 in its report and left it for the recorder. **Applied** (a4bd409): M17 is in `handoff.md` after M16.
- Nit 3: one allowlist file read by both the test and the CI job (about −40 lines). **Not applied**, by ruling (§ 5, disagreement 1).

**Review of the ponytail lane: approve, two should-fix items and three nits.** The reviewer re-ran the tests (2/2 and 27/27), the CI step (exit 0), and the README setlist snippet in Windows PowerShell 5.1 (no BOM, 4 slides).
- Should-fix 1: the CI allowlist is every lower-case `…/voxxad/*.wav` string anywhere in `media_guard.rs`, so a later test string could quietly allow a real file of that name in CI. The 10x base does not have this design.
- Should-fix 2: in `--full` every `voxxad` setup failure reads only "exit code: 1"; option (b) is a README line. **Grafted** into the 10x README (graft 2).
- Nits: `|| true` also swallows grep's exit 2 (fail-open); the runbook and release note still describe Whisper's log on stderr (**grafted**, graft 1); small gaps in the founder steps (the 10x README already covers the new window after installing ffmpeg and a way to recover from a mis-tap).

## 4. The comparator's ruling and the finisher's commits
**Base: the 10x lane**, because it is stronger on safety, the first criterion, and its founder docs are better. Ponytail is simpler (283 against 616 lines), but each saving costs some safety:
- the 10x CI job is self-contained (its own 15 exact paths, no `|| true`, fails on a short file list) and does not depend on the test file; ponytail's reads its list out of the test file (its review's should-fix 1) and can fail open on a grep error;
- the 10x test fails when an allowed WAV is no longer tracked, refuses nested `fixtures/private/`, and covers more look-alike cases;
- `eval_numbers_only` has a replay control run that shows the words do appear when not hidden, and needs no `rustc` on PATH at test time;
- the 10x README has a setup check on a public fixture (step 6, the diagnostic for "exit code: 1"), the pre-commit guard (step 9), and a reusable `tap-truth.ps1`; the ADR amendment was already written.

Costs of the 10x base: the 15 paths are written twice (a sync test makes drift a red build); replay error text changes for truth, setlist and bad `voxxad` lines; a `[[test]]` entry in `Cargo.toml` (no dependency).

One commit each on `q9v/base`:

| Commit | What | From |
|---|---|---|
| c847eab | merge `q9v/10x` (comparator's base) | 10x |
| 2b365b8 | `docs/runbooks/eval-regression.md`: full mode prints numbers only, do not add `2>/dev/null`; `docs/releases/voxxad-cli-preview.md`: `--full` now discards Whisper's log, so the stderr filter is redundant | ponytail graft 1 |
| ddb96b8 | README step 7: if step 6 worked but this run stops with "voxxad failed with exit code: 1", `audio.wav` is missing or not 16 kHz mono 16-bit, so redo step 3 | ponytail graft 2 (its review's 2(b)) |
| 4b53c67 | ci.yml `media` job on `ubuntu-latest` (bash 4 or later only); ADR-019 Q9V amendment item 2 to match | ponytail graft 3 |
| e17fe19 | `fixtures/privateer.md` and `docs/wave.md` in the guard test's fine list | ponytail graft 4 |
| 1931c59 | README step 4 quotes the BOM error as printed | 10x review should-fix 1 |
| 325a467 | README step 6: a `FAIL total` line is normal when one fixture runs | finisher (seen in its own real run) |
| a4bd409 | M17 in `handoff.md`; `status.md`; this record's first version | 10x review should-fix 2 |
| 3980585, efa5b7d, 6afd02f | verifier round 1 fixes, tests only (§ 6) | verifiers |
| be4819b | `status.md`: the fixes, gates and exit demo at 6afd02f | finisher |

**Checks on the grafts.**
- Graft 2: `voxxad` exits 1 on any error (`cli/voxxad/main.rs:53-56`), and `read_wav` refuses anything but 16 kHz mono 16-bit PCM or 32-bit float. On scratch fixtures with placeholder words, an 8 kHz dummy WAV and a missing `audio.wav` each printed only `voxxa-eval: voxxad failed with exit code: 1`, exit 2.
- Graft 4: two mutants applied by hand and reverted. The extension matched as a substring fails the fake-list test on `docs/wave.md`; `fixtures/private` without its slash fails it on `fixtures/privateer.md`.
- Should-fix 1: `voxxa-eval`, in replay and `--full`, on a scratch setlist written with a BOM: both printed `...setlist.json: not a setlist (Syntax error at line 1, column 1)`, exit 2.
- No graft was wrong, so none was skipped.

## 5. Disagreements
1. **One allowlist or two.** Ponytail and the 10x reviewer's nit prefer one list that both checks read. Kept: two copies plus the sync test (`ci_runs_the_same_check_with_the_same_list`). When the CI job reads its list from the test file, editing that file changes what CI allows (ponytail's should-fix 1). Two copies cost one extra edit per new public-domain recording, which is rare.
2. **Error text in replay.** 10x makes truth, setlist and bad `voxxad` lines number-only in replay too; ponytail keeps serde's setlist message in replay and leaves truth lines quoted. The dropped text is real debugging detail for public fixtures, but a truth line can hold a song title and a setlist value can be lyrics, so number-only everywhere is the safer default. The replay success path and the CI eval output are unchanged either way. Replay still quotes `voxxad`'s error lines and shows its stderr (pinned since 6afd02f).
3. **Tap tool: script or inline snippet.** Kept the script (`fixtures/tools/tap-truth.ps1`): it refuses to overwrite, takes a slide number to recover from a mis-tap, stops the player in a `finally` block, and matches `record-session.ps1`. A volunteer runs one command instead of pasting 12 lines.
4. **Nested `fixtures/private/`.** Ponytail matches only at the root, like the root `.gitignore`. Kept the 10x version, which also refuses nested paths: being stricter costs nothing.

## 6. Verifier rounds
**Round 1 (a4bd409): did not pass.** The verifiers found test gaps, not product defects. By their labels:
- **R2:** the real-tree test could not prove the `-z` parsing (the tree has no accented path), so reading without `-z` survived.
- **C1-C6:** no test ran the ci.yml step's own logic, so mutants of the bash step survived (no lower-casing, broken extension patterns, a weaker private arm, a wider allowlist, a neutered exit, `continue-on-error`).
- **E4c, E4d:** nothing pinned that replay still quotes `voxxad`'s error line and passes its stderr through (the fake printed a start line in replay too, so the replay path could not run against it).

**Fixes, tests only** (no product code, `ci.yml`, `voxxa-eval` or `voxxad` change):
- 3980585: `tracked(root)` takes a repository root; `tracked_paths_are_read_as_written_even_with_accents` builds a temporary `git init` repo with `café/x.txt`, `Músicas/canción.wav` and `plain/y.txt` and needs them back exactly as written, the `.wav` refused. `the_ci_step_passes_the_real_tree_and_names_every_planted_file` takes the `media` job's `run: |` block out of ci.yml and runs it with a bash 4 or later found on PATH (skipping WSL's `System32\bash.exe` and `WindowsApps`; it prints SKIPPED and returns only when there is no bash 4): exit 0 on the real tree; exit 1 on a temporary repo of 100 placeholders, 2 look-alikes, the 15 allowed paths and 17 planted paths, each named in its own `::error::` line and nothing else. The planted paths cover an accented folder, `Song A.MP4`, every other extension in mixed case, `fixtures/private/x.json`, `Fixtures/Private/notes.txt`, two nested private paths, `prayer.wav` at the root, `voxxad/new.wav`, `voxxad/sub/prayer.wav` and `voxxad/PRAYER.WAV`. It also asserts that the job has no `continue-on-error`.
- efa5b7d: the temporary repo stages empty-blob index entries (`git update-index --index-info`) instead of files, so no media-named file is written to disk and `PRAYER.WAV` can sit beside `prayer.wav` on a case-insensitive file system.
- 6afd02f: the fake `voxxad` prints its start line only when not replaying, like the real one; a sixth check runs replay in error-line mode and needs exit 1, "mode: replay", the whole quoted error line and the fake's stderr line.
- The finisher's mutation run killed R2 (both forms), C1b, C3b-C3e, C4b, C4c, C5a-C5d, C6a, C6c, C6d, C7, E4c and E4d. One listed form, `|| true` after the read loop, changes nothing (the script has no `set -e`; the exit decision is the explicit `exit 1`), so it is not a mutant.

**Round 2 (be4819b): both passed.**
- **Verifier A** (exit criteria; detached worktree, now removed): every gate. It planted six paths one at a time (`x.MP4`, `Song Ä/x.wav`, `docs/prayer.wav`, `fixtures/private/see-a-victory/setlist.json`, `y.m4a`, `z.webm`; dummy bytes, `git add -f`): each time both the Rust test (exit 101) and the ci.yml step (exit 1) named the path; each plant was removed with `git rm --cached` and `rm`, and the real tree passed again. It built its own fake `voxxad` with placeholder words and drove `voxxa-eval` through 7 run shapes, 12 malformed truth lines, a truth file with a BOM, a non-UTF-8 truth file and 10 malformed setlists: no placeholder word was printed in any of them (§ 8). It checked the ffmpeg command on a synthetic test-pattern mp4 (16 kHz, mono, 16-bit, a LIST chunk that `read_wav` skips), the setlist snippet in PowerShell 5.1 (no BOM, a 1-slide case still an array, accepted by both readers) and `tap-truth.ps1` with piped keys on a silent WAV (LF lines, no BOM, the format `parse_truth` reads). Non-blocking note: after `b`, Enter moves to the next slide and the README does not say so (Q9V-10).
- **Verifier B** (scope, safety, mutants; detached worktree, now removed): every gate; scope clean (no new crate, `Cargo.lock`, `package.json`, GUI, `tauri.conf.json` or `scripts/` change; the only `Cargo.toml` change is the `[[test]]` entry). Across all 444 commits in `git log --all`, the only media paths are the 15 allowed WAVs, and nothing was ever under `fixtures/private` (also checked case-insensitively). Every named mutant was killed on the Rust side, the CI step (including a hidden `${path,,}` removal and a commented-out private arm that keep the text), the eval and the default path. Two minor survivors, not blocking: an `if:` on the media job or its step (Q9V-03). Also noted: the Rust no-`-z` mutant is caught only while git's `core.quotepath` is true, the default (Q9V-04). It stopped two headless Edge processes that `test:gui:e2e` had left, by PID, and deleted their profile.

## 7. The planted-file evidence
Every plant was a dummy (zero bytes, a few zero bytes, `{}`, or an empty-blob index entry) in a throwaway clone, a detached verifier worktree or a test's temporary repo. None was committed on a `q9v/*` branch or pushed.

| Run | Where | Planted | CI step | Rust test |
|---|---|---|---|---|
| 10x lane | clone of `q9v/10x`, branch `scratch-plant-never-push`, deleted | `X.WAV`, `Músicas de adoración/canción.mp4`, `fixtures/private/see-a-victory/dummy.txt` | three `::error::` lines, exit 1 | `no_tracked_path_is_media_or_under_fixtures_private` failed naming all three |
| ponytail lane | `git clone --shared` of `q9v/ponytail`, deleted | `media/X.WAV`, `fixtures/église/Song A.Mp4`, `fixtures/private/see-a-victory/setlist.json`, `other/amazing-grace-v1.wav` | exit 1, all four named, accent intact | failed naming all four; with the test file deleted, the step named the 15 and failed |
| 10x reviewer | scratch clone, deleted | `Músicas/canción.M4A`, `a/fixtures/private/x.json`, `fixtures/Private/n.txt` | exit 1, all three named | not run |
| finisher, after the grafts | clone of `q9v/base`, no remote, deleted | the 10x lane's three | three `::error::` lines, exit 1 | failed naming all three |
| finisher, at 6afd02f | `git clone --no-hardlinks`, deleted | `Músicas/canción.wav`, `clips/Song A.MP4`, `fixtures/private/see-a-victory/setlist.json` | three `::error::` lines, exit 1 | the real-tree test and the CI-step test both failed naming the three; the other three tests passed |
| verifier A | its worktree, one at a time, each removed | six paths (§ 6) | exit 1 each | exit 101 each |
| verifier B | scratch repo, deleted | `Músicas/Canción.WAV`, `a b/Song A.mp4`, `x.FLAC`, `fixtures/private/see-a-victory/setlist.json`, a `prayer.wav` in another folder, plus the allowed `prayer.wav` | exit 1, exactly the 5 bad paths named | (the CI-step mutants it ran prove the bash test ran) |
| the suite itself | the CI-step test's temporary repo | 17 paths (§ 6) | exit 1, each named, nothing else | — |

A plain `git ls-files` in the finisher's clone printed the accented path quoted (`"M\303\272sicas/canci\303\263n.wav"`), which is why both checks read `-z`. **`git log --all -- fixtures/private` prints nothing** in every worktree, before and after, and `git ls-files fixtures/private` lists nothing.

## 8. The `--full` evidence
**The code at be4819b.** In `--full`, `voxxad`'s stderr goes to `Stdio::null()` (`cli/voxxa-eval/main.rs:158`); a failure reads "voxxad failed with exit code: N" (`:167`); an error line reads "an error on its line N" (`:111`); a bad setlist reads "`<path>`: not a setlist (Data error at line L, column C)" (`:80`); a bad truth or `voxxad` line reads "truth line N" / "voxxad line N" (`score.rs:269,307`). The success output is the mode line, the table, the `FAIL` lines and `OK`.

**The test.** `src-tauri/tests/eval_numbers_only.rs` (`harness = false`): the test binary copies itself next to a copy of `voxxa-eval` as a fake `voxxad` that prints "alpha bravo" on stderr and in a hypothesis line. Six checks: a failing run, a bad line, an error line and a bad setlist in `--full` print none of the placeholder words and name the exit status, line number or line and column; a replay failure does show the words (so the check can see a leak); replay still quotes the whole error line and shows the fake's stderr. The 10x lane wrote it first and saw it fail before the fix, printing the fake's stderr and stdout.

**Real runs.**
- Finisher at 6afd02f: `voxxa-eval --full ../fixtures/recordings/prayer-then-reprise` with real Whisper Tiny (a public-domain fixture) printed only "mode: full (Whisper Tiny)", the table (3/3 hits) and `FAIL total: median_ms is 2900, baseline needs <= 2500` (one fixture against the six-fixture total, as README step 6 says). **stderr was 0 bytes.** Exit 1. No `voxxad` process was left running.
- Ponytail lane: real `voxxad` with Whisper Tiny on a 1 s silent placeholder fixture: the table and `FAIL see-a-victory: no thresholds in the baseline`, exit 1, no Whisper log.
- Verifier A, with its own fake `voxxad` (placeholder words on stderr and stdout): exit 3 gave "voxxad failed with exit code: 3"; a panic "exit code: 101"; a non-JSON line "voxxad line 4: expected value at line 1 column 1"; a bad action kind or a string `t` "voxxad line 4"; an error line "an error on its line 4" plus the table; a normal run the table and `FAIL` lines only; `--record` "voxxad wrote no session log". 12 malformed truth lines with words, a BOM truth file and a non-UTF-8 truth file gave only a line number plus serde's position, or the file path. 10 malformed setlists and a BOM setlist gave only "not a setlist (Data|Syntax error at line L, column C)". In replay, `voxxad`'s text still shows, as designed. The real `voxxad` with Whisper Tiny on a synthetic sine-tone WAV through `--full` printed the table only, no whisper or ggml log; a 44.1 kHz stereo WAV or a missing `audio.wav` gave "voxxad failed with exit code: 1", as README step 7 says.
- The default CI path is unchanged: the CI eval step gives 21/21 hits, precision and recall 1.00, median 2,310 ms, p95 4,870 ms, "OK: no metric is worse"; verifier A found its output identical to the same step at 84a3966.

## 9. Gates at the finish (finisher at 6afd02f; both verifiers at be4819b, which changes only `status.md`)
Windows 11, rustc 1.99.1, `CARGO_TARGET_DIR` per agent. All pass:
1. `npm ci && npm run build`
2. `cargo fmt --check`
3. `cargo clippy --all-targets -D warnings`, without and with `--features cli`
4. `cargo test --features cli`: 242 passed, 1 ignored; `media_guard` 5/5 (the CI-step test ran under Git bash 5.3, no SKIPPED); `eval_numbers_only` "6 checks passed"
5. cargo-deny: advisories, bans and licences ok
6. `python scripts/loc_budget.py`: all rows OK. voxxa-eval **488 / 600**, voxxad 552 / 600, everything else 2315 / 2317, discovery 223 / 250. No cap raised.
7. `git grep -i dictat -- . ':!.10x'`: empty

Plus: the CI eval step (above); the ci.yml media step as written, "media guard: 287 tracked paths, no media outside the 15 allowed recordings", exit 0; `test:gui` 9/9; e2e 188/188; the control-byte grep matches nothing.

## 10. What is open
- CI, push, PR and merge (pre-approved, D6). CI has not run the `media` job, `eval_numbers_only` or the CI-step test on Linux or macOS yet. On a macOS runner with bash 3.2 the CI-step test prints SKIPPED; the `media` job itself runs on Ubuntu.
- Tickets: `reviews/2026-11-20-q9v-tickets.md`, 10 open (1 medium, 9 low). Q9V fixes #105 (Q7-11); comments go to #74, #79 and #107.
- Founder: M17 on LHMEZPC01 (the .mp4 path is the founder's own), and the church's CCLI License # for `LICENSED_RUNS.md` row 1.

## 11. Lessons for the next step
- Re-run a doc's quoted error against the binary. Step 4's quote was right before the lane changed the error, and wrong after.
- A test of a CI step should run the step, not only read its text. The text-only sync test passed every mutant of the bash logic until 3980585 ran the `run: |` block itself.
- Plant case-only variants as index entries (`git update-index --index-info`): on Windows a `PRAYER.WAV` file cannot sit beside `prayer.wav`.
- Building a test from a scratch clone with a shared `CARGO_TARGET_DIR` rebuilds `voxxa-eval` with the clone's `CARGO_MANIFEST_DIR` baked in. Rebuild from the worktree before using `target/debug/voxxa-eval` again.
- Lanes share the top level of the scratchpad. The ponytail lane's `media_step.sh` was overwritten by another writer; each lane should work only under its own subfolder from the start.
