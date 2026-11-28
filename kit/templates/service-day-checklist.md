# Service-day checklist (live readiness, 10 minutes before the service)

Date: __________  Operator: ______________  Plan: `services\__________.service.json`

## The plan and the app
- [ ] Today's plan is loaded in voxxa (not last week's)
- [ ] The presentation app's service list is in the **same order** as the plan
- [ ] Keys-only apps: nothing sits between two songs that should follow each other; where something does,
      voxxa will ask you to put the next song live
- [ ] Each song in the app opens on its blank first slide; leave it there (voxxa presses Right for the first words)
- [ ] For each song: the app's slide count (not counting the blank) = voxxa's count (from `songs.csv` `presenter_slides`)

## Licences
- [ ] Every song has a licence row; no red line from `songcheck`
- [ ] For each Creative Commons song, the credit line shows in the app (footer or a last `Credit:` slide):
      ______________ [ ]  ______________ [ ]  ______________ [ ]
- [ ] If the service is streamed or recorded: no CC BY-ND song is in the plan; SA and NC songs are as agreed

## The booth
- [ ] The presentation app is in front on the output; voxxa's own window is **not** on the projected or streamed screen
- [ ] The app is not run as administrator
- [ ] Microphone / desk feed: voxxa's meter moves
- [ ] The pedal or Stream Deck works: Next (F13), Back (F14), Pause (F15)
- [ ] Songs rehearsed today: ___ of ___ (the others may be a little late)

## Audio (30 seconds, at the soundcheck)
- [ ] Level: run the level command from `church-setup-checklist.md` § 8 with `-t 10` while one singer sings;
      peaks between about -20 and -6 dBFS: ____
- [ ] Hum: mute the feed at the desk and run it again; the quiet level (`RMS level dB`) is at least 30 dB below
      the peaks [est.]: ____
- [ ] The voice is on the channel found at setup, and the vocal-only feed is connected (if not, the onset trigger
      stays off and voxxa waits for words)

## Mode for today
- [ ] Shadow (voxxa listens and logs, moves nothing) **or** Songs (voxxa moves song slides). Use Songs only after
      two shadow Sundays met the bar in prompt 07
- [ ] Shadow service to be scored: the desk-feed recording is started before the first song (prompt 07, "How a
      shadow service is scored"), and signed consent covering service audio (the consent form's "Services" and
      "Audio recordings kept until" boxes) is in `consent\` for everyone who sings or speaks. No consent: do not record; the service is "not scored"
- [ ] Cue words and scripture stay at "voxxa suggests"

## During the service
- Pause stops voxxa moving anything; press it for spontaneous worship. Resume asks where the app is: answer it.
- "Not sure where we are": press Next if the screen is behind. voxxa follows you; the operator always wins.
- Do not stop voxxa (Off) mid-song unless something is wrong; Pause is the safe button.
