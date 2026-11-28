# Practice session checklist (the rehearsal before the service)

voxxa listens to the rehearsal, moves **nothing**, and afterwards learns each song: the order actually sung
(repeats, skips, tags), the pace, the gaps, and which words it tends to hear for each slide. That memory is used
in the service. Nothing leaves the booth PC.

Needs `voxxad` from the voxxa-tools zip (plan step V5c; see `kit/README.md` "Tools and when they arrive"). Until
V5c ships, practice sessions run only on the voxxa founder's development booth.

Date: __________  Operator: ______________  Service plan: `services\__________.service.json`

`<date>-<slot>` is the service, for example `2026-12-06-am`.

## Before the band starts
- [ ] Everyone being recorded knows voxxa is listening. voxxa's session log holds every word it hears, so each
      singer and speaker has signed the kit's consent form (`templates/recording-consent.template.md`, filed in
      `consent\`) with the rehearsal and text-log boxes ticked, and the "Audio recordings kept until" box too if
      audio is kept
- [ ] The presentation app is open, but voxxa will not touch it (shadow mode)
- [ ] The service plan's setlist file exists: `services\<date>-<slot>.setlist.json`
- [ ] Desk feed level: voxxa's meter moves during singing and does not sit at the top
- [ ] Input check done (`church-setup-checklist.md` § 8): singing peaks about -20 to -6 dBFS, the hum check passed,
      the voice channel known; a vocal-only feed if the desk has one

## Start listening in shadow mode (PowerShell)
```powershell
voxxad --device "<desk feed device name>" --shadow --model <model for this PC> `
       --setlist "<church_root>\services\<date>-<slot>.setlist.json" `
       --log-dir "<church_root>\sessions"
```
- [ ] `voxxad --list-devices` used to copy the exact device name
- [ ] The model is the one the speed check chose for this PC (on a PC that cannot keep up, the log has gaps and voxxa learns less)
- [ ] The window stays open for the whole rehearsal; close it with Ctrl+C at the end
- [ ] Nothing to note if the band stops and restarts a song: the learner recognises a jump back to the song start

## After the rehearsal (when `voxxad --learn` exists, step V12)
```powershell
voxxad --learn "<church_root>\sessions\session-<date>-<time>.jsonl" `
       --setlist "<church_root>\services\<date>-<slot>.setlist.json" `
       --out "<church_root>\practice"
```
- [ ] For each song, voxxa prints the order it heard as part labels, e.g. `V1 V2 C V3 C C · 6 parts · 1 unsure`,
      and writes it as a **proposed** run
- [ ] The worship leader or operator confirms each order that matches what was sung (this confirms that song's
      newest proposed run):
```powershell
voxxad --confirm <song_fp> --practice-dir "<church_root>\practice"
```
- [ ] A wrong order is not confirmed (it is never used)
- [ ] Songs not reached in the rehearsal are noted: they stay "not rehearsed" on Sunday

## Afterwards
- [ ] Rehearsal logs stay in `<church_root>\sessions\` and rotate after 14 days; never e-mail or upload them
- [ ] If a song's words change later, its practice memory is ignored until the next rehearsal
