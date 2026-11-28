# The private church folder

Every church keeps one private folder on its booth PC: `<church_root>`. It holds everything that belongs to
that church. It never leaves the booth PC. This page describes its **layout only**; it contains no church data.

| Who | `<church_root>` |
|---|---|
| A new church (default; one church per PC) | `%LOCALAPPDATA%\voxxa-church\` |
| The voxxa founder's development booth (the pilot church) | `C:\dev\voxxa-church-private\pilot\` |
| The fake second church for the kit acceptance test | `C:\dev\voxxa-church-private\fake-church-2\` |

On the founder's development booth the shared private tools (the lab) live in `C:\dev\voxxa-church-private\lab\`,
beside the church folders, not inside one. A church PC keeps its optional lab in `<church_root>\lab\`.

Never put `<church_root>` inside a code repository, OneDrive, Dropbox, Google Drive, a network share, the
roaming `%APPDATA%` folder (a domain profile may copy it to a server), voxxa's install folder
`%LOCALAPPDATA%\voxxa\` (Windows folder names ignore case; Explorer may show it with a capital letter) or voxxa's
app-data folder `%LOCALAPPDATA%\com.voxxa.app\`. The default `%LOCALAPPDATA%\voxxa-church\` sits beside those two
folders, never inside them: installing, updating, uninstalling or renaming voxxa never touches it.

voxxa's own files stay outside `<church_root>`: its session logs in `%LOCALAPPDATA%\com.voxxa.app\sessions\`
(app data, kept 14 days, may be removed with the app; what the uninstaller's "delete app data" option removes is
**[UNVERIFIED]**) and its trust store in `%LOCALAPPDATA%\com.voxxa.app\trust\`.
Only the logs `voxxad` writes with `--log-dir` (a rehearsal captured for `voxxad --learn`, a shadow-Sunday run or
replay) go into `<church_root>\sessions\`.

## Layout

```
<church_root>\
  .ignore                       one line: *   (code-search tools skip this folder)
  README.md                     "Private. Never leaves this PC." and the backup rule
  church-profile.json           intake answers (from templates/church-profile.template.json)
  songs\
    <slug>.txt                  one song's words, one slide per paragraph          LYRICS
  songs.csv                     licence register, one row per .txt                 no lyrics
  licences\
    <slug>.md                   where the licence was found, with date and link   no lyrics
  layout.json                   the master layout (from templates/layout.template.json)
  services\
    <yyyy-mm-dd>-<slot>.service.json      weekly service plans, e.g. 2026-12-06-am.service.json
    <yyyy-mm-dd>-<slot>.setlist.json      compiled from the plan; the setlist voxxa loads  LYRICS
  practice\
    <song_fp>.json              practice memory, one file per song (file name is a fingerprint, not a title)
  sessions\
    session-*.jsonl             voxxad --log-dir logs: every word heard, prayers included  PRIVATE, 14-day rotation
  inbox\
    <anything>.wav|.mp3|.m4a|.mp4      recordings as received; deleted after intake
  recordings\
    <yyyy-mm-dd>-<id>\
      audio.wav                 16 kHz mono 16-bit PCM
      source.md                 where it came from, consent reference, retention date (no lyrics)
      setlist.json, truth.jsonl, intake.json   made during intake, then copied into fixtures\ (wav-intake checklist)
      actions.jsonl             a shadow-Sunday replay's would-be moves (prompt 07)               PRIVATE
  fixtures\
    manifest.json               which fixtures are tune vs held-out, truth hashes, look count
                                (from templates/fixtures-manifest.template.json)
    tune\<yyyy-mm-dd>-<id>\      audio.wav, setlist.json, truth.jsonl, intake.json
    heldout\<yyyy-mm-dd>-<id>\   same; chosen by hand once, before any tuning; never re-tuned on
  reports\
    eval-<date>-<commit>.json   numbers only
    review-<date>.md            after-service review (prompt 07), numbers and song positions only
    LICENSED_RUNS.md            one row per local evaluation run: numbers, licence class, credit line
  lab\                          optional uv project for evaluation tools (templates/lab/)
  consent\
    <date>-<name>.pdf           signed recording consent forms (templates/recording-consent.template.md)
  tools\                        (only if a future voxxa version allows programs; empty today)
```

## Rules

1. **Not a git repository.** A refusing hook can be skipped and does not stop someone adding a remote.
2. **Backups:** once a week, make a new dated copy of the folder on one BitLocker-encrypted drive kept at the
   church. Nowhere else. Never use `/MIR`: it copies an accidental delete into the backup.
   ```powershell
   robocopy "<church_root>" "E:\voxxa-backup\<yyyy-mm-dd>" /E /XD sessions inbox
   ```
   - `E:` is the backup drive's letter; `<yyyy-mm-dd>` is today's date, so each week gets its own folder. In
     PowerShell: `"E:\voxxa-backup\$(Get-Date -Format yyyy-MM-dd)"`. robocopy exit codes below 8 mean success.
   - `/XD sessions inbox` leaves out the session logs (every word heard, kept 14 days only) and unprocessed
     recordings, so the backup does not keep them for 8 weeks.
   - Keep the last 8 dated copies; delete the oldest when a ninth is made.
   - Every quarter, check that one copy restores: copy one dated folder to a temporary folder on the booth PC,
     compare the file count with `(Get-ChildItem -Recurse -File <folder>).Count`, then delete the temporary copy.
   - A deleted recording stays in older copies for up to 8 weeks; on a withdrawal of consent, delete it from
     every dated copy too.
3. **BitLocker** on the booth PC if the folder holds any recording.
4. **Retention:** recordings carry a retention date in `source.md`; delete the audio once it is reduced to
   numbers-only fixtures or the date passes. Session logs rotate after 14 days. Practice files live until the
   song is deleted or its words change.
5. **Talk is private:** any recording kept as a fixture has its talk and prayer spans replaced with silence at
   intake; only the span times are kept. Everyone whose voice is recorded has signed consent on the kit's form,
   `templates/recording-consent.template.md` (a parent or guardian signs for a minor), filed as
   `consent\<date>-<name>.pdf` before the recording is made. On a withdrawal, delete that person's recordings and
   every fixture made from them within 7 days.
6. **Agents** may read only the files a prompt names, and never `songs\*.txt`, `*.setlist.json`, `setlist.json`,
   `truth.jsonl`, `actions.jsonl`, `sessions\*`, `practice\*`, `consent\*`, `reports\*` or `licences\*` contents,
   or any audio. They may list file names, write a new file a prompt names, and run tools that print counts.
7. **Nothing from here is ever committed** to the voxxa repository. The repository's `.gitignore` refuses these
   file names; its media guard and `kit_check.py` (step V5b) refuse them too, and the lyric scanner checks any
   file before it moves to a public path.
8. **Secrets** (presentation-app passwords, API tokens) are never stored in this folder.
