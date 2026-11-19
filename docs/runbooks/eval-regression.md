# Runbook: the eval shows a regression

**Use this when** one of these is red or warns:

- CI, job "fmt, clippy, test", step **voxxa-eval (fixtures vs baseline)**;
- `cargo test --features cli`: `replay_of_the_committed_fixtures_passes_the_baseline`,
  `a_broken_*` (in `tests/voxxa_eval.rs`) or
  `every_committed_log_replays_to_its_own_actions_and_twice_the_same`
  (in `tests/replay_determinism.rs`);
- the **Full eval (report only)** workflow prints a warning.

The eval is `voxxa-eval` (ADR-015). Metric definitions are in `fixtures/README.md`,
"Metrics". Never paste recognised or song text into an issue or a PR. Quote
fixture names, metric names, numbers and slide indices only. Run full mode
with stderr filtered to `voxxa-eval:` lines, as `full-eval.yml` does
(`2> >(grep '^voxxa-eval' >&2)` in bash): Whisper prints the words it hears to
stderr, and `2>/dev/null` would also hide `voxxa-eval`'s own exit-2 messages,
which step 5 needs.

## 1. Read the failure line

| Output | Meaning | Go to |
|---|---|---|
| `FAIL <row>: <metric> is <value>, baseline needs <= <limit>` (or `>=`), exit 1 | A metric on that fixture (or `total`) is worse than `fixtures/baseline.json` | 2 |
| `voxxa-eval: …`, exit 2 | An error, not a score: a missing file, a bad line, `different setlist`, `voxxad failed`, `did not run in the mode asked for` | 5 |
| `<fixture>: replay printed N action lines, the log holds M; first difference at action i` | Replay no longer reproduces the actions recorded in that log | 3c |
| `<fixture>: two replays differ` | Replay is not deterministic. Something reads the clock, the environment or a random source | 4 (a bug: fix it) |
| Full eval warning: "a metric differs from fixtures/baseline.json" | Report only. Whisper on that OS heard something different | 2 |

How serious each metric is:

- `false_advances` or `double_advances` above 0, or a lower `precision`, means a wrong or repeated slide could reach the screen. **Treat it as a real regression** until you prove otherwise.
- A lower `recall` or a higher `missed` means a slide was never shown.
- `median_ms` and `p95_ms` are latency. Read `recall` and `missed` on the same row first (step 3b).
- `blank_ms` or `unblank_ms` (`miss` = never blanked, or never came back) means the lyrics stayed up during a prayer, or came back late.

## 2. Replay or full?

