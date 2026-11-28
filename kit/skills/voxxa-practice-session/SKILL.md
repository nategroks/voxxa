---
name: voxxa-practice-session
description: Run a voxxa practice session at a church's rehearsal - voxxa listens in shadow mode (moves nothing), then learns each song's sung order, pace and gaps, and the worship leader confirms each learned order before voxxa uses it on Sunday. Use before a service when the band rehearses.
---

# voxxa practice session

Runs `kit/prompts/05-practice-session.md` with `kit/templates/practice-session-checklist.md`.

## Safety rules (every step)

- voxxa moves nothing during practice: shadow mode only.
- Never open, read or print a session log, a setlist, a practice file or any lyric. Read only what voxxa prints:
  part labels (V1, C, B), counts and confidence.
- The operator starts and stops voxxa; you never start listening.
- Signed consent forms (`kit/templates/recording-consent.template.md`) are in `consent\` for everyone who sings or
  speaks: the session log holds every word voxxa hears. The rehearsal log, written with
  `--log-dir <church_root>\sessions`, stays in `<church_root>\sessions\` and rotates after 14 days.
- `voxxad` comes from the voxxa-tools zip (plan step V5c); until it ships, practice sessions run only on the voxxa
  founder's development booth.
- Never send church data to any external service or connector; never commit anything from `<church_root>`.

## Steps

1. Before: the week's setlist file `services\<date>-<slot>.setlist.json` exists; the device name comes from
   `voxxad --list-devices`; the model matches the PC's hardware class in `church-profile.json`; the input check in
   `kit/prompts/05-practice-session.md` step 2 passed (level, hum, voice channel, vocal-only feed if any).
2. During: the operator runs `voxxad --device … --shadow --model … --setlist … --log-dir <church_root>\sessions`
   and keeps the window open. A restarted song needs no note: the learner recognises a jump back to the song
   start.
3. After (when `voxxad --learn` exists): the operator runs
   `voxxad --learn <session.jsonl> --setlist <church_root>\services\<date>-<slot>.setlist.json --out <church_root>\practice`,
   which writes each song's run as proposed; read each song's order (part labels) to the worship leader; for
   each one they agree with, the operator runs `voxxad --confirm <song_fp> --practice-dir <church_root>\practice`
   (it confirms that song's newest proposed run). An unconfirmed order is never used.
4. List songs not rehearsed: voxxa may be a little late on them on Sunday.

## Output contract

Three lists: confirmed, unconfirmed, not rehearsed. Part labels and counts only.
