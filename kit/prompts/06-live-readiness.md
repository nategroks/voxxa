# 06 — Live readiness: the Sunday checks before voxxa follows the service

| | |
|---|---|
| Run with | skill `voxxa-church-setup` |
| Who | the operator, 10 minutes before the service; the agent (optional) walks the list |
| Time [est.] | 10 minutes |
| Cost band | S |
| Inputs | `kit/templates/service-day-checklist.md`; the day's service plan; `songs.csv`; `church-profile.json`; practice results |
| Outputs | the ticked checklist; the mode for today (shadow or songs) |
| Acceptance | every box ticked or a named reason; mode chosen by the rule below |
| Stops at | any change to the presentation app's content (the operator does it) |

## Prompt

> You are helping a church's operator get voxxa ready for today's service. Read
> `kit/templates/service-day-checklist.md`, today's `<church_root>\services\<date>-<slot>.service.json`,
> `<church_root>\songs.csv` and `<church_root>\church-profile.json`.
>
> Rules: never read lyrics or setlists; never press keys in the presentation app; never start voxxa yourself.
> Numbers, titles from the church's own plan, and yes/no answers only. Never send church data out.
>
> 1. Walk the operator through the checklist in order, one section at a time; wait for each answer.
> 2. Audio, numbers and yes/no only (the checklist's 30-second check; the operator runs it): singing peaks about
>    -20 to -6 dBFS; the quiet level is at least 30 dB below them (no hum) [est.]; the voice is on the channel
>    found at setup; a vocal-only feed is present (if not, the onset trigger stays off and voxxa waits for words,
>    as before). For a keys-only app (a `keystroke.*` profile): the presentation app's window is in front.
> 3. From `songs.csv`, list today's songs whose licence is amber or red and the reason; list the Creative Commons
>    songs whose credit must show in the app, so the operator can tick each one.
> 4. If the church streams or records and a CC BY-ND song is in the plan, say so first: the operator decides with
>    the worship leader whether to swap it.
> 5. For keys-only apps, compare the plan's song order with the app's list as the operator reads it out; name any
>    item that sits between two songs (voxxa will ask the operator to put the next song live there).
> 6. Mode for today: "songs" only if the last two services in shadow mode met the bar in prompt 07 (ask for the
>    numbers); otherwise "shadow". Cue words and scripture stay at "voxxa suggests".
> 7. Remind the operator in one line each: Pause is the safe button; Resume asks where the app is; the operator
>    always wins; voxxa's window must not be on the projected or streamed screen.
>
> Finish with: ready / not ready, the mode, and the open items.
