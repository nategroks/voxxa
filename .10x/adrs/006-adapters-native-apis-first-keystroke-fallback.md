# ADR-006: Integrate through native local APIs (ProPresenter, FreeShow, OpenLP first) with a keystroke fallback; no SongSelect

**Status:** Accepted · **Date:** 2026-10-24 · **Feature:** church-autoadvance · **Reversibility:** Easy per adapter · **Owner:** nategroks

## Context
Keystroke injection is focus-dependent and blind. Three widely used apps expose local APIs that both drive slides and read the current slide text (research A1–A3), which lets Voxxa re-sync to what is actually on screen and import the setlist without any lyric licensing of its own. EasyWorship, MediaShout and WorshipTools Presenter do not (A4). ProPresenter has the largest user base (A7); FreeShow and OpenLP are open source and Windows/Linux-heavy.

## Decision
- One `trait Adapter` with capability flags: `advance`, `prev`, `goto(song, slide)`, `show_ref(reference)`, `current() -> Option<Position>`, `setlist() -> Option<Setlist>`.
- Increment 3 ships ProPresenter (HTTP `/v1`), FreeShow (REST/WS) and OpenLP (HTTP v2) adapters, each ≤ 300 LOC with a recorded-traffic contract test; Holyrics and Proclaim follow when a church asks.
- The existing enigo keystroke adapter remains as Tier 3 for every other app, sends only when the target app is foreground, and runs "blind" (Voxxa trusts its own position; status light says so).
- Discovery: probe localhost well-known ports (1025-range scan for ProPresenter is not reliable; read the user's configured port), 5505/5506, 4316, 52195; confirm identity via version endpoints before any command.
- No SongSelect integration (API retired, partner programme closed; B3). Setlists come from the adapter, Planning Center Services, or a file (ADR-007).
- MIDI (loopMIDI/IAC) deferred until requested; it requires an extra install on Windows.

## Alternatives
| Alternative | Why not |
|---|---|
| Keystrokes only | Cannot read position; breaks on focus loss; no setlist import. |
| Bitfocus Companion as the only bridge | Adds a second app for volunteers to run; Companion modules exist for the same apps, so Voxxa talks to the apps directly and also stays Companion-controllable via its CLI. |
| Build on ProPresenter only | Largest base, but Mac-heavy and unverified current API hosting (A1 gap); open-source apps give a Windows/Linux launch path and upstream contribution. |

## Consequences
Good: real position feedback, setlist import, scripture display via `start_scripture` (FreeShow) and Bibles plugin (OpenLP). Bad: three vendor APIs to track; mitigated by contract fixtures and the 300-LOC cap. Risks: ProPresenter endpoint renames since 7.9; test on a live install early in Increment 3.

## Review trigger
Re-open if > 25% of support tickets concern one Tier 2/3 app (then promote it: MIDI or a vendor conversation).
