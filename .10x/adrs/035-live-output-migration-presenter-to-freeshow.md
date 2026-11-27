# ADR-035: Live output moves from Presenter to FreeShow, one whole service at a time

**Status:** Proposed (until FA-48 is decided) · **Date:** 2026-11-26 · **Feature:** church-autoadvance · **Reversibility:** Easy per service until Presenter is retired (Thursday 18:00 rollback rule; Presenter kept a month) · **Owner:** nategroks · **Author:** 10x-Team, from `round2/50` R1, R2, R15, C12, FA-48, FA-68, `round2/51` § 2.3 and § 2.6, and the V17 prompt in `round2/52`

A stub. V17 completes it at its Phase 2 check. It is accepted or rejected with FA-48 at the end of January 2027. The Tier A/B policy lives in ADR-032 (50 § 4 numbering note); this ADR does not restate it.

## Context
- Two decisions, kept apart (R1). Engineering: FreeShow becomes voxxa's Tier A target now (ADR-032). Church: switching the Sunday output is staged and decided at the end of January on dark-Sunday numbers.
- Two apps cannot own the projector within one service (C12). A mid-service swap is a black screen, and the stream capture path may be Presenter-specific. (cited, 51 risk 6)
- WorshipTools has no bulk song export. The FreeShow library is rebuilt from licensed sources only (ADR-026), and a song with no licensed source is not re-imported. So the migration is a repertoire decision. (cited, 52 V17; 51 risk 11)
- Christmas booth freeze, Dec 18 – Jan 3: nothing changes on the booth PC (50 § 6).

## Decision
1. **Stages 0–5 (R2).** Entry criteria in full: 51 § 2.6.

| Stage | What runs | Rollback trigger → action |
|---|---|---|
| 0 | Shadow on Presenter (today) | — |
| 1 | FreeShow installed with **no output assigned**; voxxa drives it in shadow on weekdays, then on the dark Sundays while Presenter stays live; read-back logged | FreeShow crashes Presenter, VRAM > 1,800 MB or projector stutter → close FreeShow for that service |
| 2 | FreeShow live, a volunteer at its keys; voxxa in shadow, scored from read-back (the double-build Sundays) | a FreeShow-caused visible failure → rollback declared by Thursday 18:00; Presenter next Sunday |
| 3 | One promoted song autonomous; the volunteer moves the rest and holds HOLD and STOP | an S1 or S2 breach → that song demoted; two breaches → stage 2 for 2 services |
| 4 | All promoted songs autonomous; the volunteer is the minder at the PC | any panic → stage 3 next service; a song's S1 or S2 breach → that song demoted |
| 5 | No volunteer in the booth; the leader holds a Companion or phone HOLD | any S1, S2 or S5 breach → a volunteer back at the PC for 2 services |

   Stage 3 needs ADR-034's gate 1; stage 5 needs its gate 2.

