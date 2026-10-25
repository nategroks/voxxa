# Product Manager — church-autoadvance

Date: 2026-10-24 · Status: Proposed · Spec: `.10x/specs/2026-10-24-software-strategy.md`, `.10x/specs/2026-10-24-business-strategy.md` · Research: `.10x/specs/2026-10-24-user-research-plan.md`

## Problem
Volunteer slide operators (72% of churches are mostly volunteer-run; "ease of use for volunteers" is the top buying criterion) must advance lyric slides by hand while watching a worship leader who repeats choruses unplanned. Mistakes are visible to the whole congregation; training takes months; recruitment is hard.

## Users and journeys (3)
1. **Operator, Sunday 9:45:** opens Voxxa, switches to Songs, sees green, keeps hands free for lights/camera; intervenes only when the light turns red. Success: zero wrong slides. Cost of failure: congregation sees the wrong words; volunteer feels blamed.
2. **Worship leader:** nothing changes; the screen follows a repeated bridge. Success: never has to glance at the booth.
3. **Pastor/administrator (buyer):** reads one page, picks the price for their size, knows who answers on Sunday. Success: signs in under 10 minutes without a demo.

## Scope v1 (Windows launch)
In: Songs mode with ProPresenter 7.9+, FreeShow, OpenLP native; keystroke fallback for others; one-screen UI; `voxxa.toml`; Call-for-help bundle; signed installer + Store listing; eval harness.
Conditional on research O6: Songs + Scripture.
Out: lyric display, media, remote/phone control, cloud sync, SongSelect, MIDI, macOS/Linux polish (later increments).

## Success criteria (measurable)
- Eval corpus: advance precision ≥ 0.95 at recall ≥ 0.9; false advances < 1 per 10-song set; p95 latency ≤ 2.5 s (default model, 4-core CPU).
- Usability: 5 of 5 volunteers reach green within 10 minutes using only the quick-start card.
- Business (month 12): ≥ 300 churches active weekly; conversion to any paid line measured; decision gate at 3%.

## User stories (v1)
- As an operator I switch one control and Voxxa follows the song, so I can watch the room instead of the screen.
- As an operator I see why Voxxa stopped (no mic / app not found / lyrics do not match) in one sentence with a link that fixes it.
- As an operator I press Call for help and the support person has everything they need without me reading log files.
- As a buyer I see one price for my church size and what "Sunday Support" promises, in plain words.
- As a tech director I can drive and observe Voxxa from Companion or a script because the CLI is the product.

## Non-goals stated to users
Voxxa never shows lyrics, never uploads audio, never replaces the presentation software, never reports to CCLI for you.

## Pilot church (confirmed by the user, 2026-10-25)
- **The Lighthouse, Midlothian** is pilot church #1. To confirm with them before Increment 3: presentation software and version, booth computer OS, how audio reaches the booth, weekly attendance band, who operates slides, and consent for the recorded-service corpus (research plan §6).
- Two further pilots still to recruit, ideally covering the other two Tier 1 apps so all three native adapters get a real install.
