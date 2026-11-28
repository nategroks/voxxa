# V2 comparator record (2026-11-28, `v2/final` off 8df5893)

Numbers, names and paths only. The comparator read both lane worktrees (`C:\dev\v2-10x`, `C:\dev\v2-pony`) and the 10x lane's held
branch (`v2/10x-full-held` d2211b6, local, never pushed, never checked out: `git show` and `git diff` only) and modified none of them; it
built `C:\dev\v2-final` on branch `v2/final`, created off 8df5893 (origin/main, PR #193; nothing merged since the lanes branched). The
user was away; every call below was made by the comparator and is recorded here.

## 1. Inputs

| Lane | Branch, head, base | Record | Commits (oldest first) |
|---|---|---|---|
| 10x | `v2/10x` 080f126 off 8df5893 | `reviews/2026-11-28-v2-10x.md` | 5c9ef45 columns, helper, `--actions`, 13 tests in `tests/voxxa_eval_v2.rs` · a19a236 prompt 07 · 080f126 records |
| 10x, held | `v2/10x-full-held` d2211b6 off 8df5893 (local only) | 10x record §§ 1, 3, 6, 9 | one commit: the whole scope incl. `--truth-from-screen`, `--service-report`, `report.rs` (115 lines, 94 non-comment), the two example files, 20 tests; 774 / 600 |
| ponytail | `v2/ponytail` 7264b02 off 8df5893 | `reviews/2026-11-28-ponytail-v2.md` | 92f6d66 columns, `--actions`, `--truth-from-screen`, 8 tests, `fixtures/README.md` · 699aa38 prompt 07 · a251238 records · 7264b02 record fix |

Both lanes report: the eight numbers identical before and after (total `21 21 21 0 1.00 1.00 2310 4870 0 0 7280 2490`), `baseline.json` untouched, fmt and
clippy clean, `loc_budget.py` exit 0, media guard 417 tracked paths, lane scan 0 private terms in added lines. 10x: 284 / 0 / 1 tests, row
589 / 600, `--truth-from-screen` (about 42 lines) and `--service-report` (about 143) built and held back, the whole scope 774 / 600.
Ponytail: 279 / 0 / 1 tests, row 591 / 600, `--truth-from-screen` shipped (20 lines), `--service-report` written, measured at about 50
lines and removed before its first commit. Both report the shared target directory race (§ 9).

## 2. Comparison, column by column (`git diff 8df5893..<lane> -- src-tauri/src/cli/voxxa-eval/`)

| Item | 10x | ponytail | Kept |
|---|---|---|---|
| the helper | `delays` → `Vec<(bool, Option<i64>)>` per slide row, the start flag inside; 14 lines | `delays` → `Vec<Option<i64>>` per truth row (a blank row gets its first blank); 9 lines; `score` splits the starts from the rest | ponytail; the same delay on every slide row (the first goto to the row's cue in [previous row's t, next row's t), minus t_ms) |
| `on_time_1500`, `tight_300` denominators | non-song-start occurrences, missed ones included; none → NaN printed `n/a` | the same; none → `None` printed `-` (the eight names' convention) | ponytail |
| `early_300`, `early_1000` | every slide row, starts included, from the signed delay; decoupled from `EARLY_MS` | the same | same |
| song start | a slide row whose previous **slide** row is another song ("first slide of each song"; a reprise after a prayer is not one) | a slide row whose previous **truth** row is not a slide of the same song: the first row, the row after a blank, a new song | ponytail (§ 2.1) |
| `wrong_per_10_songs` | false-advance gotos only; "current" through the eight names' ownership (a row owns from 1 s before it); next = the next slide row | every goto whose slide is neither the row being sung at the goto's time (`t ≤ a.t`) nor the next slide row's | ponytail (§ 2.2) |
| `hidden_mid_song` | a false blank over a truth slide, not lifted (goto or unblank) within 1 s, counted to last + 10 s | a `blank` action while the truth shows a slide, over 1 s on the 50 ms grid; the truth's own blank ends it; never after the last truth row (no song end until truth v2) | ponytail (§ 2.3); 0 on every fixture either way |
| `wrong_slide_s` | a per-step compare of a screen table against the truth, by `Cue`, 50 ms, first row to last + 10 s | an inline walk, the same rule | same numbers on every fixture; total **74.30** in both |
| the signed error | a per-row list (`Metrics.rows`), no cell | a per-row list (`Metrics.signed`, `.starts`), no cell | same (the prompt names no cell; the columns are its summaries) |
| `--actions` | refuses none or several fixture dirs (3 lines); errors name the file and line | scores every dir given; errors name the file and line; an `error` line is named by its number only (a saved `--full` run may quote heard words) | ponytail, plus the 10x guard grafted |
| `--truth-from-screen` | held back: `parse_screen` 27 lines, collapses a settled re-report; `n/a` on the six V2 timing columns, the eight names' ms cells still printed | shipped: `parse_screen` 17 lines; `n/a` on every cell whose name holds `ms`, `300`, `1500` or `early` (median, p95, blank, unblank and the six V2 timing cells) | ponytail (Correct-bar metrics only, as the prompt says) |
| `slide` on a `screen` line | the global slide index, mapped through the setlist | the same | same (both records flag it for V8c) |
| column order | the plan's listing | timing, correctness, starts, seconds | ponytail (the base's; a reorder rewrites every pinned row for no information) |
| `main.rs` | `fn songs()`, `--actions`; `USAGE` +7 text lines | `fn songs()`, flags with a value in one map, three idioms shortened (−9 lines), `--truth-from-screen` dispatch | ponytail |
| `fixtures/README.md` | not edited | the two flags; the column table | ponytail |
| prompt 07 | conditional on the flag shipping | the flag is not in this build; names the `--actions` columns | ponytail, amended: deferred to V13s pending FA-79 |

### 2.1 Ruling: what a song start is

Sources: spec § 2.2 ("Song starts: first slide of each song: median and p95 of `t_action − t_w`, reported separately"; its status column
"needs about 4 heard words: ≥ 2 s by construction"; "On time: share of non-song-start occurrences shown within [early bound, t_w + 1.5 s];
early bound = start of the previous occurrence's last word when known, else t_w − 1 s"; "Why two numbers: ... a reactive system must
hear words before it moves"; "Unit: ... the statistical unit for confidence is the song performance (occurrences inside one song are
correlated)"), ADR-022 Decision 1 ("song starts reported separately") and Decision 4 ("Song start: ≥ 4 matched words and 2 consecutive
windows agree").

**A song start is a slide row whose previous truth row is not a slide of the same song: the first row, the row after a blank, the first
row of another song** (the ponytail rule). Why, from the definitions, not by preference:
1. The spec sets song starts apart for one reason: the follower re-enters from silence and must hear about four words first, so ≥ 2 s
   "by construction". A truth blank is the operator's record that the lyrics were down for talk; the slide row after it re-enters from
   silence the same way, under the same ADR-022 § 4 song-start gate (≥ 4 matched words, 2 windows), whether the song is new or a reprise.
2. The on-time window's early bound is "the start of the previous occurrence's last word": it presumes an occurrence leading straight
   into this one. After a blank none does (the previous occurrence's last word lies before the prayer), so the row after a blank is not
   one the on-time definition was written for; counting it there would also reward a follower that never blanked.
3. The statistical unit is the song performance because "occurrences inside one song are correlated" through continuous tracking; a
   prayer breaks the chain, so the reprise is a second performance, and `wrong_per_10_songs` divides by performances. The 10x rule
   gives the same answer wherever songs change without a blank; it differs only after a mid-song blank (`prayer-before-first-song`
   slide 1, `prayer-then-reprise` slide 1 again), and its record does not argue that case.
The denominator of `on_time_1500` and `tight_300` is the non-song-start **occurrences**, missed ones included (both lanes; ADR-022 § 1
and spec § 2.2 say "occurrences"; the plan's task-2 "hits" is read as the same share, since a missed occurrence is not on time).

Resulting total row on the six committed fixtures (`on_time_1500 tight_300 early_300 early_1000 wrong_per_10_songs hidden_mid_song song_start_median_ms song_start_p95_ms wrong_slide_s`):
**0.25 0.00 0 0 0.00 0 2800 3900 74.30**: 12 non-start rows, 3 on time (`jump-be-thou-my-vision` 1 of 3, `refrain-it-is-well` 2 of 5), 0 tight; 9 song
starts (900, 1500, 1710, 2490, 2800, 2900, 2900, 3900, 3900 ms; nearest-rank median the 5th = 2800, p95 the 9th = 3900). The 10x reading had
14 non-start rows, 4 on time (0.29) and 7 starts (median 2900). Per fixture, pinned in `replay_of_the_committed_fixtures_passes_the_baseline`
(commit 9c11ab5):

| fixture | on_time_1500 | tight_300 | early_300 | early_1000 | wrong_per_10_songs | hidden_mid_song | song_start_median_ms | song_start_p95_ms | wrong_slide_s |
|---|---|---|---|---|---|---|---|---|---|
| jump-be-thou-my-vision | 0.33 | 0.00 | 0 | 0 | 0.00 | 0 | 900 | 900 | 10.35 |
| prayer-before-first-song | - | - | 0 | 0 | 0.00 | 0 | 1710 | 2800 | 11.30 |
| prayer-between-songs | 0.00 | 0.00 | 0 | 0 | 0.00 | 0 | 2490 | 3900 | 20.65 |
| prayer-then-reprise | 0.00 | 0.00 | 0 | 0 | 0.00 | 0 | 1500 | 2900 | 7.50 |
| prayer-without-pause | 0.00 | 0.00 | 0 | 0 | 0.00 | 0 | 2900 | 2900 | 11.40 |
| refrain-it-is-well | 0.40 | 0.00 | 0 | 0 | 0.00 | 0 | 3900 | 3900 | 13.10 |
| total | 0.25 | 0.00 | 0 | 0 | 0.00 | 0 | 2800 | 3900 | 74.30 |

The eight numbers in cells 2–13 of every row are those measured at 8df5893 (both lane records § 5); the stand-ins are spoken, the conductor is
ADR-016's, so 0 of 12 within ±300 ms is the expected baseline for the follower work.

### 2.2 "Current" in the wrong-move rule
Spec § 2.2: "gotos to a slide that is neither the current nor the next truth occurrence, plus ... skips of ≥ 2". The current occurrence is
the one being sung when the goto fires (the last truth row at or before its time); the ponytail code reads it so. The 10x code reads
"current" through the eight names' ownership (a row owns from 1 s before it), so a goto two slides ahead fired less than 1 s before the
next row is "next" there and not wrong; by the spec's words it is a skip of two. The two agree on every other case under test (a double, the
next slide early, a goto before the first row, a move back, another song's slide). Kept: ponytail.

### 2.3 Hidden mid-song after the last truth row
Spec § 2.2: "a blank ... between a song's first sung word and its end". Truth v1 has no song end (`song_end` is V6's). The 10x code counts
a blank after the last row as hidden up to last + 10 s (borrowing `wrong_slide_s`'s tail); the ponytail code never counts one there and says
so in a comment and in `fixtures/README.md`. Kept: ponytail (nothing is read from a row that does not exist; V6 reads `song_end`). Both
give 0 on every fixture; the first ponytail build counted late starts as hides (6) and its own review fixed it before its commit (ponytail
record § 3).

### 2.4 Tests
- Ponytail (8 new, 3 changed, `tests/voxxa_eval.rs` 26 → 34): ±400 and 0; 1.5 s early as a false advance and both early counts; the
  300 / 301 ms boundary; three song starts incl. after a blank and a new song; `-` with no non-start row; a skip of two, a goto before the
  first row, the per-10 scaling, a double; wrong-slide seconds (2 s late, a wrong id, another song's slide, nothing shown, an empty truth, an
  unblank); hides (0.9 s, 6 s, the prayer's blank, the blank start, a blank carried out of a prayer, a blank after the last row); `--actions`
  on the committed public log (the replay row), a bad line named by file and line, an `error` line named by number only, a flag without a
  value; the screen truth (header, 22 cells, ten `n/a` cells, the Correct cells, the parser).
- 10x (13 new in `tests/voxxa_eval_v2.rs`): the same ground on a two-song truth with a prayer between, plus cases the ponytail set lacked.
  Grafted into `tests/voxxa_eval.rs` (commit 11cbcdc, one test file layout), adapted to the base's API (`Metrics::v2`, `signed`, `starts`,
  `wrong_moves`, `table(.., true)`): the helper on every truth row incl. the blank row, and the table's 22 cells
  (`1.00 0.67 1 0 0.00 0 400 500 1.90`); 900 ms early inside the window (a hit and an `early_300`) and 1.2 s early (a false advance, −1,200,
  both counts); a move back and another song's slide as wrong moves, a double not, a skip of two at 5 per 10 songs; a blank lifted by a goto
  at exactly 1 s (0) and at 2 s (1), a never-lifted blank (1); wrong-slide seconds 1.90, 42.00 with nothing shown, 0 for the same ids in an
  unplanned truth order, and the total row pooling two sets (1 wrong move over 4 starts = 2.5 per 10, 10 rows, 41.3 s); the committed
  `prayer-then-reprise/truth.jsonl` through `parse_truth` with made-up moves (`[1.0, 1.0, 2000, 2900, 0, 0, 200, 2000]`; the reprise a start at
  +2,000, the one other row +1,900 late); `--actions` on a made-up fixture (a hypothesis line skipped, a truth row by title, the 22 cells, OK
  against a made-up baseline, exit 2 with no dir and with two dirs). Not grafted: the cases the ponytail set already covers.
- The replay test now pins all 22 cells of every fixture row and the total (§ 2.1; commit 9c11ab5); the ponytail pinned 13, the 10x none.

## 3. Choice: base lane **ponytail**

By the brief's criteria in order:
1. Scope delivered within the cap: ponytail ships `--truth-from-screen` (task 7) at 591 / 600; 10x holds it back at 589 / 600. **ponytail.**
2. Definition correctness after § 2: the song-start rule (§ 2.1), "current" in the wrong-move rule (§ 2.2), `n/a` on every timing cell under
   a screen truth, `-` for nothing to measure. **ponytail.**
3. Test coverage: the 10x two-song cases grafted (§ 2.4); the ponytail set keeps the error-naming, screen-truth and boundary cases. **10x, grafted.**
4. The smaller row: 10x by 2 lines (589 against 591), without the flag. **10x, not decisive.**
Kept from 10x besides the tests: the `--actions` one-dir rule (3 lines, commit 11cbcdc). Not taken: its column order, NaN as `n/a`, the
hidden-after-last-row count, the conditional prompt 07 wording.

## 4. Final branch `v2/final` (`git log --oneline origin/main..HEAD`, oldest first)

| # | Final | Source | Scope |
|---|---|---|---|
| 1 | 313f37e | cherry-pick of ponytail 92f6d66 (clean) | `score.rs`, `main.rs`, `tests/voxxa_eval.rs`, `fixtures/README.md` |
| 2 | 47553a3 | ponytail 699aa38, amended | `kit/prompts/07-review.md`: the report deferred to V13s pending FA-79 |
| 3 | 9c11ab5 | comparator (§ 2.1) | `tests/voxxa_eval.rs`: all 22 cells per fixture row and total pinned |
| 4 | 11cbcdc | 10x 5c9ef45, adapted | `main.rs`: `--actions` scores exactly one fixture dir (+3); `tests/voxxa_eval.rs`: two fixtures, seven test functions |
| 5 | this commit | both lane records + comparator | this file; the two lane records; FA-79; prompts § V2 and § V13s; ADR-015; `process.md`; `status.md`; `handoff.md`; `decisions/sde`, `decisions/qa` and their indexes |

Commit identity `nategroks` and the two trailer lines on every commit. Draft PR #194 against `main`, title
"V2: evaluation v2 columns, --actions, --truth-from-screen; --service-report deferred to V13s (FA-79)"; merged by the comparator when every
check passed; no tag.

## 5. Gates on the final tree (Rust tree = commit 4; `CARGO_TARGET_DIR=C:/dev/v5a-target-fresh`, `-j 2`, one cargo command at a time, no other lane building)

| Gate | Result |
|---|---|
| `cargo fmt --check` | OK |
| `cargo clippy --all-targets -- -D warnings` | OK (19.7 s) |
| `cargo clippy --all-targets --features cli -- -D warnings` | OK (16.2 s) |
| `cargo test --features cli` | **286 passed, 0 failed, 1 ignored** (271 / 0 / 1 at 8df5893; `voxxa_eval.rs` 26 → 41; `eval_numbers_only` 6 checks) |
| `cargo build --features cli --bin voxxad --bin voxxa-eval && cargo run --features cli --bin voxxa-eval` | `mode: replay`; 21 / 21 / 21; total `21 21 21 0 1.00 1.00 2310 4870 0 0 7280 2490` then `0.25 0.00 0 0 0.00 0 2800 3900 74.30`; `OK: no metric is worse than fixtures/baseline.json`; exit 0 |
| `python scripts/loc_budget.py` (tokei 15.0.0) | exit 0, every row OK (§ 6) |
| the `media` job's `run:` block (`ci.yml`) | 417 tracked paths, no media outside the 15 allowed recordings |
| `git grep -lIP '[\x00\x0b]'` | nothing |
| `npm` steps | not run: nothing under `src/` or `tests/gui/` changed; `cargo test` did not need `../dist` (neither lane worktree had one) |
| lane scan, every commit | § 7 |

`cargo deny` is not installed here; V2 adds no dependency, CI covers it. Builds rewrite `src-tauri/gen/schemas/*.json` line endings (#91);
restored with `git checkout`, never staged.

## 6. Budget rows (`python scripts/loc_budget.py`)

| Row | 8df5893 | `v2/final` | Change | Allowance |
|---|---|---|---|---|
| voxxa-eval (`src/cli/voxxa-eval/`) | 488 / 600 (`main.rs` 176, `score.rs` 259, `audio.rs` 53) | **594 / 600** (`main.rs` 188, `score.rs` 353, `audio.rs` 53) | +106 | ≤ 105 for the whole V2 scope |

The whole scope measured 774 / 600 (10x: `main.rs` 254, `score.rs` 374, `report.rs` 93) and about 641 / 600 (ponytail: 591 + the leanest
`--service-report` about 50). The 105-line share held the columns, the helper, `--actions` and `--truth-from-screen` (ponytail 103, plus the
3-line guard) but not `--service-report`: about 50 lines against the 9 left after them (6 after the graft). The cap was not raised; the
report is deferred (FA-79, recommend (c): V13s). Unchanged: conductor 1,169 / 1,200; evidence 215 / 500; presenters 1,396 / 2,000; importers
1,373 / 1,600; everything else 894 / 904; wiring 1,288 / 1,438; discovery 223 / 250; updater 98 / 150; voxxad 552 / 600; recognizer 557 / 825;
onset 0 / 750; input chain 0 / 600; position 0 / 900; GUI 2,860 / 3,000; Simple mode 450 / 600; health check 0 / 450; the six drivers 128, 396,
180, 130, 169, 186 of 400; runtime npm dependencies none. No cap changed; no new row; `fixtures/baseline.json` untouched.

## 7. Lane scan per commit (scan 1 = private terms; scan 2 = the layout § 4.4 regex; counts per file, whole file / added lines; never a line)

| Commit | File | Scan 1 (private terms), whole / added | Scan 2 (regex), whole / added (class of the added hits) |
|---|---|---|---|
| 313f37e | `fixtures/README.md` | 6 / 0 (the founder section, pre-existing) | 31 / 0 |
| 313f37e | `src-tauri/src/cli/voxxa-eval/main.rs` | 0 / 0 | 3 / 0 |
| 313f37e | `src-tauri/src/cli/voxxa-eval/score.rs` | 0 / 0 | 0 / 0 |
| 313f37e | `src-tauri/tests/voxxa_eval.rs` | 0 / 0 | 10 / 2 (the committed public fixture paths) |
| 47553a3 | `kit/prompts/07-review.md` | 0 / 0 | 7 / 1 (`recordings\` as a path name in prose) |
| 9c11ab5 | `src-tauri/tests/voxxa_eval.rs` | 0 / 0 | 10 / 0 |
| 11cbcdc | `src-tauri/src/cli/voxxa-eval/main.rs` | 0 / 0 | 3 / 0 |
| 11cbcdc | `src-tauri/tests/voxxa_eval.rs` | 0 / 0 | 11 / 1 (the committed public fixture path in the grafted truth-file test) |
| records | `.10x/adrs/015-voxxa-eval-fixtures-and-replay-gate.md` | 0 / 0 | 7 / 0 |
| records | `.10x/decisions/qa/_index.md` | 0 / 0 | 0 / 0 |
| records | `.10x/decisions/qa/church-autoadvance.md` | 3 / 0 (pre-existing lines, FA-12 (c)) | 6 / 0 |
| records | `.10x/decisions/sde/_index.md` | 0 / 0 | 0 / 0 |
| records | `.10x/decisions/sde/church-autoadvance.md` | 4 / 0 (pre-existing lines, FA-12 (c)) | 19 / 0 |
| records | `.10x/handoff.md` | 9 / 0 (pre-existing lines, FA-12 (c)) | 12 / 0 |
| records | `.10x/reviews/2026-11-28-ponytail-v2.md` | 0 / 0 | 2 / 2 (the committed public fixture paths named in its scan table and text) |
| records | `.10x/reviews/2026-11-28-v2-10x.md` | 0 / 0 | 6 / 6 (the wav type named as a media type in a rule; the public fixture paths and the prompt-07 syntax its scan table quotes) |
| records | `.10x/reviews/2026-11-28-v2-comparator.md` | 0 / 0 | 3 / 3 (the public fixture folder, and the `recordings\` path name in section 7's text and in this row) |
| records | `.10x/specs/2026-11-24-founder-action-items.md` | 0 / 0 | 4 / 0 |
| records | `.10x/specs/2026-11-24-process.md` | 0 / 0 | 3 / 0 |
| records | `.10x/specs/2026-11-26-prompts-round-3.md` | 0 / 0 | 9 / 0 |
| records | `.10x/status.md` | 8 / 0 (pre-existing lines, FA-12 (c)) | 5 / 0 |

Private terms in added lines: 0 in every file of every commit. The whole-file private-term counts are pre-existing lines (`fixtures/README.md`'s
founder section; `status.md`, `handoff.md` and `decisions/sde`, `decisions/qa` per FA-12 (c)). Regex hits in added lines are the committed
public fixture paths under `fixtures/recordings/` (tests, records), the `recordings\` path name in prose (prompt 07, the records) and, in the
10x record, the wav extension named as a media type in a rule. Control bytes: none at any commit.

## 8. Open points decided

1. **`--service-report` does not ship in this PR** (both lanes measured it over the cap); **FA-79** records the facts and three options and
   recommends (c), deferral to V13s, whose prompt gains the writer as task 7; the 10x lane's full implementation stays on the local branch
   `v2/10x-full-held` (never pushed) and in its record § 6.
2. **The song-start rule** is § 2.1's; the per-fixture values are pinned (commit 9c11ab5).
3. **`--actions` scores exactly one fixture dir** (the 10x rule, 3 lines; 594 / 600). The ponytail's review had named it as the one
   "should fix" left out for the cap.
4. **Column order** stays the base's (timing, correctness, starts, seconds).
5. **`-` for nothing to measure, `n/a` for a cell a screen truth cannot fill** (the base's two cells); `n/a` covers the eight names' ms cells too.
6. **Hidden mid-song after the last truth row is not counted** until V6's `song_end` (§ 2.3).
7. **`fixtures/README.md`** documents the columns and both flags (from the ponytail lane); `kit/skills/voxxa-wav-intake/SKILL.md` stays as
   it is (both lanes; tech debt in the SDE entry).
8. **Prompt 07** says the report is deferred to V13s pending FA-79 and names the `--actions` columns the review reads meanwhile.
9. **ADR-015's review trigger** ("more than 600 lines") is marked reached 2026-11-28 with the measured numbers; FA-79 answers it. ADR-022
   stays Proposed.
10. **The shared target directory race** both lanes hit is a dated process line (`process.md` § 3 item 3): a target directory per lane,
    `-j 2`, one cargo command at a time.
11. **Draft PR number:** the repository's last issue or PR number was 193 when this record was written, so the records name #194; the
    comparator verifies the number after `gh pr create` and corrects it with a commit if it differs.

## 9. Process notes

- The harness moves a foreground command to the background after 180 s; `cargo test` crossed that line and was waited for; no second
  cargo command ran meanwhile. One cargo command at a time, all at `-j 2`; the other lanes were idle, so the shared target directory saw
  only this worktree's binaries.
- Both lanes record the race: cargo's directory lock serialises builds, not a build against the other lane's running tests, and the two
  worktrees produce identically named `voxxa-eval.exe`, `voxxad.exe` and `voxxa_eval-*.exe`; the ponytail lane saw two refused link steps
  and two contaminated full runs, the 10x lane two LNK1104 failures. Both final counts were taken with the other lane idle.
- The system git config has `core.autocrlf=true` (index LF, worktree CRLF); edits were made CRLF-aware; `cargo fmt` rewrote the test file
  with LF, which git normalises on commit.
- Skills: none of the 10x-Team skills, no `bd`/beads, no tracker, no iceberg; `superpowers:systematic-debugging` reserved for a CI failure.
- Safety: no microphone, audio device, GUI, Presenter, FreeShow or installed app; `voxxad` only as `--replay` inside the tests and
  `voxxa-eval` on the committed fixtures and made-up files; the private folders untouched except the scan-terms file read by `grep -f`; no
  worktree removed; the lane worktrees and the held branch read only; nothing but `v2/final` pushed; no force-push.
