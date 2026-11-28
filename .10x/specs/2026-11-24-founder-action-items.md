# Founder action items (CTO/CEO), deduplicated and ordered by urgency

Date 2026-11-24. One numbering (`FA-nn`) replaces the drafts' overlapping lists (A 1–8, B D1–D8, C D1–D14, UI
F1–F12, ponytail 1–14, safety SF1–SF11, feasibility 1–5). Each item: what, options, **recommendation**, why,
what it blocks. Answer with "FA-03 yes", "FA-14 option B", or "all recommendations". Items already decided in
the brief (§ 2) are not re-asked. FA-31 … FA-37 were added after the critic's round 1 (`adjudication.md` § 8);
nothing was renumbered, so each new item sits in its urgency section out of numeric order. FA-38 … FA-42
(2026-11-26) come from the onset-trigger lab work (ADR-030; `plan.md` V3b, V3c, F4). FA-43 … FA-45
(2026-11-26) come from the first live Presenter test and the parts map (ADR-031; `plan.md` V3d). FA-46 … FA-72
(2026-11-26, draft) come from the round-2 autonomy review (`round2/50-adjudication.md` § 5; section A2 below), and that round marked FA-10, FA-29, FA-38 (b), FA-39, FA-43, FA-44 and FA-45 urgent.

**Answers so far (2026-11-25):**

- **FA-01:** approved in full: the #59 fix, plus pre-approval for V0, V1, V2 and V5.
- **FA-05:** the two-number definition is approved.
- **FA-03:** done. uv 0.13.10 is on the booth PC and becomes the minimum version in `kit/README.md`. The lab
  template's `exclude-newer = "7 days"` works with it.
- **FA-04:** done.
  - Vulkan SDK 1.4.363. The GTX 960 runs driver 560.94 with Vulkan 1.3.280.
  - A local GPU build shows Small at 0.96 s per pass with the 6 s encoder (CPU: 2.95 s).
  - The GTX 960 also drives the projector. The founder considers this no concern because Presenter owns the
    displays; watch for projector stutter in the first live GPU test.
- **FA-12:** (c). Leave the existing mentions; there is no scrub PR. The deny-file scan gates only `kit/`, so plan
  V5 step 6 is dropped.
- **V0:** done 2026-11-25. PR #186 merged as `a5f7e2e` and closed #59. The booth timing is 138 s (target 150 s or less; it was 461 s). The check runs end to end at the next founder-approved rc.
- **FA-06:** the founder records only their own voice first; the worship team joins later. The private folders
  (`pilot`, `lab`, `fake-church-2`) and three public-domain hymns are set up for that test.

---

## Answers recorded (founder, 2026-11-26, batch 1: FA-10, FA-38 … FA-49)

The founder's answers, kept to their wording and intent. Items not listed here stay open. Two answers (FA-38, FA-39)
deviate from the recommendations; their effect is carried into `plan.md` (V3b, V3d, F4), `plan-round3.md`, ADR-030
amendment 1, ADR-031 amendment 2 and `spec.md` § 4.1 the same day.

| FA | Answer | Effect on the plan |
|---|---|---|
| FA-10 | **Approved** (later the same day): one 150-line increase as a recorded "wiring" row, option (a). | V7 may run: the ADR-028 amendment adds the wiring row (measured size + 150) and lowers "everything else" by the same size; caps still only go down for every existing row. V7 also carries FA-54 (new rows) and FA-65 (the `trace` line), both answered 2026-11-27 (batch 2 below); V7 runs in full. |
| FA-38 | **Deferred.** No USB interface for some time. A noisy, low-level, unbalanced feed is a supported, if degraded, condition; the plan includes fault tolerance for poor audio and never blocks on the current booth feed. | R6 "hardware first" is withdrawn. The input chain keeps the gain, high-pass and hum paths (V3b task 6) and adds a numbers-only `degraded` flag on the level line; the onset trigger disarms while it is set and words move the slide. The cheap isolator (option a) stays optional; nothing waits for it. |
| FA-39 | **Must work with music present.** A vocal-only aux is not a hard requirement; the onset trigger and the voice path function, with reduced performance where needed, when the full mix is on the feed. | Band and full-mix takes on the current feed are a required acceptance row (the degraded-mode row) in V3b and V3d; the bar there is "never worse than words only, 0 onset fires on band energy". ADR-030 § 5 and spec § 4.1 amended. |
| FA-40 | **Approved**, option (a): ADR-030 as the direction for V3b, and supervised live Presenter tests with the lab helper under task 5's rules. | V3b and F4 may run. |
| FA-41 | **Yes**, option (a): the founder builds the It Is Well song in Presenter to match the kit file when that step is reached. | F4's precondition is met when it is built; the slide count is confirmed then. |
| FA-42 | **Approved**: the lab-only Whisper-prompt experiment. | V3c may run under its R7 condition (Small > 1.0 s/pass). |
| FA-43 | **Approved**, with an addition: the framework traces song parts from metadata and applies the parts-map follower on both Linux and Windows booth PCs. | ADR-031 stands as Proposed with amendment 1 (R3). The Linux requirement is new: recorded as **FA-73** below and in ADR-031 amendment 2. |
| FA-44 | **Proceed** with the FreeShow trial path, option (d), and suggest-only on Presenter, option (e), until a jump is proven. | As already recorded. What the founder provides is the F5 checklist plus FA-53's 20-minute MIDI check (see "What the founder does next"). |
| FA-45 | **Yes**: the founder records the needed takes alone for now, so live4's truth and ADR-031's numbers can be finished. | Tap live4 (5 min). New solo takes count as founder takes in V3b task 4 and V3d task 4. |
| FA-46 | **Yes**: FreeShow stable installed beside Presenter on a weekday before Dec 18. | F5 and V3e may run. |
| FA-47 | **Approved**: the two-tier app policy. | ADR-032's Tier A / Tier B stands. |
| FA-48 | **Yes**: the Sunday-output switch gate as written; the switch is scheduled when the prerequisites are ready. | ADR-035's gate stands; the decision date (end of January) is unchanged. |
| FA-49 | **Approved**: the budget gates as written. | V3e ≤ $20 now; V8b ≤ $60 on a V3e go; no aligner, tracker or model spend before V6 reports and the GPU is measured with both apps open. |

**The two deviations (FA-38 + FA-39), as the founder stated them.** The lab, V3b and V3d keep a **noisy full-mix path**
alive and measure it, not only the clean vocal-only case. The earlier solo-voice gains stand, but the acceptance criteria
and the input chain now carry an explicit degraded-mode story: detect the condition from numbers (floor, hum share, band
energy before an onset candidate), fall back to words-only automatically, report the degraded rows per take. The USB
interface is off the critical path for the next few weeks; the isolator remains optional. Everything else matches the
prior recommendations and moves forward as scoped.

**What the founder does next (from these answers).** (1) FA-10 is approved, so V7 can start; FA-54 and FA-65 (answered 2026-11-27)
complete V7's scope, so V7 runs in full. (2) Tap live4's truth, 5 minutes (FA-45). (3) F5 on a weekday before Dec 18 (FA-46):
install FreeShow stable, auto-update off, API password, firewall 5505/5506 to loopback, the It Is Well test show as a
unique layout with groups V1–V4 and C, note the text-transition default, paste version and numbers; the exact checklist
is `plan-round3.md` F5. (4) FA-53's MIDI check in Presenter, 20 minutes, inside the V3e run. (5) Build the Presenter
It Is Well song when F4 is reached (FA-41). (6) Record the solo takes (FA-45) and, now that music present is in scope,
one take singing over a backing track on the current feed for the degraded-mode row.

**Still open after this batch:** FA-29, FA-50 … FA-72 and FA-73 (new, below); FA-54 and FA-65 were answered the next day (batch 2 below).

