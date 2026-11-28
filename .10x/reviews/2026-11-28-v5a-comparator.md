# V5a comparator record (2026-11-28, `v5a/final` off 3ff5302)

Numbers, names and paths only. The comparator read both lane worktrees (`C:\dev\v5a-10x`, `C:\dev\v5a-pony`) and modified
neither; it built `C:\dev\v5a-final` on branch `v5a/final`, created off 3ff5302 (origin/main: 54d841e + PR #191 at 79388b6 +
PR #192 at 3ff5302, which touch `founder-action-items.md`, `prompts-round-3.md`, ADR-028 and `reviews/2026-11-27-v3d-port-plan.md`
only; no file either lane changed). The user was away; every call below was made by the comparator and is recorded here.

## 1. Inputs

| Lane | Branch, head, base | Record | Commits (oldest first) |
|---|---|---|---|
| 10x | `v5a/10x` 0545c73 off 54d841e | `reviews/2026-11-27-v5a-10x.md` | 6116b82 kit · cd1a69a guards · f6e4b5e importer · 573c318 records · 0545c73 record fix |
| ponytail | `v5a/ponytail` 9405ddb off 54d841e | `reviews/2026-11-27-ponytail-v5a.md` | f520bf6 kit · 32c12ea guards · 44a1b9a importer · 02f48e3 records · 9405ddb record fix |

Both lanes report: 271 tests passed, 0 failed, 1 ignored; eval 21 / 21 / 21; fmt and clippy clean; `loc_budget.py` exit 0; 42 kit
files; the congregation notice excluded; ADR-026/027/029 Accepted; CODEOWNERS 3 patterns; `.gitignore` 11 names + 2 re-includes.

## 2. Comparison

### 2.1 Importer (`git diff 54d841e..<lane> -- src-tauri/`)

- The Rust trees differ in 5 files only: `src/aligner/mod.rs`, `src/importers/txt.rs` and the test files. The 10 mechanically
  converted files (`hymnary/mod.rs`, 7 importers, `planning_center/mod.rs`, `tests/conductor_evidence.rs`: 20 `Slide { id, text }`
  literals → `Slide::new`) are byte-identical between the lanes (`git diff f6e4b5e 44a1b9a --stat -- src-tauri`: 5 files).
- `aligner/mod.rs`: both add `kind: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` and
  `Slide::new(id, text)`; the doc comments differ in wording only (10x: spec § 4.5; ponytail: ADR-022 § 7).
- `txt.rs`, `section_label(line) -> Option<SectionKind>`: identical mapping in both (`verse` → Verse; `chorus`, `refrain` → Chorus;
  `bridge` → Bridge; `tag` → Tag; `pre-chorus`, `pre chorus`, `prechorus`, `intro`, `outro`, `ending`, `interlude` → Other; anything
  else is not a label). The plan's `[UNVERIFIED]` resolves the same way in both: every accepted label has a `SectionKind`, so the
  "unmapped label is dropped" branch never fires.
- `txt.rs`, `parse`: both walk the blocks once, open a `Section` at a label, add each following slide's index to the open section,
  keep the label off the slide text, drop a section with no slide (`sections.retain`), and give a slide whose first line starts with
  `Credit:` (exact case) `kind: Some("credit")` in no section. Differences: (a) 10x indexes `block[0]` directly, ponytail uses
  `block.first().copied().and_then(section_label)`; blocks are pushed only when non-empty (`txt.rs:20-33`), so both are safe;
  (b) 10x closes the open section at the credit slide (an `open` flag), ponytail lets a later unlabelled slide rejoin the preceding
  section; neither the plan nor the spec says, a credit slide is the last slide in every kit example, and "the credit slide ends the
  sung part" is the reading both records argue from (`aligner/evidence/mod.rs` reads section membership for the chorus-end edge);
  (c) 10x builds the slide as one literal `Slide { id, text, kind }`, ponytail as `Slide::new` plus an assignment.
- Correctness against plan V5 step 9 and spec § 4.5: both correct, same rule. Tie; (b) favours 10x by a hair.

### 2.2 Kit (`diff -r v5a-10x/kit v5a-pony/kit`)

