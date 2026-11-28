# 05 — Practice session: voxxa listens to the rehearsal and learns each song

| | |
|---|---|
| Run with | skill `voxxa-practice-session` |
| Who | the operator at the rehearsal; the worship leader confirms the orders; the agent prepares and reads counts |
| Time [est.] | the rehearsal, plus 15 minutes after |
| Cost band | S |
| Inputs | the week's service plan and its setlist file; `kit/templates/practice-session-checklist.md`; signed consent forms (`kit/templates/recording-consent.template.md`) in `consent\`; `voxxad` from the voxxa-tools zip (plan step V5c; until then only the voxxa founder's development booth) |
| Outputs | a session log in `<church_root>\sessions\`; practice memory files in `<church_root>\practice\` (when `voxxad --learn` exists) |
| Acceptance | every rehearsed song has a proposed order; the ones that match what was sung are confirmed; the "not rehearsed" list is known |
| Stops at | reading any session log or practice file contents (they hold heard words) |

## Prompt

> You are helping a church run a voxxa practice session. Read `kit/README.md`,
> `kit/templates/practice-session-checklist.md` and the week's `<church_root>\services\<date>-<slot>.service.json`.
>
> Rules: voxxa moves nothing in a practice session (shadow mode). You never open, read or print a session log,
> a setlist, a practice file or any lyric; you read only the counts and part labels that voxxa prints. The
> operator starts and stops voxxa; you never start listening yourself. Never send church data out.
>
> Before the rehearsal:
> 1. Check that `services\<date>-<slot>.setlist.json` exists (file name only) and ask the operator whether signed
>    consent forms are in `consent\` for everyone who will sing or speak (the session log holds every word voxxa
>    hears). No consent → stop.
> 2. Input check before voxxa learns (`kit/templates/church-setup-checklist.md` § 8; the operator runs it at the
>    soundcheck, it saves nothing). Ask for numbers and yes/no only: singing peaks after gain (aim for about
>    -20 to -6 dBFS; outside that, the operator adjusts the desk output or the PC's input level and checks again);
>    the quiet level with the feed muted (the hum check); the channel the voice is on; whether the feed is
>    vocal-only (voice mics only, no instruments; preferred).
> 3. Give the operator the exact `voxxad --list-devices` and `voxxad --device … --shadow --model … --setlist …
>    --log-dir <church_root>\sessions` commands from the checklist, with the model from `church-profile.json`'s
>    hardware class. If the band restarts a song, nobody needs to note it: the learner recognises a jump back to
>    the song start.
>
> After the rehearsal:
> 4. If `voxxad --learn` exists, give the operator the learn command:
>    `voxxad --learn <church_root>\sessions\<session file>.jsonl --setlist <church_root>\services\<date>-<slot>.setlist.json --out <church_root>\practice`.
>    It writes each song's run as proposed and prints, per song, the order it heard as part labels (V1, C, B),
>    the number of parts, how many were unsure and a confidence. Read those lines back to the worship leader,
>    song by song, and ask "Is that what you sang?". For each yes, give the command
>    `voxxad --confirm <song_fp> --practice-dir <church_root>\practice`, which confirms that song's newest
>    proposed run. A no is left unconfirmed: voxxa never uses it.
> 5. If `--learn` does not exist yet, record only which songs were rehearsed and that the log was kept.
> 6. List the songs not reached in the rehearsal: on Sunday voxxa may be a little late on them.
>
> Finish with: the input check numbers, songs confirmed, songs left unconfirmed, songs not rehearsed. Counts and
> labels only.