**FA-73 — Linux booth PCs for the parts-map follower (new, from FA-43's answer).**
- Context: the follower, the parts map and `voxxad` are Rust and OS-neutral; the FreeShow HTTP driver (Tier A) is
  OS-neutral; the keystroke driver (Tier B, WorshipTools Presenter) is Windows-only, and Q26 already disables keystrokes
  under Wayland. The kit today says "Windows booth only" until Q24/Q26 land. Parts already come from metadata: setlist
  `sections` from the `.txt` importer (ADR-016, V5), FreeShow groups (V17) and practice files (V12).
- Options: (a) Linux is Tier A only (FreeShow over HTTP), covered by the existing three-OS CI matrix plus one Linux smoke
  run of `voxxad` and the follower tests before V11 closes; no Linux booth before V15; (b) also port the keystroke path
  to X11 (a new row, outside the budget wall); (c) Linux after Easter.
- **Recommend (a).** Why: it costs no new row; Tier B has no deterministic jump anyway, so a Linux booth would run
  FreeShow. Blocks: V11's driver-interface wording; the kit's "Windows booth only" line (V17).
- **Answered 2026-11-27: (c), Linux after Easter.**

## Answers recorded (founder, 2026-11-27, batch 2: FA-54, FA-65)

Both answers complete V7's scope. FA-54 deviates upward from the recommendation; its effect is carried into `plan.md`
V7, `plan-round3.md` (V7, V3b, V3d, V11, V13s), ADR-028 amendment 1, ADR-031 amendment 1's row numbers, ADR-034 § 11
and `spec.md` § 9 the same day.

| FA | Answer | Effect on the plan |
|---|---|---|
| FA-54 | **Yes**, with a rule: "take the number you need and multiply times 3"; the founder wants this done comprehensively and exceedingly well. | The new rows land at three times the engineers' estimates: `onset` 750 (estimate 250), `input chain` 600 (estimate 200), `health check (src/health/)` 450 (estimate 150). The `vad/` move is unchanged. The conditional `align` row, if R5 earns it, gets 1,200 under the same rule; the `readiness` row proposed in ADR-034 § 11 gets 900 if the founder says yes to it at V13s. Caps still only go down once set; no existing cap is raised. Whether the rule also covers the rows ADR-028 proposed before FA-54 was asked (follower, practice, layout, Run, kit tools) is a new item, **FA-74**, because tripling the follower and practice rows breaks the ≤ 3,000-line core-engine bound. The quality bar is recorded in V7's prompt: the amendment documents every row, the estimate beside the cap, the measured numbers at merge and the reason for the headroom; nothing is left "to be decided later". |
| FA-65 | **Yes**, option (a): one `trace` JSONL line type with a `kind` field. | The ADR-001/003 amendment is written in V7 task 6 as drafted; replay skips the line like `window`; a `kind` that carries text fails the serialisation test. V3b (`level`), V8b (`verify`, `screen`), V13s (`watchdog`, `operator`) and V11 (`follow`) add their field lists inside the amendment's table. |

**V7 after this batch:** FA-10, FA-54 and FA-65 are all answered, so V7 runs in full (wiring row, the three new rows at
the tripled caps, the `vad/` move, the `trace` line, the share tables). Nothing is committed until the founder says go.

**Still open after this batch:** FA-29, FA-50 … FA-53, FA-55 … FA-64, FA-66 … FA-72, FA-73, and FA-74 (new, below); most of these were answered the same day (batch 3 below).

**FA-74 — Scope of the "× 3" rule (new, from FA-54's answer).**
- Context: FA-54 named three rows (onset, input chain, health check); they are now 750 / 600 / 450. ADR-028 Decision 2
  had already proposed five rows before FA-54 was asked: follower ≤ 500 (1.3 × the measured port), practice 300,
  layout 300, Run screen 350, kit tools 300. ADR-028 § 4 keeps the core engine (conductor 1,200 + evidence 500 +
  follower 500 + practice 300 = 2,500 in caps) under the strategy's ≤ 3,000-line bound
  (`.10x/specs/2026-10-24-software-strategy.md:30`). Onset and the input chain are counted as the audio front end,
  outside that sum, the way layout is "integration, not core"; the health check is JS. Tripling follower and practice
  (1,500 + 900) would put the core sum at 4,100.
- Options: (a) the rule is standing: every row not yet in `loc_budget.py` lands at 3 × its estimate (follower ≤ 1,500,
  practice 900, layout 900, Run 1,050, kit tools 900) and the strategy's core bound is raised to 5,000 in the same
  ADR; (b) the rule covers FA-54's three rows now; the five earlier rows keep their caps, and a step whose measured
  code cannot fit stops at its Phase 2 check and asks for 3 × the measured number for that row only; the core bound
  stays at 3,000; (c) the rule also covers the two non-Rust rows (Run 1,050, kit tools 900); the Rust core rows as in (b).
- **Recommend (b).** Why: the 3,000-line bound is what keeps the engine readable by one reviewer and replay
  byte-identical; the follower port's real size is unknown until V6 measures it, so a tripled cap now is a guess
  tripled, while (b) gives the same "× 3" answer at the moment a step has a measured number. Blocks: nothing this
  week; V11 (follower cap), V12 (practice) and V14 (Run) at their Phase 2 checks.
- **Answered 2026-11-27: (b).**

## Answers recorded (founder, 2026-11-27, batch 3: FA-29, FA-50, FA-52, FA-55 … FA-61; FA-62 … FA-64 asked again in plain words)

Four answers deviate from the recommendations (FA-50, FA-57, FA-59, FA-61) and one adds a fact that changes the
migration gate (FA-29). Their effect is carried into `plan-round3.md` (F5, V3e, V13s, V14, V15), ADR-032 (Decisions
13 and 14), ADR-034 § 25, ADR-035 amendment 1 and `spec.md` § 7 the same day. FA-55's configuration steps are in
`out/founder-freeshow-setup.md`.

| FA | Answer | Effect on the plan |
|---|---|---|
| FA-29 | **Yes.** All Presenter output is streamed through the Blackmagic (ATEM Constellation) and streaming devices and appears on YouTube as a lower third. The founder calls this critical. | (1) Every slide move is visible to the stream audience, not only the room: the gates' "0 early beyond the line, 0 spurious presses" now protect the stream too, which supports the founder's "extensive listening period" (FA-52). (2) voxxa's own stage strip (FA-60) and its window are never on a display the Blackmagic chain captures; the service-day checklist gets that line. (3) The FreeShow migration needs the same lower-third path (fill and key toward the Blackmagic chain) or the switch cannot happen: new **FA-75**, checked as FS13 in F5/V3e and added to ADR-035's gate. |
| FA-50 | **Use whatever licensing is required** to bring the Python work and all software into the system; "at this point it doesn't matter to me". The founder would prefer, later, a licence of their own, MIT-like with a confirmation of ministry use: "we aren't there yet". | No licence gate blocks the lab or V3f now. ADR-005's amendment becomes an **inventory**, not a gate: every dependency, model and aligner is recorded with its licence, and anything non-permissive (CC-BY-NC, copyleft) is flagged so the later own-licence decision can be reconciled. The own licence is parked as **FA-76** (not blocking). One caution recorded, not decided: an MIT-like licence with a ministry-use clause is source-available, not open source, and cannot sit over a copyleft (GPL) dependency; permissive upstreams keep that door open. |
| FA-52 | **Yes**: two shadow Sundays; the listening period will be extensive; it is important that voxxa does not interfere while the volunteer is working. | Gates unchanged (ADR-034 § 10). Added to V13s and V14: in shadow mode voxxa shows one line ("Shadow today: voxxa listens and logs, moves nothing."), raises no card, no sound and no window over the slides app, and sends nothing; the minder sees it only by opening voxxa. The shadow period runs as long as the founder wants; gate 1 is a floor, not a trigger. |
| FA-55 | **Yes**, option (a): refuse an API that answers without a password. "Let me know what configuration steps are needed." | The steps are in `out/founder-freeshow-setup.md` (one page: install stable, API password, loopback firewall, auto-update off, Planning Center connection off, the test show, the lower-third and shortcut checks, what to paste back). voxxa's `require_password` default stays `true`; nothing to set on your side for it. |
| FA-56 | **Reframed by the founder**: the important thing is a metadata / object-notation file for the layout of each song as it is loaded into the presentation software, as Proclaim and ProPresenter do with JSON; how do we do it most efficiently with Presenter and with FreeShow? | Recorded as a requirement (ADR-032 Decision 13 as amended): the object of record is a **per-song layout file in JSON** in the church folder: sections, slide order, group labels, slide counts, the app's ids; never words. **FreeShow:** its show file is already JSON; voxxa derives the layout file from the show through `get_show`'s id-and-name parser (ADR-032 Decision 11) in V17, and `songcheck` compares it with the kit `.txt`. **Presenter:** no API and no documented export, so the layout file is built from the kit `.txt` import (sections and slide splits, ADR-016) and checked against Presenter's slide count at F4; F4 also looks for a readable song store in Presenter's data folder (yes/no only) so a direct read can be added if one exists. Unique layouts stay the v1 rule for FreeShow because the jump needs an index; the layout file is what a later occurrence resolver reads. The answer to "how" for each app is in the reply of 2026-11-27 and in ADR-032. |
| FA-57 | **Keep a transition that follows human reading time**; if FreeShow's 500 ms fade is that, keep it. | Deviation from "0 ms": the followed style keeps FreeShow's 500 ms default. F5's FS12 becomes a note, not a setting. The verify read-back is unaffected; the visible change lands about 0.25 s after the command, which the eval notes beside the on-time bar. ADR-032 Decision 14 reworded. |
| FA-58 | **Approved**, on condition that the controls never interfere with the presentation software's usual hotkeys. | F13, F14, F15 and F20 were chosen because they are absent from normal keyboards and unused by Presenter and FreeShow by default (ADR-024 § 6). The condition becomes a checked row: FS14 in F5/V3e compares voxxa's keys with FreeShow's shortcut list and Presenter's documented keys; any collision moves that key. |
| FA-59 | **Not now.** | The Companion / phone veto page is deferred; HOLD is at the PC. V15's worship-leader card loses the veto line; spec § 7 and ADR-034 updated. |
| FA-60 | **Proceed** with the fix. | The heard-words line is removed and the three-state strip is voxxa's own window, on a display the stream never captures (FA-29). Ticket now, fix in V14, as planned. |
| FA-61 | **Disagree**: voxxa should be dynamic enough to keep going when a volunteer moves a slide; the badge should show that voxxa is live and will move slides from what it hears. | Deviation: no demotion on a manual move. New rule (V14 task 1c, ADR-034 § 25, spec § 7): a manual Next or Back **re-anchors** the follower. On Tier A (FreeShow) the next read-back gives the operator's slide, the follower continues from it with its evidence reset, and voxxa sends nothing for a short grace (2 s, a first guess in `FollowerConfig.manual_grace_ms`) so it never fights the volunteer. On Tier B (Presenter) voxxa is suggest-only anyway (FA-44 (e)) and the suggestion restarts from the counted position. The badge keeps "voxxa moves it" (live); the grey sentence becomes "You moved it. voxxa is following from here." The old demote rule stays as an off-by-default switch for a church that wants it. Detail added the same day from the lab review (ADR-031 amendment 3): at a re-anchor the words heard before the move are dropped from the evidence, so the words that caused a wrong jump cannot repeat it after the grace; on Presenter a press on its own keyboard is invisible to voxxa, and the cost is a wrong suggestion card, never a wrong screen. |

**Asked again in plain words (still open):**

**FA-62 — The sentences on the pre-service check page.** Before a service, a volunteer opens voxxa and sees one page with
six lines: sound, slides app, today's songs, practice memory, the HOLD test, the log. Each line is a colour and one short
sentence, for example "Hearing the singers' mic. Level good, line clean." or "FreeShow is not answering. Open it, then
press Check again." The top line is one of "Ready for today", "Ready, with a note", "Not ready". The question is only:
do you want to approve those sentences word for word (they are in `round2/44-ui-design.md` § 2.2, and V10 shows them
on a mock screen first), and do you agree that "Ready, with a note" still lets the volunteer start, while only "the
slides app is not reachable" blocks starting? **Recommend yes to both.** You can also answer "approve them when I see
the mock", which is the V10 step.

**FA-63 — The "break gate".** The parts-map follower jumps between parts of a song (verse, chorus, bridge). Singers
usually take a breath or the band plays a short gap at the end of a part. The break gate is a rule that says: "accept a
jump quickly (after one agreeing window) if the words started right after such a gap; otherwise wait for two agreeing
windows." It can make jumps faster at part ends but could also make a wrong jump faster. The question is: may it be
built as a **switch that is off by default**, measured in the lab on the recorded takes (before and after), and turned on
only if the numbers say it helps? **Recommend yes.** Nothing changes on a Sunday until the numbers are in.

**FA-64 — Per-leader timing and leader names.** Different worship leaders start lines earlier or later than the plan.
voxxa could learn a small timing adjustment per leader, but only after at least three confirmed practice runs with that
leader, so one odd rehearsal never changes Sunday behaviour. Leaders are stored only as aliases (L1, L2) in the practice
files, never by name. And voxxa never assumes the leader will follow the planned order exactly (the lab measured that
assumption as worse). The question is: do you agree with those three rules? **Recommend yes.**

**Still open after this batch:** FA-51, FA-53, FA-62, FA-63, FA-64, FA-66 … FA-72, FA-73, FA-74, FA-75 (new, below), and
FA-77 (batch 4 below). FA-76 is parked until the founder wants it.

**FA-75 — The stream's lower-third path must survive the FreeShow switch (new, from FA-29's answer).**
- Context: Presenter's output reaches YouTube as a lower third through the Blackmagic chain. ADR-035 switches the live
  output to FreeShow in 2027. If FreeShow cannot feed that chain the same way (fill and key, or whatever the Blackmagic
  setup takes today), the switch would silently drop the lower third.
- Options: (a) add it to ADR-035's switch gate: FS13 in F5/V3e records which output FreeShow offers toward the chain
  (NDI, a second display, with or without a key/alpha layer) and the Blackmagic input in use today; a dark-Sunday
  rehearsal of the lower third with FreeShow on the booth monitor only; the switch needs a "lower third matches" yes;
  (b) decide after the dark Sundays; (c) keep Presenter for the stream and FreeShow for the room (two apps, C12 says no).
- **Recommend (a).** Why: it is one row in a checklist the founder is already running, and it turns a hidden risk into
  a measured gate. Blocks: nothing before F5; ADR-035's switch decision.
- **Answered 2026-11-27: (a).**

**FA-76 — A licence of voxxa's own (parked).** MIT-like with a confirmation of ministry use. Not before the product
ships; needs counsel; the licence inventory from FA-50 is its input. No action now.

## Founder direction recorded (2026-11-27, batch 4): re-anchor for every presentation software on the market

**The founder's words, kept:** every presentation software on the market must re-anchor consistently and smoothly,
however each one does it; this is critical and most development effort should be spearheaded here; the measures are
reaction time, interoperability, interface, security, and a Unix / suckless philosophy; do what needs to be done,
avoid bloat, get it done in the most effective way.

**What it changes.** Re-anchor today exists only where an app answers "which slide are you on?" (FreeShow, ProPresenter,
Holyrics, OpenLP, Quelea). WorshipTools Presenter, MediaShout, SongShow Plus and VideoPsalm have no API, and a keyboard
hook was refused in round 2 as keylogger-shaped. The proposal that meets all five measures is **ADR-036** (draft): one
position interface; a read-back adapter as data for every app with an API; and, for every app, a position read from
the output display itself, reduced to 64-bit frame hashes learned at rehearsal and from voxxa's own moves (numbers
only, no words, no pixels, no hook). A new plan step **V8c** follows V7 and runs before the FreeShow driver. The
decision the founder must make is **FA-77** (asked as multiple choice the same day). **Revision 1 (same day):** the lab's hash probe measured that a 64-bit whole-frame hash cannot tell lyric slides apart (two distinct lower-third slides hashed identically); ADR-036 and V8c now use a 256-bit text-region hash with a dead zone, a two-frame settle rule, an attribution window after voxxa's own moves, a `blank` class, learning only with follower agreement, the key hook as a hint only, and a booth probe (task 0) before any build, because the Blackmagic chain may take the output from a capture card that the desktop cannot see. Re-anchor delay becomes the app's transition plus about 200 ms.