42 files in both; 12 differ: `README.md`, `private-folder.md`, `prompts/01-intake.md`, `prompts/02-song-library.md`,
`skills/voxxa-song-intake/SKILL.md`, `templates/bindings/freeshow.json`, `templates/church-profile.template.json`,
`templates/church-setup-checklist.md`, `templates/intake.template.json`, `templates/lab/pyproject.template.toml`,
`templates/minder-role-card.md`, `templates/presenter-test-card.md`.

- Private-term scan (the lane brief's first command) over every tracked kit file of **both** lanes: 0 in every file, both lanes.
- Regex scan over both kits: the same 15 files with hits in both; the ponytail kit carries two hits the 10x kit reworded:
  `templates/presenter-test-card.md` FS2's answer label (the word password followed by a colon, a prose label, not a configuration
  field name) and `templates/lab/pyproject.template.toml`'s project `name` (it carried the private-folder phrase; 10x: `voxxa-lab`).
- 10x fixes the ponytail kit lacks: F12 (consent placeholders `…-alex-example.pdf`, `…-jo-example.pdf` → `…-singer-1-example.pdf`,
  `…-reader-1-example.pdf`: no person-name-shaped placeholder), F13 (prompt 02 and the song-intake skill say a label names the slides
  that follow it and that voxxa marks the `Credit:` slide, matching the importer built), F14 (the two regex hits above), and
  `private-folder.md` rule 7 naming `.gitignore` as the guard in place now with the media guard and `kit_check` as V5b (the ponytail
  text says the media guard refuses the names, which is V5b).
- Ponytail text the 10x kit lacks and the comparator grafted: the README status paragraph naming the kit's own tools
  (`tools/songcheck.py`, `tools/kit_check.py`) as step V5b's and the tree's `tools/` line saying "not in kit-v0.1.0"; the FA-47
  approval date (2026-11-26, `founder-action-items.md:402`) in `church-profile.template.json`'s tier note.
- Equivalent, kept from 10x: the tier table (both carry Tier A autonomy-eligible after the prompt-07 gates, Tier B assisted with
  suggestion cards and a minder, "never promises autonomy on Tier B"); the FA-57, FA-53, FA-61 corrections; the uv 0.13.10 minimum;
  the lowercase install-folder path; the README tree's four later-step drafts (10x groups them at the end of `templates/`, marked by
  step; ponytail lists them mid-tree).
- Both lanes list and source their fixes (10x F1–F14 with FA numbers; ponytail K1–K9 with file:line).

### 2.3 Guards

- `.github/CODEOWNERS`: the same 3 patterns → `@nategroks` in both; comments and column alignment differ. Kept: 10x.
- `.gitignore`: the same 11 names in both. Re-includes: both `!fixtures/recordings/*/truth.jsonl`; 10x
  `!src-tauri/tests/fixtures/voxxad/session-*.jsonl` (the folder's future public test logs included), ponytail
  `!src-tauri/tests/fixtures/voxxad/session-pd-hymns.jsonl` (exact; its record notes a new log needs its own line). Kept: 10x.

### 2.4 Tests

- 10x: one file `src-tauri/tests/txt_sections.rs`, the three names. `old_setlists_replay_unchanged` compares every committed
  setlist's whole JSON after a round trip (the ADR-016 `refrain` → `chorus` alias normalised first) and replays all 7 committed
  logs, comparing action lines; failures print counts only. Pins the `---` split inside a labelled section, a label with no slide,
  the kinds mapping, a plain file, `Credit:` on a first line only and case-sensitive, `kind` written only when set.
- ponytail: `tests/importer_slides.rs` (2 tests) and `tests/replay_determinism.rs` (1). Its replay test compares slides only (not
  sections) and checks each replay's output is non-empty. It pins two things the 10x file did not: slides before the first label are
  in no section (with a colon label `Chorus:` and a numbered `Refrain 2`), and an `{"id","text"}` slide reads back with `kind: None`.
- Kept: the 10x file; the two ponytail pins merged into it (85677b7). Three test functions, same names.

### 2.5 ADRs and state files

- ADR-026/027/029: both lanes set Accepted 2026-11-27 and cite the same founder records (FA-13 with FA-01; FA-03; FA-01 and
  FA-12 (c); FA-26 not cited by either). 10x adds an "Amendment 1" section per ADR with line-cited sources and what V5a built;
  ponytail puts everything on the status line. Kept: 10x's, with ", 10x lane" dropped from the status line and the amendment
  heading of each ADR (2 places per file; the lane is named here and in `status.md`, not in the ADRs).
- `status.md`, `handoff.md`, `decisions/sde`, `decisions/security`: rewritten by the comparator into one V5a entry each, taking
  the 10x text as the frame and the ponytail lane's additions (the replay-fingerprint argument, the `evidence/mod.rs` chorus-end
  reason, the church-profile field list, the negation-line note, V11 as the first reader of `kind`).