- **Replay** (CI's step, and every `cargo test` check) re-runs **today's conductor** on the recogniser lines recorded in `fixtures/recordings/<name>/session.jsonl`. It loads no model and no audio. Only these can turn replay red: the conductor (`src-tauri/src/aligner/`, including `evidence/`), the shared loop's decisions (`src-tauri/src/listen.rs`), the scorer (`src-tauri/src/cli/voxxa-eval/score.rs`), or a fixture's `truth.jsonl`, `setlist.json`, log, or `baseline.json`. A recogniser change cannot turn replay red.
- **Full** (`--full`) runs Whisper Tiny on the audio. A difference that shows only in full mode comes from the recogniser (`src-tauri/src/transcription/`: window, hop, stitcher), the model, `whisper-rs`, or the OS (the full-eval workflow may differ between Linux, macOS and Windows). Full mode is report-only in CI. The Windows reference PC is the one whose full table must equal replay.

Reproduce from the repository root:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval                         # replay
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full 2> >(grep '^voxxa-eval' >&2)   # full
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- fixtures/recordings/<name>   # one fixture
```

Running a subset changes the `total` row, so judge only the fixture rows then.

## 3. Real regression, or a log to re-record?

**a. Find the change.** Run replay on the failing head and on `main`. If `main` passes, list what changed: `git diff main --stat -- src-tauri/src fixtures`. To bisect, use a script that maps exit 0 to good, 1 to bad and 2 to skip (exit 125).

**b. Did recall rise?** A per-fixture latency threshold reads a recall gain as a regression. A slide that is now hit for the first time adds a latency sample that did not exist before. In Q6 the `jump-be-thou-my-vision` row went from 1 of 4 slides at 900 ms to 4 of 4 at a 5,850 ms p95. If `recall` rose or `missed` fell on the row, and only `median_ms` or `p95_ms` failed, the change is probably an improvement. It still needs the owner's approval to loosen a cell (step 4).

**c. Does a log need re-recording?** Re-record when the log no longer matches what it is meant to replay:

- a recogniser change (window, hop, stitcher, model, `whisper-rs`): replay still uses the old recogniser lines;
- a setlist edit (for example adding `sections`): the log's setlist fingerprint changes, and replay stops with `different setlist`;
- a change to a `SmartConfig` default. Replay builds the conductor from the config stored in each log's start line, so the change is invisible to replay until the logs are re-recorded (Q5-07, #78). Since Q7 all six fixture logs store the whole config, `evidence` block included;
- a conductor change that changes actions on purpose: `replay_determinism` requires each log's own action lines to equal replay's.

How to re-record:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full --record fixtures/recordings/<name> 2> >(grep '^voxxa-eval' >&2)
```

Then check that only the start line and the action and blank-status lines changed: the input lines (`hypothesis`, `window`, `speech`, `tick`) must be identical, unless the recogniser changed. Compare counts and positions only, never the text. Say why in the commit message. Run replay and `cargo test --features cli` again. **A re-recording cannot make a conductor regression go away.** After a re-recording for a non-recogniser reason, replay's table must equal the table from before it.

**d. Otherwise it is a real regression:** the conductor does worse on the same inputs.

## 4. Fix, revert, or change the baseline

- **Fix forward**, with a test that fails without the fix. The scripted conductor tests are in `src-tauri/tests/conductor_*.rs`. They use placeholder words, never lyrics.
- **Or revert** the change (`git revert <sha>`). `main` stays green.
- **Changing `fixtures/baseline.json`**, using ADR-015's rule: a latency threshold is the next multiple of 500 ms strictly above the measured worst, and counts and ratios are exact.
  - **Tighten** a cell in the same commit as the change that improves it.
  - **Loosening needs the owner's approval, every time.** The rule is "tightened, never loosened". The only exception so far is the one-time jump row on 2026-11-18. Open an issue with the before and after rows (recall, missed, latency) and the reason, and wait for the owner. Never loosen a cell to make an unexplained FAIL pass.
  - **Adding a fixture:** add its row, then set `total` from the new set's numbers by the same rule. Name each changed `total` cell in the commit message. The rows of the existing fixtures are never loosened.
  - Never edit `truth.jsonl` to match Voxxa, never delete a fixture, and never remove a threshold (a missing threshold fails the gate anyway).

## 5. Errors (exit 2) and the full-eval workflow

- `different setlist`: the log was recorded with another setlist. Re-record it (3c).
- `voxxad failed: …`: run the printed `voxxad` command by hand. `voxxa-eval` runs the `voxxad` binary that sits beside it, so build both with `--features cli`.
- `voxxad did not run in the mode asked for`: replay printed a start line, or full mode did not. Check that the right binaries were built.
- Full eval workflow: **checksum error.** The file at the model URL is no longer the pinned Whisper Tiny. Do not just update `MODEL_SHA256`. First confirm where the new file comes from (the Hugging Face LFS pointer of `ggerganov/whisper.cpp`), then change the hash in its own PR, with a full eval on all three OSes. The app downloads the same URL and does not check a hash (Q2-27, #9).
- Full eval workflow: the build, download or run fails (not exit 1). Fix it like any CI failure; it is not an eval result.

## 6. Close out

Put the table before and after in the PR, with the decision you took: fix, revert, re-record or approved baseline change. When the decision is deferred, open an issue with the fixture, metric, numbers and commit, and no transcript text.
