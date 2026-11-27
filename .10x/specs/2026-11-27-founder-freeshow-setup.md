# FreeShow setup on the booth PC (FA-55, FA-46, FA-51; F5 in the plan)

For the founder. About one hour. On a weekday with no service or rehearsal. Presenter stays the live output the whole
time; nothing in Presenter's library changes. Paste back only the values in § 4: numbers, version strings, yes/no and
key names. Never paste the API password anywhere.

## 1. Install (FA-46, FA-51)

1. Download FreeShow from its official release page: **stable channel only, never a beta**. Note the version and
   the release date (FS1). If the page shows a checksum, compare it with the downloaded file.
2. Install it beside Presenter. Do not assign any output yet (no projector, no stream).
3. Settings → General: turn **auto-update off**. FreeShow is upgraded only on a weekday, by hand, with the version
   written down first.
4. Settings → Connections (or Integrations): leave the **Planning Center** connection **off**.

## 2. API password and loopback-only ports (FA-55)

1. Settings → Connections → **API**: enable it and **set a password**. Choose a long one; it lives only inside
   FreeShow's settings. For the probe session it is placed once in the environment variable `VOXXA_FS_KEY` in the
   PowerShell window that runs the probe, and nowhere else (no file, no chat, no paste).
2. Note the two port numbers FreeShow shows for the API (expected 5505 and 5506). If they differ, write them down.
3. Windows Defender Firewall → Advanced settings → Inbound Rules: for TCP 5505 and TCP 5506, create rules that
   **allow only 127.0.0.1** and block every other address. Easiest form: one "Block" rule for each port with remote
   address "Any", then one "Allow" rule for each port with remote address 127.0.0.1 (Allow rules with a specific
   remote address win over the broad Block rule).
4. Check (FS2): from the booth PC, a request to `http://127.0.0.1:5505` **without** the password is refused (401 or
   an error), and the same request to the PC's own LAN address is refused by the firewall. Record yes/no for each.
5. On voxxa's side nothing is needed: its `require_password` setting defaults to true, so voxxa refuses an API that
   answers without a password.

## 3. The test show, the lower third and the shortcuts (FA-56, FA-57, FA-58, FA-75)

1. Build the test show from the kit's public-domain hymn file (It Is Well): one slide per line group as in the
   `.txt`, groups named V1, V2, V3, V4 and C, and a **unique layout** (every slide appears once; the chorus is one
   slide, not repeated). **No output assigned.**
2. Text transition (FS12): open the style used for the followed output and write down the transition type and time
   (expected: fade, 500 ms). Leave it as it is (FA-57: the fade stays).
3. Lower third (FS13, FA-75): write down how Presenter reaches the Blackmagic chain today (which output of the PC or
   which input of the switcher, and whether it is fill-plus-key or a plain picture). Then look in FreeShow's Outputs
   settings for the same kind of output (a second display, NDI, a key or alpha option) and write down what exists,
   names and yes/no only. Do **not** assign it.
4. Shortcuts (FS14, FA-58): open FreeShow's keyboard-shortcut list and note whether any of F13, F14, F15 or F20 is
   used; do the same from Presenter's documented keys. One yes/no per key.

## 4. What to paste back

| Row | Value |
|---|---|
| FS1 | FreeShow version, release date, auto-update off (yes/no), Planning Center connection off (yes/no) |
| FS2 | 5505 and 5506 refuse without the password (yes/no each); LAN-address request refused (yes/no) |
| FS12 | transition type and milliseconds for the followed style |
| FS13 | Presenter's path to the Blackmagic chain (output / input, fill-and-key yes/no); FreeShow's matching output (name, key/alpha yes/no) |
| FS14 | F13, F14, F15, F20: used by FreeShow (yes/no each); used by Presenter (yes/no each) |
| profile | the version written into the private `church-profile.json` as `freeshow_version` |

Then the V3e probe session runs with you at the keyboard (the plan's V3e task 4), Ctrl+C stops everything.