## 3. Choice: base lane **10x**

By the brief's criteria in order:
1. Importer correctness against plan V5 step 9 and spec § 4.5: tie (same rule; § 2.1).
2. Kit with the fewest remaining problems: both 0 private-term hits everywhere; 10x's fixes are listed and sourced and include the
   two regex hits the ponytail kit still carries, the placeholder names and the rule-7 accuracy (§ 2.2). **10x.**
3. Smaller rows: ponytail (importers 1,370 against 1,373; the other rows equal). **ponytail, by 3 lines.**
4. Clearer tests: one file, stronger replay pin (§ 2.4). **10x.**

## 4. Final branch `v5a/final` (`git log --oneline origin/main..HEAD`, oldest first)

| # | Final | Source | Scope |
|---|---|---|---|
| 1 | eb1d5a6 | cherry-pick of 10x 6116b82 | `kit/`, 42 files |
| 2 | 4a1fae9 | cherry-pick of 10x cd1a69a | `.github/CODEOWNERS`, `.gitignore` |
| 3 | 893fb9e | cherry-pick of 10x f6e4b5e | `Slide.kind`, `Slide::new`, `txt.rs`, 10 literal conversions, `tests/txt_sections.rs` (13 files) |
| 4 | 9f260d5 | graft from ponytail f520bf6 | `kit/README.md` status paragraph and `tools/` line; `church-profile.template.json` FA-47 date |
| 5 | 85677b7 | graft from ponytail 44a1b9a | two pins merged into `tests/txt_sections.rs` |
| 6 | 397f207 | graft from ponytail 9405ddb | `reviews/2026-11-27-ponytail-v5a.md` |
| 7 | this commit | 10x 0545c73 (ADRs, its record) + comparator | ADR-026/027/029 Accepted; `reviews/2026-11-27-v5a-10x.md`; this file; `status.md`, `handoff.md`, `decisions/sde`, `decisions/security` |

