# Ponytail lane — V5a (2026-11-27)

Independent pass, as the process rule asks (user, 2026-11-11). The task: plan step **V5a** of `specs/2026-11-26-prompts-round-3.md`
(the modification block over `specs/2026-11-24-church-autoadvance-plan.md` § 4 V5: steps 1, 4's `.gitignore` half, 5's CODEOWNERS and 9,
plus the FA-47 tier table and the Phase 2 check on ADR-026, 027 and 029). Method: `ponytail` at level `full` and `ponytail-review`.
Worktree `C:/dev/v5a-pony`, branch `v5a/ponytail` off `origin/main` 54d841e. The 10x lane's worktree was not read. The user was absent:
every question ponytail would have asked is decided below (§ 2) and recorded, nothing was stopped on.

Commits (`git log --oneline origin/main..HEAD`): **f520bf6** (kit, 42 files), **32c12ea** (guards: CODEOWNERS and `.gitignore`), **44a1b9a**
(importer, `Slide.kind`, the three tests), then the records commit (this file, the three ADR status lines, the two decision files, `status.md`,
`handoff.md`). Nothing was pushed; no PR, no GitHub write, no workflow ran on GitHub.
Safety: no microphone, no audio device, no `voxxad` outside `--replay` (the tests), no GUI, no key pressed in any app, no private folder read
beyond the scan-terms file the scan command reads for me, no lyric printed (the public-domain fixture setlists were inspected by key
names and byte counts only).

## 1. What I built