2. **The switch gate (FA-48).** V3e go; ≥ 2 dark Sundays with FreeShow's read-back agreeing with voxxa's own commanded slide on ≥ 99% of polls and 0 `position_uncertain` (this measures the driver; follower correctness is scored by shadow plus tapping until stage 2, then from the volunteer's FreeShow moves, FA-66); library re-imported, `songcheck` green, groups match labels; the worship leader has checked the stage display; one volunteer trained on FreeShow's keys. Decided at the end of January.
3. **The shape (FA-48, C12).**
   - **Per-service switch, whole service.** Announcements, scripture and sermon slides are rebuilt beforehand.
   - **4 double-build Sundays.** The whole service is built in both apps. FreeShow is live; Presenter is on cold standby with the same service.
   - **Thursday 18:00 rollback rule.** A FreeShow-caused visible failure on Sunday (black projector, wrong song, lost stream) → rollback declared by Thursday 18:00, and Presenter runs the next Sunday.
   - **Presenter kept a month.** Installed and maintained for four weeks from the first FreeShow-live Sunday (R2). Keeping it longer is allowed.
4. **Window: Jan 10 – Mar 21, 2027.**
   - Dark Sundays: Jan 10, 17 and 24, 2027 (F7, stage 1, 15 min each).
   - FA-48 decision: end of January.
   - If go: the four double-build Sundays fall inside the window (51 places them Feb 7–28 [est.]). Mar 21 is the last window Sunday.
   - Easter, Mar 28, 2027: configuration frozen; no rollout change (inferred, 51).
   - Slip rule: no go by Jan 31 → re-decided after Easter; Tier B (suggest-only on Presenter, minder at the PC) runs through Lent (51 § 2.3).
5. **Repertoire sign-off before any migration weekend (FA-68, R15).** The worship leader signs the inventory counts: licensed source / none / legacy CCLI. The first ten songs re-imported are the ten most-sung licensed songs. Songs with no licensed source stay on Presenter during the transition, or the leader drops them. Only counts leave the PC.
6. **FreeShow set-up rules, from ADR-032.** Stable and pinned (FA-51); password and loopback firewall (FA-55); unique layouts (FA-56); 0 ms text transition for the followed style (FA-57). FreeShow's Planning Center connection stays off until FS20 says whether its import carries song text (ADR-026).

## Alternatives Considered
| Alternative | Why not |
|---|---|
| Switch one rehearsed song at a time | two apps cannot own the projector in one service (C12) |
| Switch everything at once, no double-build Sundays | no rollback path; the volunteers would learn FreeShow live (inferred) |
| Stay on Presenter, suggest-only | this is the fallback if FA-48 is a no; never autonomy-eligible (ADR-032) |
| Wait for a WorshipTools API | no staff reply since 2024-07 (cited, 51); FA-53's MIDI check is the last test |

## Consequences
- **Positive:** the church gets a Tier A output on which voxxa can jump and verify. Rollback stays one Thursday decision away for a month.
- **Negative:** four Sundays of building every service twice; the slide team learns a new app; the repertoire may shrink.
- **Risks:** GPU load with both apps open (the stage-1 rollback trigger); a three-week slip pushes the switch past Easter; three copies of the service order (PCO, FreeShow, voxxa) until Q12; the stage display must never reach the stream (FA-29).
- **Settled (ruling after integration, 2026-11-26):** on a dark Sunday Presenter has no read-back, so FA-48's 99% line compares FreeShow's read-back with voxxa's own commanded slide; follower correctness is scored by shadow plus tapping until stage 2.
- **Order not fixed by 50.** 52's V17 checklist re-imports the first ten songs (W1) before the dark Sundays (W2). 51 re-imports them after the Jan 31 decision. This ADR fixes only 50's rule: sign-off before any migration weekend; re-import, `songcheck` and volunteer training done before the first double-build Sunday.

## Dependencies
ADR-032 (driver, tiers, kit rules), ADR-034 (health check, gates), ADR-026 (licensed sources), ADR-029 (private church folder; the FreeShow password is never written down), ADR-022 § 3, ADR-025 § 7 (voxxa writes no FreeShow stage text); V3e, F5, V8b, F7, V17, V18; FA-29, FA-48, FA-51, FA-55, FA-56, FA-57, FA-68, FA-70.

## Review trigger
FA-48 decided (→ Accepted or rejected); any rollback declared; FA-48 not a go by Jan 31; a church with no licensed source for most of its rotation.

## Amendment 1 (draft, 2026-11-27, founder answer FA-29)

**Fact recorded.** All Presenter output is streamed through the Blackmagic (ATEM Constellation) and streaming devices and appears on YouTube as a lower third. The founder calls this critical.

**What changes.**
1. The switch gate (Decision 2) gains a row: **the lower third matches.** F5/V3e's FS13 records which output FreeShow offers toward the Blackmagic chain (NDI, a second display, with or without a key/alpha layer) and the input Presenter uses today; one dark-Sunday rehearsal shows FreeShow's lower third on the booth monitor only; the switch needs a "matches" yes (FA-75, answered (a) 2026-11-27).
2. The Thursday 18:00 rollback rule's "lost stream" now includes "lost or wrong lower third".
3. voxxa's own windows (stage strip, Run screen, health page) are never on a display the Blackmagic chain captures; the service-day checklist gets that line (FA-60, FA-29).
4. Every slide move is visible to the stream audience, which is one more reason the shadow period is as long as the founder wants (FA-52 as answered).
