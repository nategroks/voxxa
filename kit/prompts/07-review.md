# 07 — Review: what happened on Sunday, in numbers, and what to do next

| | |
|---|---|
| Run with | skill `voxxa-church-setup` |
| Who | the operator or tech lead after the service; the tech lead runs the scoring commands; the agent turns numbers into next steps |
| Time [est.] | 15 minutes, plus real-time tapping of the recording for a scored shadow service |
| Cost band | S |
| Inputs | for a shadow service: the numbers table from `voxxa-eval --actions` (procedure below); always: the operator's notes (step-ins, times, song positions) as context; the "Correct" bar below |
| Outputs | `<church_root>\reports\review-<date>.md` (numbers and song positions only); tickets for the voxxa team (numbers only); the rehearsal list for next week |
| Acceptance | the bar result stated from the scored numbers (or "not scored" with the reason); every step-in classified; next week's actions listed |
| Stops at | reading session logs, setlists, truth, actions files or lyrics; sending anything outside the church except a numbers-only ticket the tech lead approves |

## The bar ("Correct", per service)

voxxa may move slides on its own (mode "songs") only after **two consecutive services in shadow mode** where,
scored against the tapped truth of the recorded service:

- the right slide was reached for at least 95% of sung slides;
- at most 1 wrong move per 10 songs (a move to a slide that was neither the current nor the next one, a slide
  replaced before its last line was sung, or a skip of two);
- lyrics were never hidden in the middle of a song;
- nothing was shown during talk or prayer.

"On time" (slides within 1.5 s of the first sung word) is recorded too, but it is not part of this bar yet.
So are "Tight on time" (the share of slides, other than each song's first, shown within ±300 ms of the first sung
word), onset-trigger fires and early fires (ADR-030): an early fire is an onset fire that replaced a slide before
its last sung word started. They are not part of the bar, but an early fire also counts as a wrong move above.

## How a shadow service is scored

In shadow mode on a keys-only app such as WorshipTools Presenter, voxxa cannot see what the operator does, so the
operator's moves cannot be the score. Each shadow service is scored like this:

1. **Record the desk feed** to a `.wav` during the service, beside voxxa's live shadow run. This needs written
   consent that covers service audio (see `kit/templates/recording-consent.template.md`, its "Services" and
   "Audio recordings kept until" boxes) from everyone who sings or speaks. Talk spans are muted after
   scoring, before the recording is kept. Use the desk's own recorder, or ffmpeg on the booth PC (from `<church_root>`; list the input names
   with `ffmpeg -hide_banner -list_devices true -f dshow -i dummy`; press `q` in its window to stop):
   ```powershell
   ffmpeg -hide_banner -f dshow -i audio="<desk feed input name>" -c:a pcm_s16le "inbox\<date>-<slot>-desk.wav"
   ```
   Whether ffmpeg and voxxad can read the same input at once depends on the audio interface **[UNVERIFIED]**:
   try it at a rehearsal first.