| File | Change |
|---|---|
| `kit/` (42 files) | Copied from `C:\dev\voxxa-strategy-staging\out\kit\`: `README.md`, `private-folder.md`, `prompts/01…08`, 5 `skills/*/SKILL.md`, 29 `templates/*` incl. `bindings/*.json` (6) and `lab/pyproject.template.toml`. **Not committed:** `templates/congregation-notice.template.md` (FA-70: no notice at all). Fixes: § 4. |
| `kit/README.md` | New section "Two tiers of presentation app (FA-47)": one paragraph and a two-row table (Tier A jump + read-back: FreeShow, ProPresenter, OpenLP in part, autonomy-eligible after the prompt-07 gates; Tier B keys only: WorshipTools Presenter, universal, assisted with suggestion cards and a minder at the keys; the kit never promises autonomy on Tier B). |
| `.github/CODEOWNERS` (new, 5 lines) | `/kit/prompts/**`, `/kit/skills/**`, `/kit/templates/bindings/**` → `@nategroks`. Nothing about `kit_check` (V5b). |
| `.gitignore` | +17 lines: `church-profile.json`, `layout.json`, `*.service.json`, `songs.csv`, `practice/`, `sessions/`, `session-*.jsonl`, `truth.jsonl`, `*.sqlite3*`, `manifests/`, `reports/`, then two negations for the public fixtures those names would hide: `!fixtures/recordings/*/truth.jsonl` (6 tracked files) and `!src-tauri/tests/fixtures/voxxad/session-pd-hymns.jsonl` (1). `git check-ignore -v` on all 29 kit templates: 0 ignored, so no template negation was needed (their names are `*.template.*`, `songs.template.csv`, `truth-v2.example.jsonl`). 13 planted private names are ignored (§ 6). The media-guard test and CI job are untouched (V5b). |
| `src-tauri/src/aligner/mod.rs:22-37` | `Slide.kind: Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` (ADR-022 § 7), and `Slide::new(id, text)` (kind `None`). The two test fixtures that built `Slide` literals use `Slide::new` (`:777-816`, `:1288`). Row 1,173 → **1,169** (−4; share was ≤ +3). |
| `src-tauri/src/importers/txt.rs` | `section_label(line) -> Option<SectionKind>` replaces `looks_like_section_label` (`:91-108`): `verse`, `chorus`/`refrain`, `bridge`, `tag` as themselves; `pre-chorus`, `pre chorus`, `prechorus`, `intro`, `outro`, `ending`, `interlude` → `Other`; anything else is not a label. `parse` (`:50-77`) walks the blocks once: a label starts a `Section`, the slides up to the next label join it, the label stays off the slide text, a label with no slide is dropped (`retain`, `:77`); a slide whose first line starts with `Credit:` gets `kind: "credit"` and joins no section (`:67-74`). Header comment updated. |
| `src-tauri/src/hymnary/mod.rs`, `importers/{chordpro,easyworship,openlyrics,opensong,pdf,pptx,pro7}.rs`, `planning_center/mod.rs` | Each `Slide { id, text }` literal becomes `Slide::new(id, text)` (a struct literal must name every field, so the new field reached all 20 constructor sites). Importers row 1,386 → **1,370** (−16; share ≤ +40); everything else 900 → **894** (−6; cap 904 unchanged, "no other row grows" holds, the 4 reserved `lib.rs` lines are now 10). |
| `src-tauri/tests/importer_slides.rs:100-166` | `txt_section_labels_fill_sections` (made-up words: 7 slides, sections Verse [1, 2], Chorus [3, 4], Other [5], Chorus [6]; a trailing `Tag` dropped; an unlabelled file keeps `sections` empty; `check_sections` passes) and `credit_slide_kind_detected` (kind on the credit slide only, in no section; `"kind"` written once per slide list; a `{"id","text"}` slide reads back with `kind: None`). |
| `src-tauri/tests/replay_determinism.rs:107-133` | `old_setlists_replay_unchanged`: the 7 committed setlists plus `setlist.example.json`, read as `Setlist` and written back as JSON values, equal the file's own JSON (no new key, same values; a mismatch prints a yes/no, never the text), every slide reads back with `kind: None`, and `voxxad --replay` accepts every committed log (replay refuses a setlist whose FNV fingerprint changed, so success is the byte-for-byte proof; the action-line comparison stays in the first test). |
| `src-tauri/tests/conductor_evidence.rs:135`, `:397` | `Slide::new` (compile fix, 2 sites). |
| `.10x/adrs/026`, `027`, `029` | Status lines Proposed → **Accepted (2026-11-27, V5a)**, each citing its founder record (§ 7). |
| Records | this file; `decisions/sde/church-autoadvance.md` and `decisions/security/church-autoadvance.md` ("V5a"); `status.md` (V5a paragraph, Next pointer); `handoff.md` (V5a lines, row table rows for conductor, importers, everything else). |

## 2. Assumptions and decisions (the user was absent)

1. **"The slides it covers"** (plan step 9) = from the label's first slide up to the slide before the next label. An unlabelled block after a labelled one continues that section (a verse split over two slides). ADR-022 § 7's "label plus first slide index" is the same thing for one block per label.
2. **A label with no slide after it** = a label followed directly by another label or the end of the file (its section ends up empty and is dropped). A label-only block followed by an unlabelled block covers that block.
3. **Unmapped labels** (`pre-chorus`, `intro`, `outro`, `ending`, `interlude`) map to `SectionKind::Other`, not dropped: `Other` is ADR-016's documented catch-all (`sections.rs:8-9`, `#[serde(other)]`) and ADR-024 § 1 gives it a printed label (`o<n>`), so this is the ADR-016 mapping, not a guess. `refrain` → `Chorus` as the serde alias does.
4. **A credit slide joins no section.** `evidence/mod.rs:218-219` decides "the chorus ends here" from section membership; a never-sung credit slide inside the last chorus would move that edge by one slide. The spec says only that the slide gets `kind: "credit"`.
5. **`Credit:` is matched exactly** (capital C, colon), as the spec, ADR-026 § 1, the kit's prompt 02 and `church-setup-checklist.md` § 4 all write it. `credit:` or `Credits:` is a normal slide.
6. **`Slide::new` instead of `kind: None` at every literal.** A struct literal must name every field, so 20 constructor sites had to change either way. With `kind: None` rustfmt's 18-character struct-literal limit turns each site vertical: +10 lines in the conductor row (share ≤ 3), +2 in "everything else" (4 left, all reserved), +15 in importers. `Slide::new` costs 9 lines once and lets every site shrink to one line: every row goes down. Not an abstraction for later: one constructor, used at every site.
7. **Where the tests live.** The two importer tests sit in `tests/importer_slides.rs` (the importer slide-numbering file; uncounted), not inline in `txt.rs`, where their ~60 lines would have used most of the 40-line share. `old_setlists_replay_unchanged` sits beside the replay tests that own `common::committed_logs()` and the `voxxad` runner.
8. **ADR-026's record** is FA-13 (licence policy "as written"; the founder record's closing line, `founder-action-items.md:239`, states every FA-01 … FA-77 answered) plus FA-01's pre-approval of V5. FA-26 is about relicensing the church's own songs and does not name the register, so it is not cited.
9. **One gate run on the final tree** covers the four commits: the kit and guard commits add no code, and the Rust commit's tree is the final tree minus records. Budgets were measured before (54d841e) and after (§ 5); the lane scan ran on each commit's staged files (§ 6).
10. **Kit files that belong to later steps** (`freeshow.json`, `presenter-test-card.md` FS rows, `minder-role-card.md`, `church-profile.template.json`) are committed as drafted, with only the lines that contradict a recorded founder answer corrected (§ 4). V8b, V17 and V18 finish them.
11. **`.gitignore` negations for the public fixtures.** `truth.jsonl` and `session-*.jsonl` would have hidden 7 tracked public files from `git status`; the plan says "except the existing public fixtures", so each gets a negation by path. A new public session log named `session-*.jsonl` needs its own line (noted in `handoff.md`).