The three cherry-picks applied clean. Commit identity `nategroks` on every commit; the two trailer lines on every commit (the
cherry-picked messages keep the lane's blank line between the two trailers).

## 5. Gates on the final tree (Rust tree = commit 5; `CARGO_TARGET_DIR=C:/dev/v5a-target-fresh`, `-j 2`, one command at a time)

| Gate | Result |
|---|---|
| `npm ci && npm run build` (the `../dist` prerequisite; nothing under `src/` or `tests/gui/` changed, so `test:gui` was not run) | built |
| `cargo fmt --check` | OK |
| `cargo clippy --all-targets -- -D warnings` | OK |
| `cargo clippy --all-targets --features cli -- -D warnings` | OK |
| `cargo test --features cli` | **271 passed, 0 failed, 1 ignored** (31 result lines; `txt_sections` 3 / 3 in 113.8 s) |
| `cargo build --features cli --bin voxxad --bin voxxa-eval && cargo run --features cli --bin voxxa-eval` | 21 / 21 / 21 hits, precision and recall 1.00, 0 false, 0 double, median 2,310 ms, p95 4,870 ms, "OK: no metric is worse" |
| `python scripts/loc_budget.py` (tokei 15.0.0) | exit 0, every row OK (§ 6) |
| the `media` job's `run:` block (`ci.yml:181-215`) | 415 tracked paths at commit 6, no media outside the 15 allowed recordings; 417 at commit 7 |
| `git grep -nE '"(text|lyrics|slides|words|run)"' -- 'kit/**/*.json'` | empty |
| `git check-ignore --no-index` over the 42 kit files | 0 ignored; 13 planted names caught; `git ls-files -ci` 15 (the allowlisted recordings); 7 re-included public fixtures tracked |
| kit hygiene | capital "Voxxa" 0; `Users` 0; the founder's login 0; 11 JSON files parse; `VOXXA_FS_KEY` once (an identifier) |
| lane scan, every commit | § 8 |

`cargo deny` is not installed here; V5a adds no dependency, CI covers it. `src-tauri/gen/schemas/*.json` were touched by the
build (line endings only) and restored with `git checkout`, never staged.

## 6. Budget rows (`python scripts/loc_budget.py`)

Before (54d841e, both lanes' tables; 3ff5302 has no Rust change) → after (`v5a/final`):

| Row | Before | After | Change | V5a allowance |
|---|---|---|---|---|
| conductor (`aligner/`) | 1,173 / 1,200 | **1,169 / 1,200** | −4 | ≤ +3 |
| importers/ + planning_center/ | 1,386 / 1,600 | **1,373 / 1,600** | −13 | ≤ +40 |
| everything else in src-tauri/src | 900 / 904 | **894 / 904** | −6 | no growth |

Unchanged: conductor evidence 215 / 500; presenters 1,396 / 2,000; wiring 1,288 / 1,438; discovery 223 / 250; updater 98 / 150;
voxxad 552 / 600; recognizer 557 / 825; voxxa-eval 488 / 600; onset 0 / 750; input chain 0 / 600; position 0 / 900; GUI 2,860 /
3,000; Simple mode 450 / 600; health check 0 / 450; drivers 128, 396, 180, 130, 169, 186 of 400; runtime npm dependencies none.
No cap changed; no Python row.

## 7. Kit on the final tree: 42 files; regex-scan classification

`README.md`, `private-folder.md`, 27 `templates/*` (6 `bindings/*.json`, `lab/pyproject.template.toml`), 8 `prompts/`, 5
`skills/*/SKILL.md`. Excluded: `templates/congregation-notice.template.md` (FA-70). Private-term scan: 0 in every file.

Regex scan (counts of matching lines per file; 27 files at 0): `README.md` 2; `private-folder.md` 9; `prompts/02-song-library.md`
3; `prompts/07-review.md` 6; `prompts/08-wav-intake.md` 10; `skills/voxxa-song-intake/SKILL.md` 1; `skills/voxxa-wav-intake/SKILL.md`
5; `templates/agent-settings.example.md` 4; `templates/bindings/freeshow.json` 1; `templates/church-setup-checklist.md` 1;
`templates/intake.template.json` 1; `templates/lab/pyproject.template.toml` 1; `templates/presenter-test-card.md` 2;
`templates/service.template.json` 1; `templates/wav-intake-checklist.md` 16. Total 63 lines.

Matched fragments (count, class): the `recordings` folder name followed by a separator 39 (a private-folder layout name used as a
path, `private-folder.md`'s layout); the wav extension 28, mp3 3, m4a 3, mp4 1 (media **types** named in the intake instructions
and the wav-intake checklist, the same types the deny rules name); the `songs` folder name 11 (layout path); the private root's
generic name 6 (the path ADR-029's alternatives table allows, in prose); the loopback address 4 (`freeshow.json` 1,
`presenter-test-card.md` 2, `agent-settings.example.md` 1). No e-mail address, no other address, no token, no password label, no
user-profile path, no project name carrying the private phrase. Classification call: the brief's classes say media types "in a
deny rule" and folder names "in prose or `.gitignore`"; the kit names them in prose and in generic paths, which the comparator
classified as the same intent (the kit describes the private folder; rewriting would remove its subject). Both lanes made the
same call.

## 8. Lane scan per commit (scan 1 = private terms; scan 2 = the layout § 4.4 regex; counts per file, never a line)

| Commit | Files | Scan 1, whole files | Scan 1, added lines | Scan 2, whole files (nonzero) | Scan 2, added lines (fragments) |
|---|---|---|---|---|---|
| eb1d5a6 kit | 42 | 0 in every file | 0 | the 15 files of § 7 (63 lines) | 63: § 7's fragments |
| 4a1fae9 guards | 2 | 0 | 0 | `.gitignore` 6 (4 media types and the private fixtures folder, pre-existing; 1 added) · `CODEOWNERS` 0 | 1: the recordings folder in the re-include path |
| 893fb9e importer | 13 | 0 | 0 | `tests/conductor_evidence.rs` 2 (pre-existing fixture paths); 12 files 0 | 0 |
| 9f260d5 kit text | 2 | 0 | 0 | `README.md` 2 (pre-existing) | 0 |
| 85677b7 test pins | 1 | 0 | 0 | 0 | 0 |
| 397f207 ponytail record | 1 | 0 | 0 | 7 | 7: recordings folder 5, private fixtures folder 2, private root name 1, songs folder 1, the password label as deny-pattern text 1, loopback 1 |
| records (this commit) | 9 | `decisions/sde` 4, `handoff.md` 9, `status.md` 8 (pre-existing lines, FA-12 (c)); 6 files 0 | 0 | ADR-026 1, ADR-027 1, ADR-029 8, `decisions/sde` 19, `decisions/security` 11, `handoff.md` 12, `status.md` 5, this file 1, the 10x record 0 | 6: the recordings folder name inside the public truth-file re-include path (`decisions/sde` 2, `decisions/security` 1, `handoff.md` 1, `status.md` 1, this file 1) |

The records commit was measured on its staged files before committing (the table's last row); its added lines carry 0 private
terms, and their only regex fragment is the recordings folder name in the public re-include path. Control bytes
(`grep -lP '[\x00\x0b]'`): none at any commit.

## 9. Open points decided

1. **The four later-step draft templates** (`incident.template.md` V13s, `minder-role-card.md` V18,
   `repertoire-inventory.template.csv` V17, `rota.template.csv` V18) **stay**: the brief copies `templates/` whole and excludes only
   the notice; each passes the private-term scan, the regex classes, the lyric-key grep and `check-ignore`; each is marked by step
   in the README tree and `minder-role-card.md` says it is not for a Sunday yet.
2. **Regex-scan classification**: § 7, verified on the final tree, counts per file; the same call as both lanes.
3. **The credit slide joins no section**, and closes the open one (10x's reading; § 2.1 (b)); `credit_slide_kind_detected` pins it.
4. **`Slide::new`** over `kind: None` at 20 sites (both lanes; every row shrinks).
5. **`kind` is a string**, not an enum, until V11 reads it (both lanes).
6. **`Credit:` exact case** (both lanes; the spec, ADR-026 § 1 and every kit file write it so).
7. **The lane name stays out of the ADRs** (status line and amendment heading say "V5a"); the lanes are named here and in `status.md`.
8. **ADR-022 stays Proposed** (its § 7 is built; accepting it is not in V5a).
9. **`.gitignore` re-include by pattern** for `src-tauri/tests/fixtures/voxxad/session-*.jsonl` (10x), not by one file name.
10. **Draft PR number**: the repository's last issue or PR number was 192 when this record was written, so the records name #193;
    the comparator verifies the number after `gh pr create` and corrects it with a commit if it differs.

## 10. Process notes

- The harness moves a foreground command to the background after 180 s; `cargo test --no-run`, `cargo test` and nothing else
  crossed that line. One cargo command ran at a time, all at `-j 2`; no memory event this run.
- The system git config has `core.autocrlf=true` (index LF, worktree CRLF); edits were made CRLF-aware; the test-file graft was
  written whole.
- Skills: none of the 10x-Team skills, no `bd`/beads, no tracker, no iceberg; `superpowers:systematic-debugging` reserved for a CI
  failure.
- Safety: no microphone, audio device, GUI, Presenter, FreeShow or installed app; `voxxad` only as `--replay` inside the tests and
  `voxxa-eval` on the committed fixtures; the private folders untouched except the scan-terms file read by `grep -f`; no worktree
  removed; the lane worktrees read only. Pushes: `v5a/final` and the `kit-v0.1.0` tag only.