**FA-77 — The universal position source for apps without an API.**
- Options: (a) output-frame hashes plus read-back APIs where they exist (ADR-036 as drafted; no hook; about 100 ms;
  one new row `position` at 900 under FA-54's rule); (b) a navigation-key hook that sees Next/Back only while the
  slides app is in front (tiny and instant, but hook-shaped: antivirus flags, the round-2 review refused it, and it
  cannot see a mouse, a remote app or a pedal); (c) both, hashes as the universal source and the hook as an optional
  accelerator later; (d) APIs only, apps without read-back stay suggest-only (the plan before this direction).
- **Recommend (a).** Why: it is the one source every app shares, it keeps the security story simple (no injection into
  another process, nothing stored but numbers), and it is small (a 30-line hash plus the OS capture already linked).
- **Answered 2026-11-27: all three, in the priority order hashes 1, navigation-key hook 2, APIs 3** (batch 5; ADR-036 Decision 5 reworded).

## Answers recorded (founder, 2026-11-27, batch 5, by multiple choice: FA-51, FA-53, FA-62 … FA-64, FA-66 … FA-75, FA-77)

Seven answers deviate from the recommendations (FA-53, FA-63, FA-67, FA-69, FA-70, FA-73, FA-77's order); FA-68 and
FA-72 are deferred by the founder; FA-71 raises a cap with the number still to come. Carried into `plan-round3.md`
(V3e, V3d, V11, V13s, V15, V18, V8c), ADR-028 amendment 1, ADR-031 § 3 and amendment 2, ADR-035 amendment 1, ADR-036.

| FA | Answer | Effect on the plan |
|---|---|---|
| FA-77 | **Priority order: frame hashes 1, navigation-key hook 2, APIs 3.** | ADR-036 gains a third source, `keys`, built second: a hook that sees navigation keys only (Right, Left, Up, Down, PageUp, PageDown, Space, Enter), only while the slides app is the foreground window, keeps counts only, and is off when Songs is off; its threat note names the antivirus and trust cost the round-2 review raised, and the Security role signs the scope. `screen` (hashes) is built first; `api` third (FreeShow's read-back in V8b stays, as the first `api` entry). V8c's acceptance lists the three in that order. |
| FA-51 | **Yes**, pin FreeShow. | As recorded (stable channel, auto-update off, version in the private profile, health check warns on change, weekday upgrades). |
| FA-53 | **Skip** the MIDI check. | FS10 and the Presenter MIDI check leave V3e; the "wait for WorshipTools" option is closed by this answer instead. |
| FA-62 | **Approve when I see the mock** (V10). | V10 shows the page on a mock first; the sentences are approved or edited there. "Ready, with a note" may start; only an unreachable slides app blocks. |
| FA-63 | **Build the break gate on by default.** | Deviation from "off, measure first": `PartsConfig.break_gate` defaults to on in V11; V3d still measures before/after on the detour, spoken-aside and two-singer takes, and a measured loss there turns the default back off by a dated ADR-031 amendment with the numbers. ADR-031 § 3 reworded. |
| FA-64 | **Yes** to all three rules. | As recorded (per-leader timing after ≥ 3 confirmed runs; aliases L1/L2 only; plan order is a hint). |
| FA-66 | **Yes**: read-back as shadow truth, tapping one Sunday in four. | As recorded. |
| FA-67 | **Yes, and auto-start voxxa.** | Deviation: voxxa may start with Windows on service days; Songs mode is still armed by a person and the health check still runs within 30 minutes before the service; no remote desktop Sat 18:00 to Sun 14:00; Windows Update active hours and auto-restart off. The runbook `sunday-window.md` and ADR-034 § 6 reworded; the auto-start must not open the mic (standing rule: listening starts only when a person arms Songs). |
| FA-68 | **Later**: decide at the end of January with the FreeShow switch. | V17's dependency on FA-68 stands; nothing migrates before the sign-off question is answered. |
| FA-69 | **Fewer minders is fine.** | One or two minders may start; the founder may stay in the rotation until there are more; gate 2's "two volunteers passed the card" stays as the no-minder condition. V18's rota wording relaxed. |
| FA-70 | **No notice at all.** | Deviation: no congregation notice is drafted and nothing is said from the stage; V18 drops the notice item. Recorded, not argued: the kit's privacy page still states what voxxa does and does not keep (no audio, no words), for any church that wants to say it. |
| FA-71 | **The founder can give more time.** | The caps become 90 min per scored Sunday and 4 h per week (number given in batch 6). V15's close line reworded. |
| FA-72 | **Show me the list first.** | The R17 list is in `out/founder-r17-list.md`; answered per line in batch 6. |
| FA-73 | **Linux after Easter**, option (c). | Nothing Linux-specific before the FreeShow switch; the three-OS CI stays as it is; the Linux smoke run leaves V11; ADR-031 amendment 2 and the kit's "Windows booth only" line stand until then. FA-43's Linux requirement is deferred, not dropped. |
| FA-74 | **Only on a measured overrun**, option (b). | The five earlier rows keep their caps; a step whose measured code cannot fit stops at its Phase 2 check and asks for 3 × the measured number for that row; the 3,000-line core bound stays. ADR-028 amendment 1 item 6 closed. |
| FA-75 | **Make it a switch-gate row**, option (a). | ADR-035 amendment 1 confirmed: FS13 in the setup page, one dark-Sunday lower-third rehearsal, the switch needs a "matches" yes. |

**Still open after this batch:** FA-72 (after the R17 list) and FA-71's number; both answered later the same day (batch 6 below). FA-76 is parked.

## Answers recorded (founder, 2026-11-27, batch 6: FA-72 per line, FA-71's number)

| FA | Answer | Effect on the plan |
|---|---|---|
| FA-72 drops | **All four approved**: V6b, Companion as code, metric renaming; the **Holyrics driver** with a condition: hold off unless confidence is high that it is a twin of ProPresenter or FreeShow, and even then not as an integration against the Unix philosophy; open to it only if there is a better way for Holyrics to listen to voxxa than the other apps. | Dropped as a driver. Under ADR-036 a Holyrics path would be one `api` bindings file (data: `GetCurrentPresentation` read-back and `ActionGoToSlideDescription` jump by section name), not code; it is written only if V3e's FreeShow probe fails or a church asks, and then as that file. |
| FA-72 deferrals | V13b (cue engine), V5b/V5c (second-church kit tools), V8 steps 2–4 (burst keys, per-app key profiles) and the product aligner: **as recommended** (after the first autonomous Sunday; V8 steps 2–4 after V8c proves re-anchor; the aligner lab-first, product only on a measured pass). **V16 (rename and icon): do it now.** | V16 moves into the current wave as a small step (installer, updater and docs touched); the others stay deferred with their triggers. |
| FA-71 | **90 min per scored Sunday, 4 h per week.** | V15's close line and the kit's "founder's week" use these numbers. |

**Two items are open: FA-78 and FA-79 (batches 7 and 8, below).** Every FA from FA-01 to FA-77 is answered or closed; FA-76 (voxxa's own licence) is parked by the
founder's choice. The commit gate in `process.md` is now the founder's "go" on committing the round-3 outputs.

**Lab report (same day, numbers in `reviews/2026-11-27-lab-r14-reanchor-lag.md`).** The R14 fixes, the re-anchor hook
with 18 tests and the capture probe for V8c task 0 are done in the private lab (nothing committed). One finding changes
a number the plan assumed: the live path reads words **2.15 s** after they are sung, not 1.0 s, so every offline result
was optimistic by about 1.15 s; the recogniser pass itself is about 0.5 s, so about 1.6 s sits in the hop wait, the
pipe, the log flush and the read. Because reaction time is the founder's first measure, V3b gains **task 8, the
lag-breakdown row**, and the fix is proposed from its numbers. No founder decision is needed for it.

## Open item (2026-11-27, batch 7): FA-78, the follower row's cap, from the V3d port plan

**FA-78 — How large may the follower row be?** The lab's port plan (`reviews/2026-11-27-v3d-port-plan.md`) measured the
size factor from lab code to compiling Rust at 2.37 (104 lab code lines became 246 tokei lines) and estimates the
parts-map follower at **about 860 Rust lines (765–955)**, over ADR-028's 500 ceiling for that row. FA-74 (b) lets a
measured overrun ask for 3 × at the step's Phase 2 check; 3 × here is about 2,580, and the core-engine sum in caps
becomes about 4,580 against the 3,000 bound that ADR-028 amendment 1 § 6 keeps. One of the two has to give.
- Options: **(a)** 3 × the measured port (about 2,580); the 3,000 core bound moves. **(b)** 1.3 × the measured port
  (about 1,120; ADR-028's original row formula without the 500 ceiling); core sum about 3,120. **(c)** as (b), and the
  `evidence/` row (215 / 500 today: the Q6 position tracker the parts map replaces) retires once the follower's default
  becomes `Some`; core sum about 2,620, inside the bound.
- **Recommend (c).** Why: it holds the measured estimate with about 17 % over the top of its range, keeps the core bound
  the strategy set, and removes the second follower instead of carrying two (the suckless rule in the founder's
  direction). If V11's Phase 2 count still overruns 1,120, FA-74 (b) applies again with the number in hand.
- Blocks: V11's Phase 2 check (the row is added in V11's first commit). Nothing before it.
- Recorded from the same plan, no decision needed: the V11 prompt's test `break_gate_off_by_default_changes_nothing` is
  renamed `break_gate_none_keeps_two_windows` (FA-63: the default is on), and `BreakGate` carries `grace_ms`,
  `min_gap_ms`, `rate_lo`, `rate_hi` (the lab's parameters; there is no `delta`). The evidence row's "may shrink,
  never grow" line gets one exception asked at V11: about 4 lines for `PreChorus` and `Ending` in `sections.rs`
  (ADR-031 § 1). Five findings for V11 (no `dr_show` in the lab; no follower row in `loc_budget.py` yet; no audio time
  in `Conductor::apply`; the `SectionKind` variants; two slide-id spaces) are in the plan's § 1. The lab closes the
  `dr_show` arm, the eleven ADR-031 pytests still unwritten, and the reference move lists before V11 (numbers only).

## Open item (2026-11-28, batch 8): FA-79, the voxxa-eval row and `--service-report`

**FA-79 — Where does `--service-report` live, and may the voxxa-eval row grow?** Step V2 was prompted at ≤ 105 lines for the
voxxa-eval row (488 / 600): columns and helper ≤ 45, `--service-report` ≤ 35, `--truth-from-screen` ≤ 25. Both lanes built the whole
scope and measured it (tokei 15.0.0, after rustfmt): the 10x lane **774 / 600** (`main.rs` 254, `score.rs` 374, `report.rs` 93: +286),
the ponytail lane **about 641 / 600** (591 for the columns, `--actions` and `--truth-from-screen`, the leanest `--service-report` about
50 more). The prompt's shares were about 2.7 × short because rustfmt's 60-character chain and call widths break most one-line iterator
chains and calls into three to six lines (`reviews/2026-11-28-v2-10x.md` § 3, `reviews/2026-11-28-ponytail-v2.md` § 6). Merged (PR
#194, `reviews/2026-11-28-v2-comparator.md`): the nine columns, the helper, `--actions` and `--truth-from-screen` at **594 / 600**
(6 left); `--service-report` is written, tested and measured on the 10x lane's local branch (`v2/10x-full-held`, never pushed) and
described in its record § 6, not merged. The row is not one of FA-74's five, so FA-74 (b)'s "ask at the Phase 2 check" needs the
founder's line here rather than a lane's.
- Options: **(a)** raise the voxxa-eval cap to **750** = 600 + 3 × the measured 50 (FA-74 (b), a measured overrun; the first
  existing-row raise under ADR-028 amendment 1's "caps only go down", on a tools row outside the 3,000-line core bound). **(b)** a new
  module `src-tauri/src/cli/voxxa-eval/report.rs` with its own row "service report" capped at **150** (3 × 50, FA-54's rule for new
  rows); the voxxa-eval row keeps 600, and the flag's dispatch lines in `main.rs` (about 5) still come out of its 6. **(c)** defer
  `--service-report` to **V13s**, whose title already carries "per-service report" and whose inputs (the `level` trace, the service
  manifest, dispatch timing `t_decision` / `t_sent` / `t_ack`, the SLO table's hold, panic and override counts) do not exist before
  V3b, V13a and V13s; V13s's Phase 2 check then proposes the row together with the `readiness` row (ADR-034 § 11), with the number in
  hand.
- **Recommend (c).** Why: nothing the report would carry exists yet (every consumer field would be `null` or an empty cell; S1–S4 are
  already the eval table's `false_advances`, `early_1000`, `wrong_slide_s`, `recall` and `on_time_1500`); no cap moves; the written
  design (the field set, the `trend.csv` header, the FNV-1a config hash, the numbers-only walk test) is kept on the 10x lane's local
  branch and in `reviews/2026-11-28-v2-10x.md` § 6, so V13s ports it rather than designing it twice; `kit/prompts/07-review.md` reads
  the `--actions` table until then.
- Blocks: nothing before V13s (`specs/2026-11-26-prompts-round-3.md` § V13s task 7 carries the writer; the row question is asked
  there with the readiness row if (c) stands).

## A. This week (blocks the first wave)

**FA-01 — Approve the #59 fix (V0) and the wave-1 pre-approvals.**
- Options: (a) approve V0 plus a standing batch for V0, V1, V2, V5 (push, PR, CI fixes, merge when green); (b) approve step by step.
- **Recommend (a).** Why: two CI env lines and a test; every live number and Q11's volunteer test depend on it (`.10x/handoff.md:92`).
- Blocks: V0, Q11, V3.

**FA-02 — One GitHub ticket batch.**
- Contents: the #59 cause comment; new tickets for the partial-key-sequence drift bug (S-10), the unguarded universal keystroke profile (S-09), FreeShow/OpenLP/OpenSong wire formats (W1–W3), SHA-pinning `ci.yml`/`pages.yml`, help-bundle words default (S-37), roaming session logs (S-36), church identifiers and song titles in public files (only if FA-12 says scrub), the CLI tools zip (V5c, if FA-31 is (a)), and the updater weekend window and Whisper model SHA-256 pinning (if FA-32 includes them), and the input chain (V3b, if FA-40 is (a) or (b)).
- Options: (a) the orchestrator files them under your approval; (b) you file them.
- **Recommend (a).** Why: one approval instead of twelve; texts are drafted in `plan.md` § 5. #59 already exists, so no new ticket for it.
- Blocks: V8, V9, Q13 planning.

**FA-03 — Install uv on the booth PC.**
- Command: `winget install --id astral-sh.uv -e`, open a new PowerShell, `uv python install 3.12`. Python 3.9 stays.
- **Recommend yes, this week.** Why: the private lab and the kit tools run with `uv run`; 5 minutes, no admin.
- Blocks: V1, V6 (private lab).

**FA-04 — Install the Vulkan SDK on the booth PC, and tell us whether the GTX 960 also drives the projector.**
- Steps: install the LunarG Vulkan SDK (winget id **[UNVERIFIED]**; or the installer from vulkan.lunarg.com); run `vulkaninfo --summary` and paste only the GPU line; say whether Presenter's output display is on the GTX 960.
- **Recommend yes, before week 3.** Why: the GPU proof (V4) needs it to build; GPU sharing with Presenter is a stutter risk we must measure.
- Blocks: V4 (full scope), FA-23.

**FA-05 — Approve the two-number "95%".**
- Options: (a) **Correct** (recall ≥ 0.95, ≤ 1 wrong move per 10 songs, 0 hidden mid-song, 0 shown during talk) committed now, **On time** (≤ 1.5 s, song starts separate) target set after V6b; (b) one number, slide accuracy within +1.0 s (draft 10); (c) the strategy's original "≥ 95% within 1.5 s" as the gate now.
- **Recommend (a).** Why: song starts can never meet +1.0 s; today's best offline evidence puts "on time" at ≤ 45%; committing to (c) now would block every release. "Within milliseconds" holds for the move itself (≤ 100 ms).
- Blocks: V2, V6, V11, the default flip.

**FA-06 — Recording consent and the corpus plan (includes "more song fixtures").**
- Ask: written consent from the worship team **and from anyone whose voice is recorded** (pastor, readers); about 3 rehearsal recordings through the desk feed (Nov 29, Dec 6, Dec 13); ≥ 10 songs, ≥ 2 worship leaders, ≥ 3 songs with long instrumental gaps; talk spans muted at intake; a retention date per recording; ~2 h of tap-truth.
- Options: (a) rehearsals only (no service audio); (b) rehearsals + services with talk muted. Option (b) is also what scores the two shadow Sundays: each is measured from a desk recording of the service, tapped and replayed offline (spec § 7.6), because on WorshipTools voxxa cannot see the operator's moves.
- Consent form: `kit/templates/recording-consent.template.md` (who and what is recorded, the purpose, storage on the booth PC only and never uploaded or e-mailed, talk spans muted, a retention date, withdrawal at any time with deletion within 7 days, a parent or guardian for minors, signature and date). It is not legal advice; have the church's counsel review it.
- **Recommend (a) now and (b) before the shadow Sundays (weeks 5–8)**; prefer public-domain or CC0 songs (December carols can become public fixtures with consent). Why: the corpus gates every accuracy claim; rehearsals carry no pastoral prayer; the shadow Sundays cannot be scored without service audio.
- Blocks: F1, V6, V11, V12; the shadow Sundays (V15) need option (b).

**FA-07 — The team: people or agent lanes, and the wave-1 budget.**
- Options: (a) agent lanes for all roles now, the founder as the only human gate; (b) agent lanes plus one human UI consultant for usability rounds; (c) hire or contract senior engineers.
- **Recommend (a) now, (b) before any volunteer usability round.** Why: one step per day was the pace for Q0–Q10; the limits are your gates and Sundays.
- Cost to approve: wave 1 (V0–V6) ≈ $85–210 with the ponytail lane; the whole V plan (21 steps with V3b, V3c and V5c) ≈ $295–730 [est.].
- Blocks: everything.

**FA-08 — Q11's pending decision batch (already open).**
- Which build the volunteers use and where, signing secrets, one more rc tag, the updater key (`.10x/handoff.md:81-87`). New: Q11 should test a build with the #59 fix; #185 moves out of Q11 (FA-37); the V5c tools zip will use the same Windows signing path.
- **Recommend:** answer it together with FA-01 and FA-37.
- Blocks: Q11.

**FA-09 — Delete the old Python prototype from the public repo.**
- `main.py`, `voxxa/`, `requirements.txt` (337 lines, 5 unpinned packages, nothing runs them).
- **Recommend yes.** Why: git history keeps it; it is the only Python that needs pip and a second, wrong "how to run voxxa" path.
- Blocks: V1.

**FA-37 — #185 "Sync from Presenter": Q11 or Q12?**
- Context: the brief (§ 2) lists #185 (a read-only copy of Presenter's service list) as a Q11 idea. The plan moves it to Q12, which builds presenter read-back and setlist import (`plan.md` § 1).
- Options: (a) move it to Q12; (b) keep it in Q11.
- **Recommend (a).** Why: Q11 is the volunteer test of what exists, on a build with the #59 fix; a new feature delays that test, and Q12 is where read-back lands anyway.
- Blocks: Q11's scope (answer with FA-08).

**FA-38 — Fix the hum on voxxa's audio feed.**
- **Answered 2026-11-26: deferred.** No USB interface for some time; the noisy, low-level, unbalanced feed is a supported degraded condition (answers table above).
- **Urgent (round 2, 2026-11-26):** buy option (b), the USB interface with a balanced input, now (~$100–150 est.); hardware first (R6).
- Context: the feed is a WING output on an unbalanced cable into the PC's mic jack (set to Line In), voice on the left channel only. Its floor is about −60 dBFS, mostly 60 Hz ground-loop hum (60, 180, 300 and 420 Hz). Raw singing sits at about −34 dBFS: the voice detector then let through 2 Whisper passes in 200 s; the same take +12 dB gave 152.
- Options: (a) a ground-loop isolator on the cable now (about $15–40 [est.]); (b) a USB audio interface with a balanced line input (about $100–150 [est.], e.g. Focusrite Scarlett Solo or 2i2, or Behringer UMC202HD), fed by a balanced cable from the WING; its gain knob also fixes the low level; (c) (a) now, (b) before voxxa runs on Sundays.
- **Recommend (c).** Why: hum and a low level hide the breaths and soft first words that the onset trigger and the voice detector listen for.
- Blocks: F4 runs after the first (the first may use today's feed through the lab helper's 100 Hz high-pass and +12 dB, as the lab did), the shadow Sundays (V15), FA-23 (minimum hardware).

**FA-39 — A vocal-only feed for voxxa.**
- **Answered 2026-11-26: not a hard requirement.** Must work, with reduced performance, when the full mix is on the feed (answers table above).
- **Urgent (round 2, 2026-11-26):** set up the vocal-only aux now; hardware first (R6).
- Ask: one WING output that carries only the lead vocal mic(s), not the mix, into voxxa's input (mono, the voice on one channel), labelled so nobody repatches it.
- **Recommend yes,** with FA-38's interface (b), before the first test with the band. Why: the onset trigger listens for the breath before each line; over a full band it found 2 of 11 slide starts (P1), on a solo voice 10 of 11 (T1).
- Blocks: the onset trigger with a band (V3b band takes, F4 with the band), the shadow Sundays (V15).

**FA-40 — Approve ADR-030 (Proposed) and supervised live Presenter tests with the lab helper.**
- **Answered 2026-11-26: (a), both approved.**
- Context: ADR-030 "Phrase-onset trigger: arm on words, fire on the onset" (`adrs/030-phrase-onset-trigger-arm-on-words-fire-on-onset.md`, Proposed 2026-11-26). Once the words show the singer within the last 2 words of the slide on screen, the next phrase onset shows the next slide; with no onset, words move it as before. Lab, 1 s passes, Small, median delay: your solo take (T1) 1.90 s → 0.18 s, within ±300 ms 10% → 80%, no wrong slide; the TTS take (T2) 2.12 s → 0.04 s; the band take (P1) 4.30 s → 4.01 s, 11 of 11 both ways.
- Ask: (1) approve ADR-030 as the direction for V3b (it is accepted only with V3b's numbers); (2) approve supervised live tests (F4) with the lab helper `live.py`: you at the booth, no service that day, `--dry-run` first, Right only while a Presenter window is in front, never Left, at most 3 steps per move, Ctrl+C stops it, `--minutes` caps it, numbers-only logs. It runs a local-only voxxad build with streaming stdin (`--stdin-audio - --realtime`; voxxa-private, never pushed).
- Options: (a) both; (b) the ADR and dry runs only (no keys); (c) neither for now.
- **Recommend (a).** Why: the lab cut your solo take's median delay by about 1.7 s with no wrong slide, and only a live test shows it with Presenter and the real feed.
- Blocks: V3b, F4.

**FA-41 — The Presenter test song.**
- **Answered 2026-11-26: (a), the founder builds it to match the kit file when the step is reached.**
- Ask: in WorshipTools Presenter, "It Is Well" (public domain) with slides and arrangement matching the kit song file: a leading blank, then v1 c v2 c v3 c v4 c, each slide split as in the `.txt`.
- Options: (a) you build it in Presenter to match the kit file and confirm the slide count (not counting the blank); (b) permission for voxxa to read a scratch copy of Presenter's own song (a copy, never the live library) and mirror it in the kit file.
- **Recommend (a), with (b) as the fallback if the counts differ.** Why: voxxa presses Right blind on Presenter, so one extra or missing slide shifts every move after it.
- Blocks: F4.

**FA-42 — Approve the V3c Whisper-prompt experiment.**
- **Answered 2026-11-26: approved, lab only.**
- Context: voxxad can hand slide text to Whisper as a prompt (`--bias`, off by default because it can make Whisper write words nobody sang; ADR-004/E6). Today the text follows the conductor, which lags the singer (2 of 11 changes on T1). V3c builds it from the follower's position (the rest of the current line plus the next slide's first line, ≤ 40 tokens) and compares Base with the prompt against Small without it, on recorded takes only, with a phantom-word count (words matched while no voice is heard).
- **Recommend yes:** lab only, recorded takes, never the held-out set, off by default; the prompt text is never logged. Why: Base runs 1.13 s per pass against Small's 2.57 s on sung audio but followed only 5 of 11 changes unbiased (Small 9 of 11); if the prompt closes that gap, passes near 1 s become possible, and the onset trigger needs them.
- Blocks: V3c, the V6 live configuration (which model).

**FA-43 — Approve ADR-031 (Proposed): the parts-map follower.**
- **Answered 2026-11-26: approved**, plus a framework that traces parts from metadata and runs on Linux and Windows booth PCs (FA-73).
- **Urgent (round 2, 2026-11-26):** approve ADR-031 as recommended, with the R3 amendment (break gate as a lab switch, FA-63).
- Context: in the first live test (It Is Well, 16 Right presses in order) you saw early moves and no way back to the chorus. A replay found 5 changes more than 1 s early: two from guessing the pace while you held long notes, two from the phrase start after a chorus (a repeated "with my soul"), one between the chorus's two look-alike slides. ADR-031 (`adrs/031-parts-map-follower-plan-is-a-hint.md`) treats a song as a map of parts. The planned order is a hint, and any part can come back. A slide moves on words or a phrase start, never on pace alone. Lab, over 7 sung takes (your two takes, the TTS fixture, and four re-cuts of your take into other orders: an extra chorus, a skipped verse, the chorus first, the last line repeated): right changes 49 → 74 of 77, wrong moves 31 → 4, early by more than 1 s 14 → 1, seconds of a wrong slide on screen 327 → 112.
- The cost: after a chorus, the next verse appears 1–3 s after its first word (voxxa waits to hear which part it is), not on the breath. Verse into chorus stays on the breath.
- Options: (a) approve as the direction for V3d (accepted only with V3d's numbers), waiting for words after a chorus by default; (b) the same, but the phrase start after a chorus shows the planned next part (faster when the plan is kept; in the lab it made 3–4 wrong moves on takes with detours); (c) not yet.
- **Recommend (a).** Why: you said the problem was moving too early, not lag; (a) cut early moves from 14 to 1. Practice memory can later switch a leader who keeps the plan to (b), from measured habits.
- Blocks: V3d, the parts map in V11.

**FA-44 — How voxxa goes to any slide in the presentation app.**
- **Answered 2026-11-26: proceed with (d) the FreeShow trial and (e) suggest-only on Presenter until a jump is proven.**
- **Urgent (round 2, 2026-11-26):** answered as (d) FreeShow trial via V3e and (e) suggest-only on Presenter until then (FA-47 Tier A/B).
- Context: to follow flowing worship voxxa must sometimes go back (to the chorus) or skip ahead (a skipped verse). WorshipTools Presenter's keys are next and previous slide, next and previous item, and F10–F12 clear. There is no key for a section or a slide number. Going back three slides means three Left presses, and each slide in between shows on the screen.
- Options: (a) **Presenter, arrow bursts:** Left or Right several times in a row; the in-between slides may flash for a moment (how visible is not measured); (b) **Presenter, hide while stepping:** clear the text (F10 or F11), step with the arrows, then show again; works only if Presenter keeps the screen clear while stepping (not known; a 10-minute test on a test song); (c) **Presenter, click the slide in its window** (Windows UI automation): direct, no flash, but breaks when Presenter's layout changes; it needs the same front-window guard as the keys; (d) **a presentation app with commands:** FreeShow is free and open source, and voxxa already has a FreeShow driver that can go straight to a slide (its wire format, W1, still to verify); ProPresenter has a network API but is paid; (e) **suggest, don't jump:** for a move off the plan voxxa shows the operator one button ("Chorus?"), and the operator confirms it.
- **Recommend:** this week, test (b) and (a) on a Presenter test song (no service, you at the booth, voxxa only watching the screen). Use (e) during services until a jump is proven. Trial (d) FreeShow as the long-term path, since it fits your "API commands" and free-tools goals.
- Blocks: V3d task 5 (a live test that leaves the plan), the jump interface in V11's port plan.

**FA-45 — Optional: tap the truth for the live test (live4).**
- **Answered 2026-11-26: yes; the founder records the needed takes alone for now.**
- **Urgent (round 2, 2026-11-26):** tap live4's truth (5 min) before ADR-031 cites it; the replay's word-driven delays are optimistic by about 0.5–1.2 s (R14).
- Ask: replay the live4 recording in `vocal_test.py tap` with your headphones and press SPACE at each slide change (about 5 minutes).
- Why: ADR-031's live4 numbers use slide times read from Whisper's words (about ±1 s). Tapped truth makes "early" and "late" exact.
- Blocks: nothing; it sharpens the live4 row.

## A2. Autonomy round (2026-11-26, draft): FA-46 … FA-72

Verbatim from `round2/50-adjudication.md` § 5 (rulings C1–C17 and R1–R17 there). Answer with "FA-52 yes" or "all recommendations". Nothing here is approved yet.

Already open and now urgent: **FA-10** (wiring row) -> approved 2026-11-26, option (a); **FA-44** -> answered as (d) FreeShow trial
via V3e and (e) suggest-only on Presenter until then; **FA-45** -> tap live4's truth (5 min) before ADR-031 cites it;
**FA-38 (b), FA-39** -> answered 2026-11-26 as deferred and not required (a deviation from R6: the full mix on the current unbalanced, low-level feed is a supported, degraded condition); **FA-29** -> confirm the
stage output is never captured into the stream before any stage display is used; **FA-43** -> approve ADR-031 as
recommended, with the R3 amendment.

| FA | Decision | Recommendation |
|---|---|---|
| FA-46 | Approve the V3e spike and installing FreeShow **stable** beside Presenter, weekday before Dec 18; Presenter stays live | yes · **answered yes, 2026-11-26** |
| FA-47 | Two-tier app policy: Tier A (jump + read-back) is autonomy-eligible; Tier B (keys-only) is assisted with suggestion cards; Left bursts refused in code | yes · **answered yes, 2026-11-26** |
| FA-48 | Gate and shape for switching the Sunday output: V3e go; >= 2 dark Sundays with FreeShow's read-back agreeing with voxxa's own commanded slide on >= 99% of polls and 0 `position_uncertain` (ruling after integration: The 99% line measures the driver: FreeShow's read-back against voxxa's own commanded slide over all polls, with 0 `position_uncertain` (Presenter has no read-back on a dark Sunday). Follower correctness is scored as in stage 0 (shadow plus tapping) until stage 2, where the volunteer's FreeShow moves become operator-timed truth (FA-66).); library re-imported, songcheck green, groups match labels; worship leader checked the stage display; one volunteer trained on FreeShow keys; **per-service switch, whole service, 4 double-build Sundays, Thursday 18:00 rollback rule, Presenter kept a month**; window Jan 10 - Mar 21, 2027; decide end of January | adopt · **answered yes, 2026-11-26** |
| FA-49 | Budget gate: V3e now (<= $20); V8b (<= $60) on a V3e go; nothing for an aligner, tracker or new model before V6 reports and the GPU is measured with both apps open | yes · **answered yes, 2026-11-26** |
| FA-50 | Model and aligner licences: only permissive weights (MIT, Apache, BSD, CC-BY, CC0) in the product or kit tools; CC-BY-NC lab-only; amend ADR-005's dependency rule to cover model weights | yes · **answered 2026-11-27: use whatever licensing is required for now; own MIT-like ministry licence later (FA-76); ADR-005 amendment = inventory, not a gate** |
| FA-51 | Pin FreeShow: stable channel, auto-update off, version in `church-profile.json`, checked by the health check; upgrades on a weekday | yes · **answered yes, 2026-11-27** |
| FA-52 | Two gates: 2 scored shadow Sundays -> songs mode with a minder at the PC (ADR-022 unchanged); no-minder gate = 4 consecutive green Sundays, >= 300 scored occurrences, 0 early beyond the line, 0 spurious presses, two volunteers passed the card; early line 0.3 s on rehearsal truth, 1.0 s live; tighten only | yes · **answered yes, 2026-11-27: extensive listening period; never interfere with the volunteer** |
| FA-53 | Presenter MIDI mapping check (20 min, inside V3e): does any command name a slide or section? A "no" closes the "wait for WorshipTools" option | yes · **answered 2026-11-27: skip it** |
| FA-54 | New ADR-028 rows: onset 250 (split from follower), input chain ~200, health check 150; `vad/` -> recognizer row; `align` 400 only per R5 | yes · **answered yes, 2026-11-27, with caps at 3 × the estimates: onset 750, input chain 600, health check 450 (batch 2; FA-74 for the rule's scope)** |
| FA-55 | FreeShow API posture: password set, 5505/5506 firewalled to loopback, voxxa refuses an API that answers without a password unless `require_password` is set to false (setting `require_password`, default true; ruling after integration), bearer header only, token never logged or bundled | (a) refuse by default · **answered yes, 2026-11-27; steps in `out/founder-freeshow-setup.md`** |
| FA-56 | Unique layouts (every slide once) for followed songs in v1; a per-song occurrence resolver later with Q12 | yes · **answered 2026-11-27: the per-song JSON layout file is the object of record (ADR-032 Decision 13 as amended); unique layouts stay the v1 rule** |
| FA-57 | Text transition 0 ms for the followed output style (FreeShow default is a 500 ms fade) | measure in F5, then yes · **answered 2026-11-27: keep the 500 ms default (reading time)** |
| FA-58 | Controls: F15 HOLD (toggle), F20 STOP FOLLOWING (no confirm; dispatch off, listening continues), F14 Back one; routes `/pause /resume /panic /state`; FreeShow output lock as external panic; voxxa API stays 127.0.0.1; no Escape; Off only in the mode switch | yes · **answered yes, 2026-11-27, on condition of no hotkey collisions (FS14 check)** |
| FA-59 | Alarm/veto surface when nobody is at the PC: a Companion/Stream Deck page (or a phone) at the stage, colour from `/state`, calling `/pause`; docs recipe only; hardware optional ($0-80 est.) | (a) · **answered 2026-11-27: not now (deferred)** |
| FA-60 | Stage cue: voxxa's own three-state strip; heard-words line removed (ticket now, fix in V14) | yes · **answered yes, 2026-11-27** |
| FA-61 | Badge words "voxxa moves it / voxxa suggests / You move it"; any manual move demotes the song for the rest of the song | yes · **answered 2026-11-27: no demotion; voxxa re-anchors and keeps following; the badge shows live mode** |
| FA-62 | Health-check sentences approved word for word on the V10 mock; "Ready, with a note" may start Songs; only a red slides-app row blocks | yes · **answered 2026-11-27: approve when I see the mock (V10)** |
| FA-63 | Break gate: lab switch in V3d, `PartsConfig.break_gate` default `None` in V11, before/after measured; live4 is not its evidence | yes · **answered 2026-11-27: build it on by default (deviation; V3d still measures before/after)** |
| FA-64 | Per-leader handicaps only after >= 3 confirmed runs; leader alias L1/L2 in practice files; never `boundary = plan` | yes · **answered yes, 2026-11-27** |
| FA-65 | JSONL: one `trace` line type with `kind` (ADR-001/003 amendment) | (a) · **answered yes, 2026-11-27** |
| FA-66 | Read-back as shadow truth (operator-timed) for the Correct bar; tapping one Sunday in four for Tight | yes · **answered yes, 2026-11-27** |
| FA-67 | Booth PC policy on service days: no RDP Sat 18:00 - Sun 14:00; Windows Update active hours cover services, auto-restart off; voxxa started and armed by a person, never auto-start; health check within 30 min of the service | yes · **answered 2026-11-27: yes, and voxxa may auto-start (Songs still armed by a person)** |
| FA-68 | Repertoire sign-off: the worship leader signs the inventory counts (licensed source / none / legacy CCLI) before any migration weekend; first ten re-imported songs = most-sung licensed songs | yes · **answered 2026-11-27: later (end of January)** |
| FA-69 | Who minds: >= 3 minders from the existing slide team; the founder is never the Sunday minder from month two; a veteran (3 months) trains the next | yes · **answered 2026-11-27: fewer minders is fine** |
| FA-70 | Words: no "AI" from the stage or in the bulletin; one-paragraph notice; the pastor decides wording | yes · **answered 2026-11-27: no notice at all** |
| FA-71 | Scoring labour: talk-span notes live, talk muted, a consented volunteer taps songs only, partial scoring counts; founder <= 45 min per scored Sunday, <= 2 h/week total | yes · **answered 2026-11-27: 90 min per scored Sunday, 4 h per week** |
| FA-72 | Approve the drops and deferrals in R17 | yes · **answered 2026-11-27 per line: all drops approved (Holyrics only ever as an ADR-036 bindings file, on need); deferrals as recommended except V16, which runs now** |

## B. Next (before wave 2–3, weeks 2–3)

**FA-10 — The budget wall: approve one 150-line increase as a recorded "wiring" row.**
- **Answered 2026-11-26: approved, option (a).** V7 may run.
- **Urgent (round 2, 2026-11-26):** approve this week (V7 must land before the FreeShow driver V8b and the new budget rows, FA-54).
- Context: "everything else" has 1 line left (`.10x/handoff.md:111`); dispatch, HTTP routes, hotkeys and log location all live there.
- Options: (a) ADR-028 wiring row (measured size + 150), "everything else" lowered by the same size; (b) a refactor PR to free lines first; (c) stop all backend work.
- **Recommend (a).** Why: no file moves, the rule "caps only go down" stays true for every existing row, and the 150 lines are visible in one ADR.
- Blocks: V7, V8, V9, V11, V13, V14.

**FA-11 — Where the "listener DB" lives.**
- Options: (a) one JSON file per song in the church folder (ADR-023); (b) one SQLite file per church (draft 12, amends ADR-003, ~900 lines of store code).
- **Recommend (a).** Why: < 25 KB per song, one PC per church, ADR-003's review trigger is not hit; it can be imported into a database later if a church grows past ~500 songs.
- Blocks: V12.

**FA-12 — Scrub church identifiers from the public repo.**
- Today's public files name the church's CCLI account number, its worship band (as a song's artist), the booth PC's host name and the church itself (review 20 S-42), and private song P1's title or slug.
- Scope, by `path:line` only (ranges here; every exact line is in `plan.md` V5 step 6, checked on `866b51e`): the song title or slug in 36 lines of 8 files (`.10x/handoff.md:134-144`, `.10x/status.md:4` and `:342`, `.10x/reviews/2026-11-23-ponytail-q10.md:4`, ADR-019 `:110-113`, `fixtures/README.md:294-352`, `fixtures/LICENSED_RUNS.md:5`, `:9`, `:13`, `fixtures/tools/tap-truth.ps1:20`, `src-tauri/tests/media_guard.rs:180-182`); the host name in 64 lines of 21 files (not only `fixtures/README.md:288`); ADR-019's private-song row; `LICENSED_RUNS.md` row 1, whose template columns become `alias`, `licence_class`, numbers (ADR-026 § 10); the prompt pack's church name.
- Order: the scrub PR merges **before** the deny-file scan (ADR-029 § 3, which includes song titles) becomes a gate over `.10x/` and `fixtures/`; otherwise that scan fails every PR that touches them. Before the PR merges, you run the deny-file scan on the booth PC over `.10x/`, `docs/`, `fixtures/` and `src-tauri/tests/` and paste counts and `file:line` only; anything it finds joins the PR.
- Options: (a) replace them with aliases in current files now, no history rewrite; (b) (a) plus a history rewrite (outward, hard to reverse, breaks forks and archive branches); (c) leave them (then the deny-file scan can gate `kit/` only).
- **Recommend (a).** Why: stops new copies (the `LICENSED_RUNS.md` template invites every church to commit its number); a rewrite costs more than it protects because the data is already public and archived.
- Blocks: V5, the deny-file gate in V15.

**FA-13 — Licence policy for the church and the public repo.**
- Allowed in the church library: PD, CC0, CC BY, CC BY-SA, church-owned, written permission; CC BY-ND only if the church does not stream or record (FA-15); NC per FA-16; CCLI optional, legacy; UNKNOWN allowed as a draft with a warning.
- Public repo fixtures and kit examples: PD and CC0 only for now; CC BY/BY-SA later, song by song, with written performer consent.
- **Recommend as written.** Why: CC treats music synced with video as an adaptation (ND and SA risk on streams) **[UNVERIFIED legal]**.
- Blocks: V5.

**FA-14 — How lyrics are typed in.**
- Options: (a) Notepad `.txt` per song plus the `songs.csv` licence register and `songcheck`, then voxxa's existing loader and double-click editor; (b) an in-app song library and editor now (1,050–1,700 GUI lines); (c) (a) now, (b) when church #2 asks.
- **Recommend (c).** Why: (a) works in days and gives Presenter and voxxa the same text from one file; the strategy ruled out lyric editing (`.10x/specs/2026-10-24-software-strategy.md:30`). Note: this is a deviation from "entered in voxxa" in the literal sense; your call.
- Blocks: V5 (format), later UI work.

**FA-15 — Does the church livestream or record services? Is the stream monetised or does it ask for gifts?**
- Why it matters: with streaming, CC BY-ND songs may not be usable, SA songs carry ShareAlike duties on the video, NC songs on a monetised stream are a grey area **[UNVERIFIED legal]**.
- **Recommend:** answer yes/no for each; if yes to streaming, get counsel before adopting ND or SA songs.
- Blocks: the church profile, `songcheck` rules.

**FA-16 — Non-commercial (NC) song policy.**
- Options: (a) NC songs allowed in a church's library with an amber warning; never on a monetised stream; voxxa's paid Setup & Calibration staff never type NC lyrics for a church; (b) NC not allowed; (c) allowed with no warning.
- **Recommend (a).** Why: CC's NC test is about the use, not the user; worship use is likely fine; paid work by voxxa is the risk.
- Blocks: `licence-kinds.csv` values.

**FA-17 — How the operator overrides while Presenter is in front (pilot).**
- Options: (a) a USB foot pedal or Stream Deck sending F13–F24, plus voxxa's HTTP API (about $30–150 [est.]); (b) keyboard F-keys after a conflict check; (c) both.
- **Recommend (a) for the WorshipTools pilot:** F13–F24 from a pedal or Stream Deck, plus the HTTP API. Keyboard F-keys (b) only for other apps, after the conflict check. Why: WorshipTools uses F6–F9 (documented on its shortcuts page; confirm on the test card, T7), so keyboard F-keys would collide with Presenter; F13–F24 exist on no keyboard, so nothing else uses them; the HTTP path already works today; never Ctrl+Alt+arrows (display rotation on some Intel drivers **[UNVERIFIED per PC]**). Pause is F15 from the pedal, or Space when voxxa's window is in front (V14).
- Blocks: V13, V14.

**FA-18 — WorshipTools test card 2 on the booth PC (F3).**
- The rows still open on WorshipTools, by the numbers of `kit/templates/presenter-test-card.md`: T4 the key that restores the lyrics after F10; T5 what Right shows on a song's last slide when the next item is a graphic; T6 whether voxxa may ever send Down; T7 function keys: Presenter's use of F6–F9 is documented on its shortcuts page, so confirm it on the card, and whether it reacts to F13–F19; T8 confirm the `goto.keys` chord (Right Right into the next song); T9 the window title/class of Presenter's main window and an edit dialog; T10 confirm Presenter is not run as administrator.
- **Recommend:** 30 minutes on a weekday. Why: V13's hints and the window-class guard depend on it; agents never press keys in Presenter.
- Blocks: V13.

**FA-19 — Help bundle: "include the words heard" unticked by default.**
- **Recommend yes.** Why: after a prayer-heavy service, a ticked default e-mails prayer words with names (`src/simple/help.js:38`); the volunteer can still tick it.
- Blocks: V9.

**FA-20 — The private church folder.**
- Options: (a) no git; dated weekly copies to one BitLocker-encrypted drive (`robocopy "<church_root>" "E:\voxxa-backup\<yyyy-mm-dd>" /E /XD sessions inbox`, keep the last 8, test one restore every quarter; not `/MIR`, which copies an accidental delete into the backup); BitLocker on the booth PC; an `.ignore` file so code-search tools skip it; (b) a local git repository with a refusing hook (draft 12).
- **Recommend (a).** Why: hooks are skipped by `--no-verify` and do not stop `git remote add`; BitLocker protects recordings if the PC walks.
- Blocks: V5, F1.

**FA-21 — Show the first line of the current and next slide on the Run screen (operator only).**
- Amends ADR-017 ("titles only"); ADR-011 allows operator previews.
- **Recommend yes.** Why: it is the fastest way for a volunteer to see that voxxa is behind. Risk: if the booth display is duplicated to the projector or stream, those lines would show (see FA-29).
- Blocks: V14.

**FA-31 — How churches get the command-line tools (`voxxad`, `voxxa-eval`).**
- Context: the kit's practice capture, `.wav` intake and layout check run through `voxxad` and `voxxa-eval`. Both are developer tools behind the off-by-default `cli` feature, and the installers leave them out on purpose (your Q7 decision; `src-tauri/Cargo.toml:21-35`, `:121-122`). The kit also forbids test builds on a church booth PC. Without a change, church #2 cannot run a practice session, a `.wav` intake or a layout check.
- Options: (a) a separate signed release asset, `voxxa-tools-<version>-windows-x64.zip`, built with `--features cli` in the Windows release job (new step V5c, after V0 and before church #2); (b) a Tauri `externalBin` sidecar inside the installer; (c) wait for GUI capture in Q14.
- **Recommend (a).** Why: no installer change, so the updater and signing paths stay the same and the Q7 rule (installers carry voxxa only) still holds; it is one S step. (b) changes the installer and every updater artefact; (c) leaves church #2 without practice memory or intake for months.
- Until V5c ships: practice capture, `.wav` intake and `--check-layout` run only on your development booth from a local build. The kit README says so.
- Blocks: V5c, church #2.

**FA-32 — What did "latest updates" mean?**
- Context: your request says "latest updates" and nothing more. Spec § 4.12 reads it as five things: (1) app updates for churches through the existing signed updater on the stable channel, never between Saturday 18:00 and Sunday 14:00 local time or while Songs is on; (2) a monthly agent-run window that updates `Cargo.lock` and the GitHub Actions SHAs, with `cargo deny` passing and your approval on the PR (whisper-rs stays at 0.13.2); (3) Whisper models pinned by SHA-256, with a model change going through the eval gate; (4) kit versions tagged `kit-vX.Y.Z` with upgrade notes, `kit_version` recorded in `church-profile.json`, and an upgrade by copying the new templates and running `kit_check`; (5) the minimum uv version is the one the booth installs in V1, recorded in `kit/README.md`.
- Options: (a) all five; (b) app updates only; (c) something else, such as pulling song updates from a source (say which).
- **Recommend (a).** Why: each part is small (two tickets, one monthly agent run, one README line), and together they keep the booth, the kit and the dependencies current without surprising a church on a Sunday.
- Blocks: spec § 4.12; the updater and model tickets in `plan.md` § 5; FA-30's dependency question.

**FA-33 — Command lines out to other programs: not in v1.**
- Context: you asked for "cli commands" paired with the layout. Any command line can already drive voxxa through its local HTTP API (PowerShell `Invoke-RestMethod`, curl, Companion; spec § 4.6). The other direction, voxxa starting a program when it hears a phrase, is the highest-risk feature in the plan (review 20 S-01, S-03, S-05), and no church needs one yet.
- Options: (a) defer until a named need (a specific program a church must start); (b) build ADR-025's controls (allowlist, absolute path, SHA-256 of a voxxa-owned copy, typed placeholders only, GUI approval, HMAC trust record) in wave 3 for one named program.
- **Recommend (a).** Why: no program is named yet, and ADR-025 already records every rule, so building it later needs no redesign.
- Blocks: nothing now; ADR-025 § 5 work if (b).

**FA-34 — Autonomous scripture and key-word graphics: suggest-only in v1.**
- Context: you asked voxxa to "autonomously change slides based on song, scripture or key words identified with graphic". Songs move autonomously. In v1, cue words and scripture ship as **suggestions** the operator accepts or dismisses; scripture waits for Q16–Q18 and is a hint only on WorshipTools (it has no API). Auto mode needs the exposure bar (0 wrong fires in ≥ 300 armed opportunities or ≥ 30 h of armed talk audio from public recordings) and a typed confirm in `layout.json` (spec § 4.6).
- Options: (a) suggest until the exposure bar is met, then auto per cue with the typed confirm; (b) auto for graphics on spoken items only, with the typed confirm, from the first release; (c) off.
- **Recommend (a).** Why: a graphic fired during prayer or a sermon is the most visible mistake voxxa could make; the bar is measured on public recordings, so it costs no church data.
- Blocks: V13 (mode defaults), Q18.

**FA-35 — Outbound hotkey sequences.**
- Context: you asked for hotkey sequences paired with the layout. A binding profile may send a `keys` step of up to 4 chords per action (ADR-024 § 1, ADR-025 § 7). `kit/templates/bindings/keystroke.universal.json` and `kit/templates/bindings/keystroke.worshiptools_presenter.json` each show one example `keys` step with a chord, marked **[UNVERIFIED]** per app: nobody has pressed those keys in that app yet. In v1 only slide stepping runs on keystroke apps; a `keys` step runs only after the presenter test card's row T8 confirms the chord (Home, or Right Right) on a live install.
- Options: (a) as described: key sequences live in the voxxa-owned binding profiles and run only after a test card per app; (b) let each church write its own key sequences in `layout.json`.
- **Recommend (a).** Why: binding profiles are compiled in and allowlisted (review 20 S-04); a church-written key sequence carries the same risk as a command line (a shared file that types into whatever window is in front).
- Blocks: V13 (which steps run), F3 (test card).

## C. Later (before the step that needs it)

**FA-22 — Which presentation apps to support next, after WorshipTools Presenter.**
- Options, in recommended order: (1) **FreeShow** (free, open source, documented HTTP API, a test install costs nothing; its driver has a suspected body-shape defect, W1); (2) **ProPresenter 7.9+** (REST API; widely used **[UNVERIFIED market share]**; needs a licence for a test install); (3) **OpenLP 3** (free; driver paths need fixing, W2); then Proclaim, Holyrics, EasyWorship only when a church asks.
- **Recommend:** a Windows VM with FreeShow and OpenLP 3.1 now (Q13), ProPresenter when a church on it signs up. Why: no API step can ship without a live contract test.
- Blocks: Q13, binding-profile steps.

**FA-23 — Minimum hardware for other churches.**
- Options: (a) promise "volunteer-ready" only on class G (a Vulkan GPU with ≥ 2 GB) or a CPU that passes the V3 bench; class C4 (older 4-core, no GPU) gets "rehearse every song first" and separate numbers; (b) promise every PC.
- **Recommend (a), decided after V3/V4 numbers.** Why: Small is the smallest clean model and needs < 1 s per pass.
- Blocks: kit hardware check, sales promises.

**FA-24 — When to do the lowercase "voxxa" rename.**
- Options: (a) before you publish the first full v0.1.0 (V16); (b) later with a migration.
- **Recommend (a).** Why: the exe, install folder, updater artefacts and winget names follow `productName`; before the first publish there are no installed copies to migrate.
- Blocks: V16, the v0.1.0 publish.

**FA-25 — Kit governance.**
- CODEOWNERS for `kit/prompts/**`, `kit/skills/**`, `kit/templates/bindings/**` (you); churches use tagged kit releases (`kit-vX.Y.Z`); the kit's example agent settings deny personal-data connectors during church setup.
- **Recommend yes.** Why: a kit skill is an instruction agents run at every church with its data.
- Blocks: V5 (CODEOWNERS file), church #2.

**FA-26 — The church's own songs: share them under Creative Commons?**
- E.g. release private song P1 under CC BY 4.0 so it can become a public fixture and other kit churches can sing it.
- **Recommend:** the church's and the writers' decision; voxxa supports either. Why: a public sung fixture would let CI test real singing.
- Blocks: nothing now.

**FA-27 — The written question to CCLI (ADR-007:13).**
- **Recommend:** keep it only if other churches will use CCLI songs with voxxa. Why: CC, PD and church-owned songs carry their own permission.
- Blocks: ADR-026's amendment wording.

**FA-28 — A booth freeze over Christmas.**
- **Recommend:** no release or dev build on the booth PC from Dec 18 to Jan 3, plus the weekly Friday 18:00 → Sunday 14:00 merge freeze. Why: Christmas services; shadow runs only.
- Blocks: calendar.

**FA-29 — Is the booth desktop duplicated to the projector, or captured for the stream?**
- **Answered 2026-11-27: yes.** All Presenter output is streamed through the Blackmagic (ATEM Constellation) chain and appears on YouTube as a lower third; critical. See batch 3 and FA-75.
- **Urgent (round 2, 2026-11-26):** confirm the stage output is never captured into the stream before any stage display is used (R9, FA-60).
- **Recommend:** check once and add it to the service-day checklist. Why: operator previews (slide lines, heard words on the stage window) would reach the room or the stream (review 20 S-38); automatic detection is an unverified spike.
- Blocks: FA-21's safety, the checklist.

**FA-30 — Deferred spending decisions.**
- A volunteer usability round for new screens (about $400 for 10 volunteers [est.]): **defer** until the Run screen ships. Dependabot vs an agent-run monthly dependency window: now part of FA-32 (its option (a) includes the monthly window); keep the weekly `cargo deny` idea for the ticket queue.
- Blocks: nothing now.

**FA-36 — The circle-logo exe icon.**
- Context: brief § 7: the exe icon becomes the circle logo (`voxxa_logo_circle.png`: teal topographic contour lines in a white and teal ring, no text), and small sizes need a simplified variant.
- Ask: add the source PNG (square, at least 1024 px, transparent corners) in the V16 PR (it becomes public as the app icon anyway), and approve a simplified 16 px and 32 px variant (fewer contour lines, a thicker ring) drawn by you or a designer.
- Options: (a) do it in V16 with the rename (`tauri icon` plus the simplified variant); (b) a separate step after v0.1.0.
- **Recommend (a).** Why: both change the installer and what a church sees; one install-over-old-version check on a test PC covers both, before any church has installed voxxa.
- Blocks: V16.

---

## What was merged into the items above

- Draft 10 (1–8) → FA-05, FA-02, FA-04, FA-06, FA-13, FA-23; its PyTorch install (8) is deferred with the draft-truth lab.
- Draft 11 (D1–D8) → JSON format (ADR-024, no question), FA-17/FA-25 (D2 keyring deferred with API steps), FA-13 (D3 modes: suggest), FA-18 (D4/D5), deferred CLI (D6), deferred EasyWorship (D7), FA-22 (D8).
- Draft 12 (D1–D14) → FA-09, FA-11, FA-13, FA-16, FA-15, FA-26, FA-04/FA-23 (D7), deferred (D8), FA-20 (D9), ADR-029 (D10), numbering done here (D11), FA-03 (D12), the founder moves P1's fixture with F1 (D13), FA-02 (D14).
- Draft 13 (F1–F12) → FA-21, FA-17, ADR-026 "warn, never block" (F3), Rehearse listen-only (deferred screen, F4), Space as Pause (V14, F5), names in ADR-024 (F6), FA-10 (F7), F8 booth-only preparation (service files may be e-mailed but only lower modes), typed AUTO confirm in ADR-024 (F9), F10 deferred with Review, FA-26 (F11), FA-30 (F12).
- Draft 14 (1–14) → FA-01, FA-02, FA-06, FA-04, FA-07, FA-18, FA-13, FA-14 (8: the Planning paste check is on the church-setup checklist), FA-24, FA-08 (10), FA-06 (11), FA-27, FA-12, FA-22.
- Review 20 (SF1–SF11) → ADR-025 (SF1), ADR-024 (SF2), FA-02 (SF3), FA-12 (SF4), FA-15 (SF5), FA-06 (SF6), FA-20 (SF7), FA-19 (SF8), FA-20 (SF9), not planned (SF10), FA-25 (SF11).
- Review 21 (1–5) → FA-05, FA-10, this list (3), FA-14, FA-17.
- Critic round 1 (G01–G20, `adjudication.md` § 8) → FA-31 (G01), FA-32 (G05), FA-33, FA-34 and FA-35 (G06), FA-36 and FA-37 (G19); FA-06 links the consent template (G08) and its option (b) now also scores the shadow Sundays (G09); FA-12 covers song titles and the host name by `path:line`, merged before the deny-file gate (G03); FA-20 uses dated weekly copies (G20); FA-30's dependency question moves into FA-32 (G05).
