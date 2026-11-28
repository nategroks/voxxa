# Church setup checklist (one time, about one evening)

Church id: ______________  Tech lead: ______________  Date: __________  Kit version: kit-v0.1.0

Tick each box. Agents tick only the boxes marked (agent); everything else is done by a person at the booth PC.

## 1. Intake
- [ ] `church-profile.json` filled from `church-profile.template.json` (prompt 01) (agent)
- [ ] Presentation app and version written down: ______________
- [ ] The presentation app is **not** run as administrator (right-click its shortcut → Properties → Compatibility)
- [ ] Audio: the desk's aux output goes to the booth PC's audio input; voxxa's level meter moves during singing
- [ ] Does the church livestream or record services? ____  Monetised or asking for gifts? ____
- [ ] Is the booth desktop duplicated to the projector or captured for the stream? ____ (must be "no" before
      voxxa shows slide lines on its Run screen)
- [ ] Does the church still hold a CCLI licence? ____ (sets `ccli.active`)

## 2. Install (PowerShell, one line each)
- [ ] `winget install --id astral-sh.uv -e` then, in a new window, `uv python install 3.12`
- [ ] `winget install Gyan.FFmpeg`
- [ ] voxxa from its installer (never a test build on the booth PC)
- [ ] Once plan step V5c ships: the voxxa-tools zip of the same version and its `.sha256` file downloaded from the
      release page; the zip checked with `Get-FileHash -Algorithm SHA256` (the hash equals the one in the `.sha256`
      file); unzipped **outside** `<church_root>` (e.g. `%LOCALAPPDATA%\voxxa-tools\`) and that folder added to the
      user `PATH` (commands in `kit/README.md` "Tools and when they arrive"); `voxxad --version` and
      `voxxa-eval --help` both run (until then, practice sessions, recording intake and the layout check run only
      on the voxxa founder's development booth; see `kit/README.md`)
- [ ] Model downloaded in voxxa (Advanced → Models): the size the hardware class allows
- [ ] Hardware check, paste only these lines into `church-profile.json`:
      `Get-CimInstance Win32_Processor | Select Name, NumberOfCores`
      `Get-CimInstance Win32_VideoController | Select Name, AdapterRAM`
      `Test-Path C:\Windows\System32\vulkan-1.dll`

## 3. The church folder
- [ ] `<church_root>` made exactly as `private-folder.md` (default `%LOCALAPPDATA%\voxxa-church\`) (agent may create empty folders)
- [ ] `.ignore` file holding `*` at its root
- [ ] Not inside OneDrive, Dropbox, Google Drive, a network share, a code repository, roaming `%APPDATA%`, voxxa's
      install folder `%LOCALAPPDATA%\voxxa\` or voxxa's app-data folder `%LOCALAPPDATA%\com.voxxa.app\`
- [ ] BitLocker on if any recording will be kept
- [ ] Backup drive chosen (BitLocker-encrypted, kept at the church); first dated copy made as in
      `private-folder.md` rule 2 (never `/MIR`)
- [ ] Consent forms: `recording-consent.template.md` printed for everyone who will be recorded (singers,
      speakers; a parent or guardian for minors); signed scans go in `consent\`

## 4. Songs (prompt 02; repeat per song)
- [ ] `songs\<slug>.txt` typed or pasted by the tech lead (agents never see the words)
- [ ] The **same** text pasted into the presentation app
- [ ] One row in `songs.csv`: licence code, credit line, source, evidence of the licence, who checked and when;
      for public domain `pd_basis` (and `published_year` when the basis is `us_published_95_years`)
- [ ] Keys-only apps (WorshipTools, universal): `presenter_slides` = the app's slide count, not counting the
      blank slide the song opens on, and equal to voxxa's count (a blank line or `---` starts a slide)
- [ ] If the credit is a slide, it is the song's last slide in **both** copies and starts with `Credit:`
- [ ] `uv run kit/tools/songcheck.py <church_root>` shows no red line (or the same checks by hand until step V5b ships it)

## 5. Master layout and bindings (prompts 03 and 04)
- [ ] `layout.json` from the template: `presenter.profile` set; each graphic mapped to the app's own graphic (or `hint`)
- [ ] Cue phrases reviewed: 2–8 words, said only when the graphic should show; all at `suggest`
- [ ] The presenter test card done for this app and version (`presenter-test-card.md`)
- [ ] Hotkeys: pedal or Stream Deck sends F13–F19, or keyboard keys enabled after the conflict check
- [ ] `voxxad --check-layout layout.json` shows 0 errors (when available)

## 6. First service plan
- [ ] `services\<date>-<slot>.service.json` (for example `2026-12-06-am.service.json`) from the template, items
      in the app's order; it compiles to `services\<date>-<slot>.setlist.json`
- [ ] For keys-only apps: songs that follow each other in the plan follow each other in the app's list
- [ ] Notes hold no personal details or prayer requests

## 7. Agent settings
- [ ] If an AI agent helps: `agent-settings.example.md` applied; no Planning Center, Drive or Gmail connector enabled for this work

## 8. Audio feed (before the first practice session)
- [ ] Connection: a balanced cable (XLR or TRS) into a line input or a USB audio interface, or a DI box or
      isolator if the PC has only a mic jack. An unbalanced cable can pick up ground-loop hum
- [ ] A vocal-only output for voxxa: an aux or matrix with the voice mics only (singers and speaking mics, no
      instruments). Without it voxxa still follows the words, but the onset trigger (ADR-030) stays off
- [ ] Level: the tech lead runs this while one singer sings at full voice (20 seconds; it saves nothing and prints
      numbers only; list input names with `ffmpeg -hide_banner -list_devices true -f dshow -i dummy`). Read
      `Peak level dB` under each `Channel:` (1 is left, 2 is right)
```powershell
ffmpeg -hide_banner -f dshow -i audio="<desk feed input name>" -t 20 -af astats -f null -
```
- [ ] Singing peaks between about -20 and -6 dBFS after gain (desk output or the PC's input level): ch 1 ____ ch 2 ____
- [ ] Hum: mute the output at the desk and run the command again. The quiet level (`RMS level dB`) is at least
      30 dB below the singing peaks [est.]: ____. If not, it is usually ground-loop hum: use the balanced or
      isolated connection above
- [ ] Channel the voice is on: ____ (1 left, 2 right, or both). If one side only, send the voice to both
      sides at the desk