## 3. Ponytail review of the change (ponytail-review, on my own diff)

What this change does: the `.txt` importer now keeps section labels as `sections` and marks `Credit:` slides, `Slide` gains an optional
`kind`, the kit skeleton, CODEOWNERS and the `.gitignore` guards land, and three ADRs move to Accepted. Load assumed: one tech lead
importing a few hundred `.txt` files on a booth PC; CI replaying 7 logs.

**Must fix** — none left open.

1. **A failing setlist comparison would have printed slide text** (`tests/replay_determinism.rs:123`, first draft)
   - **What this is:** the new test compares each committed setlist, written back, with the file's JSON.
   - **Problem:** `assert_eq!` on two `serde_json::Value`s prints both on failure, that is every slide's words, into the CI log.
   - **Fix (done):** `assert!(a == b, "<path>: written back differently")`, a yes/no with the path.
   - **If we skip it:** a broken serialiser would paste public-domain lyrics into a CI log, against the records rule.

**Should fix** — none left open.

2. **Credit slide inside the last section** (`importers/txt.rs:67-70`)
   - **What this is:** which section a `Credit:` slide belongs to.
   - **Problem:** the first draft let it join the preceding section; `evidence/mod.rs:218` reads section membership to see where a chorus ends, so the end edge would sit on a slide nobody sings.
   - **Fix (done):** `sections.last_mut().filter(|_| !credit)`: the credit slide is in no section; `credit_slide_kind_detected` asserts it.
   - **If we skip it:** one late "return to slide 1" prior on every song with a credit slide once V11 lands.

**Nice to have** — left as they are, with the reason.

3. **`Credit:` is case-sensitive** (`importers/txt.rs:67`): `credit:` stays a normal slide and the follower may try to match its words. Left: the spec and every kit file write `Credit:`; widening it is a one-word change when a church trips on it.
4. **Seven replays run twice** (`tests/replay_determinism.rs:129-131` and `:49-89`): `old_setlists_replay_unchanged` re-runs `voxxad --replay` on every log the first test already replays, about 7 process starts. Left: it is what the test's name claims, and it survives if the first test is ever split.
5. **`Slide::new` has no doc comment** (`aligner/mod.rs:30`): the field above it says what `kind` is. Left: nothing to add.
6. **`agent-settings.example.md` syntax is `[UNVERIFIED]`** (`kit/templates/agent-settings.example.md:12`): the draft says so itself; checking it against the current Claude Code settings is the booth's job before a church uses it (ADR-029 § 4). Left for V5b/V17.

Verdict: Ship. Lean: 0 lines further; the change removed 26 more lines than it added across the counted rows. Not checked: the GUI import path
in a browser (`src/main.js` collects the parsed songs and calls `load_setlist`); the Rust side is covered by `check_setlist`, which every
section the importer writes passes by construction (ascending, in range, no slide in two sections, none empty).

