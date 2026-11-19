# Runbook: the eval shows a regression

For whoever finds a red eval check on a pull request, or a warning in the
full-eval workflow. The eval scores what `voxxad` would have done on the
committed fixtures against what an operator did (`fixtures/README.md` has
every metric's definition).

## 1. Symptoms

| Where | What you see | Meaning |
|---|---|---|
| `ci.yml`, step `voxxa-eval (fixtures vs baseline)` | `FAIL <row>: <metric> is <value>, baseline needs <= <limit>`, exit 1 | a metric is worse than `fixtures/baseline.json` (replay mode) |
| `ci.yml`, step `cargo test` | `replay_of_the_committed_fixtures_passes_the_baseline` fails | the same gate, run as a test |
| same step | `replay_determinism`: `<fixture>: replay's actions differ from the log's; re-record it` | the log's `action` lines came from another conductor |
| same step | `replay_determinism`: `<fixture>: two replays differ` | replay is not deterministic: a real bug |
| eval step, exit 2 | `voxxa-eval: <fixture>: ...` | an error, not a score: a missing file, a bad line, `this session was recorded with a different setlist` |
| `full-eval.yml` (report only) | warning `a metric is worse than fixtures/baseline.json` | Whisper on this OS did worse than the baseline; the job stays green |
| `full-eval.yml` red | build, download, `SHA-256` or exit 2 | the run broke, see section 5 |

The metrics, in the order to read them: `recall` and `missed` (slides never
shown), `false/set` and `double` (wrong or useless changes), then the
timings `median ms`, `p95 ms`, `blank ms`, `unblank ms`.

## 2. Reproduce locally

From the repository root, after `npm ci && npm run build`:

```sh
cargo build --manifest-path src-tauri/Cargo.toml --features cli --bin voxxad --bin voxxa-eval
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval              # replay, as CI
cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full 2>/dev/null   # Whisper Tiny
```

Replay of all fixtures takes a few minutes in a debug build; `--full` three to
four times as long and needs `ggml-tiny.bin` in the app's model folder.
Keep `--full`'s stderr out of logs and messages: Whisper prints the words it
recognised there.

## 3. Triage

1. **Replay or full?** Replay (CI) re-runs today's conductor on recorded
   recogniser output, so a red replay means the conductor, the scorer, a
   fixture or the baseline changed in this PR. A full-eval warning can be
   Whisper behaving differently on that OS; compare with replay and with the
   other two OSes before acting.
2. **Latency failed? Read recall and missed first.** A change that shows a
   slide it used to miss adds a new, often late, hit and can raise `p95` or
   `median` without any old hit getting slower (Q6's jump fixture). Compare
   hit by hit: `voxxad --replay <fixture>/session.jsonl --setlist <fixture>/setlist.json`
   on the base commit and on the PR, `action` lines only, against
   `truth.jsonl`. Only a hit that got slower, a new miss, or a new false or
   double advance is a regression.
3. **What did the PR change?**
   - Conductor code (`src-tauri/src/aligner/`, `listen.rs`): treat a FAIL as
     a real regression until a hit-by-hit comparison says otherwise.
   - A `SmartConfig` or `EvidenceConfig` default: replay does not see it.
     Each log stores the config it was recorded with (since Q7 all six store
     the `evidence` block), so the default reaches the eval only after the
     logs are re-recorded (Q5-07, #78). Re-record, then read the table.
   - The recogniser (window, hop, stitcher, model), or a fixture's
     `setlist.json` (adding `sections` changes its fingerprint): re-record
     the affected logs; exit 2 `different setlist` means exactly this.
   - Only `voxxa-eval`'s scorer: the scores moved, not the behaviour; the
     scorer's unit tests in `src-tauri/tests/voxxa_eval.rs` must explain it.
4. **Determinism failed?** `two replays differ` is a bug (hash order, the
   wall clock, threads): fix it, never re-record to hide it. `replay's
   actions differ from the log's` after an intended conductor change means
   the log must be re-recorded; without such a change it is a regression.

## 4. Fix, re-record or revert

- **Real regression:** fix forward with a test that fails without the fix,
  or `git revert` the commit. Do not merge with the eval red.
- **Re-record** (intended conductor, config or recogniser change):

  ```sh
  cargo run --manifest-path src-tauri/Cargo.toml --features cli --bin voxxa-eval -- --full --record fixtures/recordings/<name> 2>/dev/null
  ```

  Then check that the `hypothesis` and `window` lines did not change (same
  model, same audio: Whisper Tiny reproduced them byte for byte on the
  reference PC in Q6 and Q7) and that only the `start` line, `status` and
  `action` lines moved. Say in the commit message which change made the
  re-record necessary. Re-recording runs Whisper on files in shadow mode;
  it never opens a microphone or drives a presenter.

## 5. The full-eval workflow is red

- **Checksum mismatch:** the file behind the app's download URL changed. Do
  not just paste the new hash. Find out what changed upstream, run
  `--full` locally with the new file, and decide with the owner whether the
  app keeps that URL (the app's own download has no checksum yet: Q2-27, #9).
- **Build or exit 2:** treat as a CI break: fix forward.
- Run it by hand from the Actions tab (`Full eval`, Run workflow) or with
  `gh workflow run full-eval.yml`; each OS uploads `full-eval-<os>`.

## 6. Changing `fixtures/baseline.json`

- **Tighten** in the same commit as the change that improves a metric, by
  the rule in `fixtures/README.md` (latency: the next multiple of 500 ms
  strictly above the measured worst; counts and ratios exact).
- **Loosen only with the owner's written approval** (in the PR or the
  issue), with the reason in the commit message, as with Q6's one-time
  exception for the jump fixture. Never loosen to make an unexplained
  regression pass, and never loosen in a PR that also changes the
  conductor without saying so in its description.
- **New fixture:** add its row, then set `total` from the new set by the
  same rule and name each changed `total` cell in the commit message.
