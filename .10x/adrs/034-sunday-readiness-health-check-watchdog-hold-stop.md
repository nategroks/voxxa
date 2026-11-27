# ADR-034: Sunday readiness: health check, watchdog, HOLD and STOP FOLLOWING

**Status:** Proposed (2026-11-26) · **Date:** 2026-11-26 · **Feature:** church-autoadvance · **Reversibility:** Medium (a new module and, if approved, a new `readiness` row; thresholds are config and sentences are copy) · **Owner:** nategroks · **Author:** 10x-Team, from `round2/50` R8, R10, C13–C15, FA-52, FA-58, FA-59, FA-62, FA-67 and the V13a/V13s prompts in `round2/52`

A stub. V13s writes the full text at its Phase 2 check and asks the founder for the row. Amends ADR-022 § 3 (adds the no-minder gate; the 2-Sunday gate is unchanged) and § 4 (the watchdog runs off-thread), ADR-024 § 6 (hotkeys and routes) and ADR-025 § 2 (hold and stop states beside `position_uncertain`).

## Context
- None of this exists in code: no watchdog, no hold, no panic (measured, 43 F1–F2). The first shadow Sunday with a volunteer needs it, so it moved up (C15).
- The listening loop decodes inline (`listen.rs:166-167`). A watchdog on that thread stalls with the decoder. (measured, 52 V13s)
- The audio layer falls back to the default device (`audio/mod.rs:49-62`) and can pick the wrong input silently. (measured, 52 tickets)
- RDP takes the console; a locked console drops keys and may move the audio. (cited, 51 risk 7)
- Escape is refused: ADR-025 § 7 forbids it, voxxa is never the front window, and Escape's effect in Presenter is undocumented (C14).

## Decision
1. **Controls (R8, FA-58; built in V13a).**
   - **F15 HOLD** (toggle): pause following and cancel pending moves. Listening, aligning and logging go on.
   - **F20 STOP FOLLOWING** (no confirm): dispatch off for the rest of the service. Listening and logging go on.
   - **F14 Back one** (the existing manual step), on the pedal or Companion page.
   - Routes `POST /api/v1/pause`, `/resume`, `/panic` and `GET /api/v1/state`, with the existing token and origin rules, on `127.0.0.1` only. `/state` is numbers and codes only: `following` (0 following, 1 holding, 2 stopped), `song`, `slide`, `position_uncertain`, `health` (0–3), `t`.
   - **No Escape.** Off only in the mode switch. No key or route stops listening. Keys a profile already uses (WorshipTools F6–F12) are never registered for it. FreeShow's output lock is the external panic. Companion or Stream Deck is a docs recipe, colour from `/state` (FA-59 a).
2. **Defined post-states.**
   - **Holding:** current slide kept, nothing sent. "Holding slide n of m. Press HOLD again to let voxxa move." Cleared by HOLD or `/resume`.
   - **Stopped:** current slide kept, nothing sent. "Stopped. You move every slide today. voxxa still listens and keeps its log." Cleared only by Off then Songs, and only after the health check passes again (`rearm_requires_green_check`).
   - **Watchdog hold:** "voxxa is slow right now. Holding slide n of m."
   - The words are FA-62's, approved on the V10 mock. Each change writes a `trace` line (`kind: operator` or `kind: watchdog`), numbers only. A manual move re-anchors the follower and does not demote the song (FA-61 as answered 2026-11-27; `demote_on_manual_move` is an off-by-default switch; on Tier B a press on Presenter's own keyboard is invisible to voxxa and costs a wrong suggestion card, never a wrong screen, spec § 4.7); shadow mode shows one line and nothing else (FA-52 as answered).