## 4. Kit: files committed, excluded, fixed

42 files committed under `kit/`; 1 excluded: `templates/congregation-notice.template.md` (FA-70, answered 2026-11-27: no notice at all, so
nothing is drafted; the kit's privacy page still says what voxxa keeps). Checks on the 42: `git grep -nE '"(text|lyrics|slides|words|run)"' -- 'kit/**/*.json'`
empty; `C:\Users` 0; e-mail addresses 0; capital "Voxxa" 0 after the fixes (4 before); the founder's login 0; the scan-terms grep 0 in every
file; regex hits only in the allowed classes (§ 6).

Fixes to the draft (each is one of the brief's "fix what the draft gets wrong"):

| # | File:line | What was wrong | Fix |
|---|---|---|---|
| K1 | `README.md:40-49` | No tier table | Section "Two tiers of presentation app (FA-47)" added |
| K2 | `README.md:9-11` | "draft kit … written 2026-11-24"; tools "built in step V5" | Status says committed by V5a (2026-11-27); the kit tools are step V5b (also `:93`, `:133`, `:180-181`) |
| K3 | `README.md:143-145` | uv minimum "written by V1's PR on this line" | **0.13.10** from FA-03 (2026-11-25) |
| K4 | `README.md:82-87` | The tree omitted four templates in the draft | `incident.template.md`, `minder-role-card.md`, `repertoire-inventory.template.csv`, `rota.template.csv` listed with their steps |
| K5 | `private-folder.md:17`, `prompts/01-intake.md:41`, `skills/voxxa-church-setup/SKILL.md:37`, `templates/church-setup-checklist.md:37` | `%LOCALAPPDATA%\Voxxa\` with a capital V | lowercase, with the "Windows folder names ignore case" note kept where it was |
| K6 | `templates/church-profile.template.json:19` | "(FA-47, proposed)" | "(FA-47, approved 2026-11-26)" |
| K7 | `templates/minder-role-card.md:30`, `:38` | "voxxa only suggests for the rest of that song" and "the demotion rule" | FA-61 (2026-11-27): a manual move re-anchors, never demotes |
| K8 | `templates/bindings/freeshow.json:43-44` | `text_transition_ms: 0` and "a person sets … to 0 ms" | FA-57 (2026-11-27): keep the 500 ms default; value 500, note rewritten |
| K9 | `templates/presenter-test-card.md:62`, `:64`, `:81` | FS10 asks for Presenter's MIDI mapping (FA-53: skipped); FS12 and FS19 ask to set 0 ms (FA-57: keep 500) | FS10 marked skipped with the FA; FS12 records the default only; FS19 confirms the default is still in use |

Not changed on purpose: `rota.template.csv` has two columns more than layout § 3.3 lists (`tapper`, `pressed_record`; V18's file, no names in it);
`songs.template.csv` keeps `Amazing Grace` / John Newton (public domain, allowed) and made-up rows; the FreeShow and OpenLP binding notes keep
their `UNVERIFIED` markers for V3e/V8b.

## 5. Budgets (`python scripts/loc_budget.py`, tokei 15.0.0, every row OK, exit 0 both times)

Before (54d841e): conductor 1173 / 1200 · conductor evidence 215 / 500 · presenters 1396 / 2000 · importers + planning_center 1386 / 1600 ·
everything else 900 / 904 · wiring 1288 / 1438 · discovery 223 / 250 · updater 98 / 150 · voxxad 552 / 600 · recognizer 557 / 825 ·
voxxa-eval 488 / 600 · onset 0 / 750 · input chain 0 / 600 · position 0 / 900 · GUI 2860 / 3000 · Simple mode 450 / 600 · health check 0 / 450 ·
drivers freeshow 128, keystroke 396, openlp_v2 180, opensong 130, propresenter7_rest 169, propresenter7_ws 186 (each / 400) · runtime npm dependencies none.

After (this branch): conductor **1169** / 1200 · conductor evidence 215 / 500 · presenters 1396 / 2000 · importers + planning_center **1370** / 1600 ·
everything else **894** / 904 · wiring 1288 / 1438 · discovery 223 / 250 · updater 98 / 150 · voxxad 552 / 600 · recognizer 557 / 825 ·
voxxa-eval 488 / 600 · onset 0 / 750 · input chain 0 / 600 · position 0 / 900 · GUI 2860 / 3000 · Simple mode 450 / 600 · health check 0 / 450 ·
drivers unchanged · runtime npm dependencies none.

| Row | Before | After | Δ | Share in the brief |
|---|---|---|---|---|
| conductor (`aligner/`) | 1,173 / 1,200 | 1,169 / 1,200 | −4 | ≤ +3 |
| importers + planning_center | 1,386 / 1,600 | 1,370 / 1,600 | −16 | ≤ +40 |
| everything else in `src-tauri/src` | 900 / 904 | 894 / 904 | −6 | 0 (no growth) |
| every other row | unchanged | unchanged | 0 | 0 |

No cap changed; no Python row; no dependency added.

## 6. Gates and the lane scan

Gates (Windows, Git Bash, rustc 1.99.1, `CARGO_TARGET_DIR=C:/dev/v5a-target-fresh`; `SEED-DONE` did not exist when cargo was first needed):
- `cargo fmt --check`: OK (one unformatted test edit was caught by the check and fixed with `cargo fmt` before any commit).
- `cargo clippy --all-targets -- -D warnings` and `cargo clippy --all-targets --features cli -- -D warnings`: OK (the second re-run after the last test edit: OK).
- `cargo test --features cli -j 2 --no-fail-fast`: **271 passed, 0 failed, 1 ignored** in 30 test binaries (before, inferred from the 3 added
  tests: 268 passed, 1 ignored). `importer_slides` 6/6 (4 before), `replay_determinism` 3/3 (2 before), `media_guard` 5/5, `conductor_evidence` 25/25,
  `conductor_unchanged` 2/2, `importer_examples` 4/4. The first full run failed `old_setlists_replay_unchanged` on one fixture: ADR-016 writes a
  `refrain` section back as `chorus`, so the test now compares each song's slides (what V5a changed), not its sections.
- `cargo build --features cli --bin voxxad --bin voxxa-eval`, then `cargo run --features cli --bin voxxa-eval`: 21 of 21 hits, precision and recall 1.00,
  0 false, 0 double, median 2,310 ms, p95 4,870 ms, "OK: no metric is worse than fixtures/baseline.json" (unchanged since Q6).
- `python scripts/loc_budget.py`: exit 0, every row OK, before and after (§ 5).
- The first gate run, in the background, was stopped by the harness for low system memory while the test binaries linked (two lanes build on
  one PC); the runs that count used `-j 2`, as `handoff.md` advises for this PC, and finished.

Media guard: the `media_guard` tests pass on this tree (the bash step from `ci.yml` runs inside `the_ci_step_passes_the_real_tree_and_names_every_planted_file`);
`kit/` adds no media file and nothing under `fixtures/private/`. `npm ci && npm run build` ran once (cargo embeds `../dist`); `npm run test:gui` not run (nothing under `src/` or `tests/gui/` changed).
`cargo deny` not installed here; no dependency changed. `git grep -lIP '[\x00\x0b]'` empty before every commit. `src-tauri/gen/schemas/*.json` were
restored with `git checkout` after each build (#91).

Lane scan, staged files of each commit (counts per file; the scan-terms grep is 0 for every file in every commit):
| Commit | Staged files | scan-terms, whole file | scan-terms, added lines | Regex counts per file (whole file); control bytes 0 at every commit |
|---|---|---|---|---|
| f520bf6 kit | 42 | 0 in every file | 0 | `README.md` 2, `private-folder.md` 9, `prompts/02` 3, `prompts/07` 6, `prompts/08` 10, `skills/voxxa-song-intake` 1, `skills/voxxa-wav-intake` 5, `templates/agent-settings.example.md` 4, `bindings/freeshow.json` 1, `church-setup-checklist.md` 1, `intake.template.json` 1, `lab/pyproject.template.toml` 2, `presenter-test-card.md` 2, `service.template.json` 1, `wav-intake-checklist.md` 16; the other 27 files 0 |
| 32c12ea guards | 2 | 0 | 0 | `.gitignore` 6 (four media types, the private fixtures folder, the public fixtures path in a negation); `CODEOWNERS` 0 |
| 44a1b9a importer | 14 | 0 | 0 | `tests/conductor_evidence.rs` 2 (pre-existing), `tests/replay_determinism.rs` 1 (the public fixtures path); the other 12 files 0 |
| 02f48e3 records | 8 | pre-existing lines only, kept per FA-12 (c): `decisions/sde` 4, `handoff.md` 9, `status.md` 8; the other 5 files 0 | 0 | added lines: this record 7, `decisions/security` 3, `decisions/sde` 1, `handoff.md` 1, the three ADRs and `status.md` 0 (whole file: ADR-026 1, ADR-027 1, ADR-029 10, sde 18, security 13, handoff 12, this record 7, status 4) |

Allowed-class check on the regex hits: media file types in `.gitignore`'s deny rules and in prose about the deny rule; `fixtures/private`
and `recordings/` as path names (`.gitignore`, the public `fixtures/recordings/` path in two tests and this record); `voxxa-church-private`,
`songs\`, `recordings\` as the private folder's path names in kit prose; `127.0.0.1` (2 in `freeshow.json`, 2 in `presenter-test-card.md`);
`password:` once in `presenter-test-card.md` T12 ("never write the password here"); `C:\Users\` only as the deny-pattern text in ADR-029
(pre-existing line). No e-mail address, no other IP, no API token anywhere.

## 7. ADR status changes

| ADR | Was | Now | Founder record cited |
|---|---|---|---|
| ADR-026 | Proposed (2026-11-24) | Accepted (2026-11-27, V5a) | FA-13 (licence policy as written) with FA-01's pre-approval of V5; `founder-action-items.md:239` ("every FA answered"); FA-26 not cited (it does not name the register) |
| ADR-027 | Proposed (2026-11-24) | Accepted (2026-11-27, V5a) | FA-03, done 2026-11-25 (uv 0.13.10 on the booth PC); Decisions 2–4 still land with V1 |
| ADR-029 | Proposed (2026-11-24) | Accepted (2026-11-27, V5a) | FA-01 (pre-approval of V5) and FA-12 (c), both 2026-11-25; Decision 5's scrub is void |

## 8. Skipped on purpose, tech debt, open questions

- **Not in V5a (V5b, deferred behind the first autonomous Sunday, R17):** `kit/tools/songcheck.py`, `kit/tools/kit_check.py`, the media-guard
  and `licensing.rs` extensions, the `budgets` job's `kit_check` line, the kit tools row, the fake second church. Until then the kit's checks are
  the media guard, the lyric-key grep and the lane scan (all run here).
- **Not extended:** `src-tauri/tests/media_guard.rs` and the `media` CI job (the brief says so). The `.gitignore` alone does not stop a
  forced add; V5b's guard does.
- **The follower does not read `kind` or the importer's `sections` yet** (V11); `voxxad --learn` (V12) and the FreeShow group labels (V8b, V17)
  are the consumers. Nothing in the GUI shows sections (as before).
- **Other importers still write `sections: Vec::new()`** (OpenSong `[V1]`/`[C]`, ChordPro, OpenLyrics); ADR-016 names them as a later step.
- **Kit content owned by later steps** was corrected only where it contradicted a recorded founder answer (§ 4 K6–K9); V8b finalises
  `freeshow.json` and the FS rows, V17/V18 the cards and the README's steps 11–12.
- **Open question for the comparator:** whether `Slide::new` (one constructor, 20 call sites) or `kind: None` at every literal with a
  conductor share of about +10 is preferred; the budget made the choice here (§ 2.6).
- **Open question for V5b:** a new public `session-*.jsonl` or a `truth.jsonl` outside `fixtures/recordings/` needs its own `.gitignore`
  negation; the media-guard extension should carry the same allowlist.