2. **Intake and tap truth afterwards** with `kit/templates/wav-intake-checklist.md`: convert (do not mute the talk
   spans yet: the replay in step 3 must hear them), use the day's `services\<date>-<slot>.setlist.json` as `setlist.json`, tap the truth with
   `<repo>\fixtures\tools\tap-truth.ps1`, and add the `talk_start`/`talk_end` rows. The files stay in
   `<church_root>\recordings\<date>-<slot>\`: skip the checklist's § 7, § 8 and § 9 (steps 3–4 below replace § 9) unless the recording should also
   become a tune fixture (copy it then, after scoring).
3. **Replay the recording offline** on the booth PC, from `<church_root>`, with the same voxxa build and
   configuration as the live run. `--realtime` feeds the audio at live speed, so voxxa skips the same hops it
   would skip live. `cmd /c` writes the actions file without the byte-order mark that Windows PowerShell's `>`
   adds. The actions file also holds the words voxxa heard: nobody opens or prints it.
   ```powershell
   cmd /c "voxxad --stdin-audio recordings\<date>-<slot>\audio.wav --realtime --shadow --model <same model as live> --setlist services\<date>-<slot>.setlist.json --log-dir sessions > recordings\<date>-<slot>\actions.jsonl"
   ```
4. **Score** the replay's actions against the tapped truth (numbers only):
   ```powershell
   voxxa-eval --actions recordings\<date>-<slot>\actions.jsonl recordings\<date>-<slot>
   ```
   Then mute the talk spans (wav-intake checklist § 3) before the recording is kept.
5. **Cross-check only:** the live shadow session log (from `voxxad --device … --shadow`) is not on the
   recording's clock, so it never gives the score. It may only confirm that the live run and the replay saw the
   same service (for example, the same songs started).

"Shown during talk" is checked against the talk spans. If the `voxxa-eval` in use does not report it yet, the
tech lead counts the replay's song moves and unblanks inside the talk spans (the span times from
`source.md`, written here in milliseconds); the command prints one number and no text:
```powershell
$spans = @( @{ s = 600000; e = 742000 }, @{ s = 1500000; e = 1620000 } )
@(Get-Content recordings\<date>-<slot>\actions.jsonl | ForEach-Object { $_ | ConvertFrom-Json } |
  Where-Object { $_.type -eq 'action' -and $_.action -in 'goto', 'unblank' } |
  Where-Object { $t = $_.t; @($spans | Where-Object { $t -ge $_.s -and $t -le $_.e }).Count -gt 0 }).Count
```

**Operator notes are context, never the score.** They explain why the operator stepped in.

These commands need `voxxad` and `voxxa-eval` from the voxxa-tools zip (plan step V5c) and `voxxa-eval --actions`
(plan step V2). Until both ship, a shadow service can be scored only on the voxxa founder's development booth.
Without a recording and truth, the service is "not scored" and does not count towards the two.

## Prompt

> You are helping a church review last Sunday with voxxa. Read `kit/prompts/07-review.md` (the bar and the
> scoring procedure), `<church_root>\church-profile.json` and the operator's notes.
>
> Rules: never open session logs, setlists, truth files, actions files, practice files, audio or lyrics. You give
> commands; the tech lead runs them and pastes back only exit codes and the numbers table. Songs are named by
> their position in the plan ("song 2") in anything that may leave the PC. Never send church data out.
>
> 1. Ask whether the service ran in shadow mode, whether the desk feed was recorded, and whether consent for
>    service audio is on file for everyone who sang or spoke. If any answer is no, the service is "not scored":
>    go to step 5.
> 2. Walk the tech lead through "How a shadow service is scored" one step at a time: the wav-intake checklist,
>    tap-truth, the offline replay, then `voxxa-eval --actions`. Ask for the numbers table only (recall, wrong
>    moves per 10 songs, lyrics hidden mid-song, shown during talk, on-time share, Tight on time share,
>    onset-trigger fires, early fires, song-start delay). A number the `voxxa-eval` in use does not report yet is
>    "not reported".
> 3. State the bar result from those numbers and whether this is the first or second passing service in a row.
> 4. Ask for the operator's notes: each step-in with song position, slide numbers and what voxxa did (early,
>    late, wrong slide, hid the lyrics, nothing). Classify each one (voxxa wrong / voxxa late / operator choice,
>    e.g. spontaneous worship) as context for the numbers; they do not change the bar result.
> 5. Next week: songs to rehearse again (two or more wrong or late moves); songs not rehearsed this week; whether
>    to keep learning from this service (only for songs the worship leader says were sung as planned).
> 6. If the numbers or the step-ins show a pattern (same song, same part), or any early fire, draft a numbers-only
>    note for the voxxa team: song position, slide numbers, times, what happened. The tech lead decides whether to
>    send it.
> 7. Write `<church_root>\reports\review-<date>.md` with the above.
>
> Finish with: the bar result (or "not scored" and why), the next-week list, and the note for the voxxa team if
> any.
