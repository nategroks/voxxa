<img src="brand/voxxa-logo-on-light.png" alt="Voxxa" width="140" />

# Voxxa quick-start (Windows)

Voxxa listens to the band and moves your slides; nothing you sing leaves this computer. There is
no download page yet: your tech lead gives you the installer (`.msi` or `-setup.exe`, built from
this repository with `npm install` then `npm run tauri build`). Run it and start **Voxxa**.

## 1. The set-up guide (first start)

It opens by itself once; **Advanced > Set-up guide** opens it again. Follow its four steps:

1. **Which microphone?** Pick the input that carries the music, often the sound desk's USB input.
2. **Find your presentation app.** Open ProPresenter, FreeShow, OpenLP or OpenSong first; pick
   yours from the list. If none is found, Voxxa presses the arrow keys: keep that app in front.
3. **Get the voice model.** Press **Download** (Tiny, 75 MB, once; needs the internet).
4. **20-second test.** Press **Choose song files** to load this Sunday's songs, then **Start the
   20-second test** and play a song from the list. It passes when the slide changes.

## 2. The three controls

- **The switch.** **Songs** follows the singing; **Off** stops listening; **Songs + Scripture**
  comes later.
- **Call for help.** See section 4.
- **Advanced.** Every other setting. **Simple mode** at the top brings you back.

## 3. The light under the switch

- **Grey:** off. The sentence beside it says what is missing; add that in **Advanced**, then
  choose **Songs**. No voice model: **What to do** opens [model missing](runbooks/model-missing.md).
- **Amber:** listening; waiting for a song, hearing talking, or between songs. Nothing to do.
- **Green:** following a song: "Following `<song>`, slide 2 of 6." Nothing to do.
- **Red:** the microphone or the presentation app has a problem. **What to do** opens
  [no audio device](runbooks/no-audio-device.md) or
  [presentation app not reachable](runbooks/presentation-app-not-reachable.md). You can always
  move the slides by hand.

## 4. Call for help

It shows every file it will save: a diagnostic report, your settings (no passwords or keys) and
the last 2 minutes of heard words, if any (a box leaves them out). Nothing is sent. **Save to
Desktop** opens an e-mail naming the file, and its folder: add your tech lead's address and drag
the file in.

## 5. Updates

Voxxa never looks for updates by itself. Your tech lead can press **Check for updates** in
**Advanced > Settings** (not while Voxxa is listening); an update installs only after a yes.