3. **Off-thread watchdog with heartbeats.** The listening loop publishes heartbeats to a shared struct. A watchdog thread wakes every 500 ms. Four signals, each with a hold reason:
   - audio chunks: none for 2 s → `audio_gap`;
   - decoded windows: voice heard and no window for 8 s → `decoder_stall`;
   - pass time: p95 over the last 30 s above 1.5 s → `slow`;
   - level against words: RMS above floor + 10 dB for 30 s with no words → `sound_no_words`.

   A fifth hold reason, `remote_session` (item 6), is a system check, not a heartbeat. (The grouping into four signals is inferred from 52's list.) `slow` and `audio_gap` clear after 10 s healthy; the rest clear on Resume. Test: `watchdog_runs_while_decoder_blocks`.
4. **Health check page: 6 rows and one top line.** Run from Simple mode when Songs is chosen, and by `voxxad --check-ready`.
   - Rows: **Sound**, **Slides app**, **Songs**, **Practice** (never red), **HOLD test** (a real F15 or `/pause` press, then Resume), **Log**.
   - Top line: "Ready for today" / "Ready, with a note" / "Not ready". The worst colour wins. "Ready, with a note" may start Songs. Only a red Slides-app row blocks (FA-62).
   - FreeShow's Slides-app row: the API answers with the password, and a self-probe to the PC's own LAN address is refused ("FreeShow answers only on loopback", ADR-032). With a FreeShow profile, a running `Presenter.exe` is amber.
   - Hidden tech-lead rows: remote session; pending reboot; VRAM (red above 1,800 MiB); FreeShow version against the pin (FA-51); a canary replay of the public TTS fixture in ≤ 30 s (red at 1.5 × the manifest's pass time).
   - Nothing dispatches during the check.
5. **Service manifest, frozen at the last rehearsal.** Written when a `--learn` session stops: version, model SHA-256, backend, config hash, setlist fingerprint, FreeShow version, and per song `{fp, practice_fp?, status?}`. The check diffs it. Build, model, backend or FreeShow version differs → red. A song differs or is not `promoted` → that song amber, suggest-only. Hashes and indices only, never titles or words.
6. **RDP check.** `GetSystemMetrics(SM_REMOTESESSION)` true → a red row and a `remote_session` hold (Win32 FFI in `keystroke.rs`'s style, with a non-Windows stub). Booth policy (FA-67): no RDP Sat 18:00 – Sun 14:00; Windows Update active hours cover services, auto-restart off; voxxa started and armed by a person, never auto-start; the check within 30 min of the service.
7. **Device pinned by name.** "Default" is red (`default_device_is_red`).
8. **Per-service report, numbers only.** The checklist's last step runs `voxxa-eval --service-report` (V2), with `--truth-from-screen` when FreeShow was live. It writes `reports\service-<date>-<slot>.json` and one `trend.csv` row, and lists the songs that hit a demotion trigger.
9. **Runbooks.** `docs/runbooks/sunday-window.md`, `watchdog-tripped.md` and `panic-pressed.md`, each linked from its status row.
10. **Two gates (FA-52, C13).**
    - **Gate 1** (ADR-022 § 3, unchanged): 2 scored shadow Sundays meeting the Correct bar → songs mode with a minder at the PC.
    - **Gate 2** (no minder; Tier A only, R10): ≥ 4 consecutive green Sundays, ≥ 300 scored occurrences, 0 early moves beyond the line (0.3 s on rehearsal truth, 1.0 s live), 0 spurious presses, two volunteers passed the card. Thresholds tighten, never loosen.
    - 51 § 2.7 and 52 V15 add "one fault drill run" (F6) to gate 2. Kept, because FA-52 allows tightening. The computable formulas are in 51 § 2.7.
11. **Budget.** One new row, `readiness (src-tauri/src/readiness/)` ≤ 900 (the 300 estimate × 3 under the founder's FA-54 rule, 2026-11-27), is proposed; the founder must say yes (FA-54's new-module rule). The alternative: the watchdog in the wiring row's remaining ~55 lines, and the manifest moved to V12. Health page row 150; wiring ≤ 15 (publish calls) beside V13a's ≤ 40; voxxad ≤ 8. No cap is raised.

## Alternatives Considered
| Alternative | Why not |
|---|---|
| Escape as a reserved panic key | refused (C14, ADR-025 § 7) |
| One gate of four shadow Sundays | mixes "minder at the PC" with "nobody at the PC" (C13) |
| Watchdog on the listening thread | it would stall with the decoder it watches (inferred from `listen.rs:166-167`) |
| Widen voxxa's API for a stage button | FreeShow's output lock is the external panic; the API stays on loopback (R8) |
| Companion buttons as voxxa code | a docs recipe is enough (FA-59 a, R17) |

## Consequences
- **Positive:** every Sunday starts from a measured check; doubt holds the slide instead of guessing; the minder has three controls with known outcomes; each service leaves a numbers-only record for the gates.
- **Negative:** one more module and row; a HOLD test press every Sunday; a red Slides-app row blocks Songs.
- **Risks:** the thresholds (2 s, 8 s, 1.5 s, 30 s, 1,800 MiB) are first guesses (inferred); a watchdog that trips too often trains minders to ignore amber; on Presenter, voxxa sees manual moves only through its own hotkeys (R10).

## Dependencies
ADR-022 § 3–4, ADR-024 § 6, ADR-025 § 2 and § 7, ADR-028 (rows), ADR-032 (Slides-app row, `position_uncertain`); V13a, V2, V3b task 6 (level and hum numbers), V10 (FA-62 words), V14 (draws the page); FA-52, FA-54, FA-58, FA-59, FA-62, FA-67.

## Review trigger
The founder's answer on the `readiness` row; the first watchdog hold on a live Sunday; any panic; FA-62's words change; gate 2 reached.
