# 03 — Master layout: graphics, cue words, modes, hotkeys

| | |
|---|---|
| Run with | skill `voxxa-layout-author` |
| Who | tech lead + agent |
| Time [est.] | about 1 hour |
| Cost band | S |
| Inputs | `church-profile.json`; `kit/templates/layout.template.json`; the binding profile for the church's app in `kit/templates/bindings/`; the list of graphics the church already has in its presentation app |
| Outputs | `<church_root>\layout.json` |
| Acceptance | valid JSON; every graphic's entry for the active profile uses a `via` that exists in that profile as `show_graphic.<via>`, or `hint` (a missing entry is a warning and falls back to `hint`); every cue phrase 2–8 words, letters only, at `suggest`; no `run`, `text`, `lyrics`, `slides`, `words`, password, token, host or port keys; `voxxad --check-layout` shows 0 errors when it exists |
| Stops at | anything that would turn a cue to `auto` (that needs proven safety and a typed confirm by the tech lead, later) |

## Prompt

> You are helping a church write its voxxa master layout. Read `kit/README.md` ("The master layout in one
> paragraph"), `kit/templates/layout.template.json`, `<church_root>\church-profile.json` and the binding profile
> named by its `presentation_app` in `kit/templates/bindings/`.
>
> Rules: the layout says what should happen, never how; it never holds lyrics, passwords, hosts, ports or program
> commands (a `run` key is refused by voxxa). Never press keys in the presentation app. Never send church data out.
>
> 1. Copy the template to `<church_root>\layout.json`. Set `church_id` and `presenter.profile`.
> 2. Graphics: ask which graphics the church already has in its presentation app (logo, welcome loop, prayer card,
>    scripture background, giving slide, others). For each, record a short operator label and the entry for the
>    active profile, in the template's shape, with how the app names the graphic. One `via` convention
>    (ADR-024 § 2):
>    - `via` is a name defined per binding profile. A profile's `actions` keys are `<abstract_action>` or
>      `<abstract_action>.<via>`. Examples: `show_graphic.overlay` (freeshow), `show_graphic.macro`
>      (propresenter7_rest), `goto.item`, `goto.keys`.
>    - `hint` is a via every profile has implicitly. It shows the operator the graphic's name and never sends
>      anything.
>    - A layout graphic entry is per profile: `graphics.<id>.<profile> = {"via": ..., ...}`. Its `via` must exist
>      in **that** profile, not in every profile.
>    - A missing entry for the active profile falls back to `hint`, and `--check-layout` prints a warning, not an
>      error. So "changing app = changing one field" stays true: a missing mapping degrades to a hint.
>
>    Use `{ "via": "hint" }` when the active profile has no `show_graphic.<via>` key for the graphic. A key that
>    exists runs only after its contract test passes; until then the driver turns it into a hint (ADR-024 § 5).
>    Say so plainly: voxxa will remind the operator instead of acting. On WorshipTools Presenter every graphic is
>    a reminder.
> 3. Cue words: for each graphic that should follow a spoken phrase ("let us pray" → prayer card), write 1–12
>    phrases of 2–8 words, letters, spaces and apostrophes only. No single words. No phrase that is part of a song
>    title in the church's list. Mode `suggest`, cooldown 120 s (300 s for giving). Explain that a cue is only ever
>    armed inside a spoken item of the service plan that lists it, never during a song.
> 4. Defaults: songs `auto`; scripture and cues `suggest`; suggestion cards expire after 10 s (offer 20 or 30 s).
> 5. Hotkeys: ask whether the booth has a foot pedal, a Stream Deck or Companion. Those send F13–F19 (next, back,
>    pause, accept, dismiss, screen is ahead, screen is behind). Keyboard keys stay disabled unless the tech lead
>    wants them; voxxa checks for conflicts when it registers them. Never Ctrl+Alt+arrows.
> 6. Named actions (optional): composite steps such as a pre-service loop. Mark `service_callable` only if it
>    holds no HTTP or CLI step: no step written in `layout.json` of kind `http` or `cli`. Abstract actions inside
>    the composite (`show_graphic`, `blank`, …) are allowed, because they resolve through the compiled-in binding
>    allowlist. The template's `pre_service` (one `show_graphic` step) stays `service_callable`.
> 7. Validate: the file parses as JSON; run `voxxad --check-layout <church_root>\layout.json` if available and fix
>    each line it reports.
>
> Finish with: the graphics and which are reminders, the cue ids with their phrase counts, the hotkey source. No
> other content.
