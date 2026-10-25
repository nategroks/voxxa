# Voxxa – instructions for local session (Windows PC)

From: the cloud session on branch `claude/trusting-newton-v71ebu` · Date: 2026-10-25 · Read first: `.10x/handoff.md`, `.10x/specs/2026-10-24-delivery-plan-and-prompts.md` §7, ADR-002, ADR-007, ADR-008.

## Ground rules so we do not collide
- Work on your own branch `local/windows-<topic>` off `claude/trusting-newton-v71ebu`; open PRs into that branch. CI (`.github/workflows/ci.yml`) must be green.
- File ownership: **you** own `fixtures/`, `packaging/`, `docs/support/`, `docs/quick-start.md`, and `.10x/decisions/devops/` + `.10x/decisions/qa/` (append-only). **Cloud** owns `src-tauri/src/`, `src/`, `.github/workflows/`, `.10x/adrs/`, `.10x/specs/`, the other `.10x/decisions/` folders. Ask before crossing.
- Ponytail `full` on; smallest complete change; end each report with what you skipped and any risk.

## Tasks, in priority order

1. **Windows verification of Increment 0** (today). Pull the branch; `npm ci && npm run build`; in `src-tauri`: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, then `npx tauri build`. Install the NSIS `-setup.exe` on the PC as a non-admin user: does WebView2 bootstrap run, does the app open, does the tray icon appear, does the Models tab download `Base`? Record findings in `.10x/decisions/qa/church-autoadvance.md` (create it). Note the exact SmartScreen dialog you see on the unsigned build; we need the wording for ADR-008's risk row.

2. **Fixture recording kit** (before P1.6). Set up `fixtures/README.md` and a tiny recording helper (PowerShell or ffmpeg one-liner) that captures the desk feed or USB interface as 16 kHz mono WAV, plus a key-logger-free way to timestamp slide changes (a second script where the operator presses Enter at each change, writing `truth.jsonl` lines `{t_ms, song, slide}`). The founder records two public-domain hymns (e.g. "Amazing Grace", "Be Thou My Vision") with a lyric `setlist.json` in the v1 schema from P1.3 (sections → slides). Do not commit audio over 25 MB; store in Google Drive `Voxxa/fixtures/` and commit only the JSON and a `SOURCES.md` with the Drive links.

3. **Planning Center sample setlist** (feeds P3.5 and the pilot). Using the Planning Center connector, pull The Lighthouse, Midlothian's next service plan: service type, plan items, songs, arrangements and whether lyrics are present. Save an anonymised `fixtures/pco/plan-sample.json` (strip people) and note which songs carry CCLI numbers and lyrics. This tells us if PCO is a viable setlist source for them.

4. **Pilot church logistics**. Draft an e-mail to The Lighthouse's tech lead asking: presentation software and version, booth PC OS and CPU, how audio reaches the booth, weekly attendance band, who operates slides, and consent to record one service (desk audio only, stays on their machine; see research plan §6). Put a 30-minute call on the calendar with the founder. Create the Drive folder `Voxxa/pilot-lighthouse/` with the consent text from the research plan.

5. **Signing and Store paperwork prep** (unblocks Increment 2). Draft `docs/signing-policy.md` (SignPath Foundation requires a public code-signing policy page: who may sign, which branch, how releases are tagged, how keys are stored). List the Partner Center individual-account steps and what identity documents are needed; do not submit anything without the founder.

6. **Quick-start card draft** (P2.5, later). One page in Canva: install → first run → load setlist → switch to Songs → what the colours mean → Call for help. Export PDF to `docs/`. Wait until P2.1 lands so the screenshots are real.

## Report back
Commit notes to `.10x/decisions/qa/` or `.10x/decisions/devops/` and push your branch; the cloud session reads them at the start of each increment. If something blocks you, write it in `.10x/status.md` under Blocked.
