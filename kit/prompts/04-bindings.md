# 04 — Bindings: what voxxa can do in this presentation app, checked on the real app

| | |
|---|---|
| Run with | skill `voxxa-layout-author` |
| Who | the tech lead runs the test card at the booth PC; the agent explains and records |
| Time [est.] | 30 minutes |
| Cost band | S |
| Inputs | `church-profile.json`; `layout.json`; `kit/templates/bindings/<profile>.json`; `kit/templates/presenter-test-card.md` |
| Outputs | `<church_root>\presenter-test-card-<app>-<date>.md`; a "who moves it" table for the first service plan |
| Acceptance | every test-card row answered; the binding profile's known open questions for this app listed; each layout graphic marked "voxxa does it" or "voxxa reminds you" |
| Stops at | any request to edit a binding profile (they are compiled into voxxa; changes go through the voxxa team) |

## Prompt

> You are helping a church check what voxxa can do in its presentation app. Read `kit/README.md`, the binding
> profile `kit/templates/bindings/<profile>.json` for the app in `<church_root>\church-profile.json`,
> `<church_root>\layout.json` and `kit/templates/presenter-test-card.md`.
>
> Rules: you never press keys in the presentation app and never start voxxa listening; the tech lead runs each
> test and tells you the answer. Never record a password or token. Never send church data out.
>
> 1. Explain the binding profile in plain words: which actions voxxa runs today (a key that exists runs only after
>    its contract test passes; until then the driver turns it into a hint, ADR-024 § 5), which are reminders
>    (`hint`), which are impossible (`unsupported`), and the profile's `note` (known open questions, e.g. a driver
>    that may not match the app's current API). A `keys` step such as `goto.keys` is used only after the test
>    card's row T8 confirms its chord on this app.
> 2. Walk the tech lead through `presenter-test-card.md` row by row and record the answers in
>    `<church_root>\presenter-test-card-<app>-<date>.md`.
> 3. Compare: if an answer contradicts the profile (for example the app's Right on a last slide does something
>    else, or the app uses F13–F19), write it at the top of the card as "needs the voxxa team" with the row number;
>    the tech lead reports it to the voxxa team with numbers and key names only.
> 4. Produce the "who moves it" table for the church's layout: each graphic, scripture and song change → "voxxa
>    does it", "voxxa suggests" or "you (voxxa reminds you)".
> 5. If the app runs as administrator (T10), stop and ask the tech lead to change that first: Windows silently drops
>    voxxa's keys.
>
> Finish with: the card's file name, the "needs the voxxa team" rows, and the who-moves-it table.
