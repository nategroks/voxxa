# SRE — church-autoadvance

## 2026-11-19 — Q7: runbook "eval shows regression"; what the eval gates guard

### Decisions
- **Runbook:** `docs/runbooks/eval-regression.md`. Symptoms: the CI eval step, the `voxxa_eval` and `replay_determinism` tests, and the full-eval workflow's warning, each with the line it prints and what that metric means. Triage: replay against full, and how to find the change. Then decide: real regression or re-record, with recall and missed read first, then fix or revert. It says how to change `baseline.json`: tighten in the same commit, and loosening needs the owner's approval every time. It covers errors (exit 2) and the full-eval checksum failure, and how to close out without pasting transcript text.
- **Severity order** for an eval FAIL, highest first: a false or double advance, or lower precision (a wrong slide can reach the screen); then recall or missed (a slide never shown); then blank or unblank (lyrics stay up in a prayer); then latency. A latency FAIL on a row whose recall rose is read as an improvement that needs an approved threshold change (the Q6 lesson).
- **Gates, by what they catch:**
  - CI replay step + `replay_of_the_committed_fixtures_passes_the_baseline`: a conductor or scorer change made a fixture worse. It needs no model; it runs on 3 OSes.
  - `replay_determinism` (new in Q7): replay stopped reproducing a log's own actions (a conductor change, so the log must be re-recorded with a reason), or two replays differ (nondeterminism: a bug).
  - `replay_no_sockets` (new in Q7): replay opened a socket (Linux and Windows only).
  - Full-eval workflow (manual, report only): the recogniser or the OS changes what Whisper hears. It never blocks a merge.
- **Re-recording is part of operating the eval, not a fix.** After a re-record for a non-recogniser reason, the replay table must equal the table from before it. Q7 re-recorded four logs. The input lines were byte-identical and the table was unchanged (QA report §3).

### Operability notes
- The eval has no service to page: it runs in CI. Its "incident" is a red PR or a red `main`. The owner of a red `main` is whoever merged last: fix forward or revert the same day.
- Time budget: in debug on the reference PC, a replay of all six fixtures takes about 2.5 min. The determinism test replays every log twice in parallel (about 80 s). The full eval takes about 12 min on the reference PC; the workflow has a 180 min timeout per OS.
- **Model supply chain:** the app's model URL is a moving `main`. The workflow pins the SHA-256, and a mismatch fails the job; the runbook says not to just update the hash. The app itself still checks no hash (Q2-27, #9).

### Open
- The full-eval workflow's first real runs (3 OSes) happen on the Q7 PR. Until then its timing and the non-Windows tables are unknown.
- Ticket Q7-02: no notification or trend for the report-only full eval. Someone has to read the job summary.
