# Voxxa quick-start (Windows)

Voxxa listens to the band and moves the slides in your presentation app. It runs on this
computer only: no account, and nothing you sing is sent anywhere. It never puts lyrics on the
screen; your presentation app does that.

## 1. Install

Download the Windows installer (`.msi` or `-setup.exe`) from
<https://github.com/nategroks/voxxa/releases>, run it, and start **Voxxa**.

## 2. The set-up guide (first start)

The guide opens by itself the first time. Follow its four steps:

1. **Which microphone?** Pick the input that carries the music, usually the sound desk's USB or
   line input.
2. **Find your presentation app.** Open ProPresenter, FreeShow, OpenLP or OpenSong first; pick
   yours from the list. If none is found, Voxxa presses the arrow keys instead, so keep your
   presentation app in front during the service.
3. **Get the voice model.** Press **Download** (Tiny, 75 MB, once; needs the internet).
4. **20-second test.** Press **Choose song files** to load this Sunday's songs, then **Start the
   20-second test** and play a song from the list. It passes when the slide changes.

**Skip setup** closes the guide; **Advanced > Set-up guide** opens it again.

## 3. The three controls

- **The switch: Off / Songs.** **Songs** follows the singing; **Off** stops listening.
  "Songs + Scripture" comes in a later update.
- **Call for help.** See section 5.
- **Advanced.** Every other setting: songs, models, presentation app, microphone. **Simple mode**
  at the top brings you back.

Under the switch: the light, one sentence, and the song and slide on screen.

## 4. The light

| Colour | Means | Do |
|---|---|---|
| Grey | Off. The sentence says what is missing, if anything | Load songs or the voice model in **Advanced**, then choose **Songs** |
| Amber | Listening and waiting for a song, hearing talking (the screen stays blank), or the screen is blank between songs | Nothing |
| Green | Following a song: "Following `<song>`, slide 2 of 6." | Nothing |
| Red | The microphone or the presentation app has a problem | Press **What to do** under the sentence |

**What to do** opens the matching page:
[no audio device](runbooks/no-audio-device.md),
[presentation app not reachable](runbooks/presentation-app-not-reachable.md),
[voice model missing](runbooks/model-missing.md) (from the grey "Download the voice model"
sentence). You can always move the slides by hand in your presentation app.

## 5. Call for help

Press **Call for help**. Voxxa shows every file it will save: a diagnostic report, your settings
(passwords and keys removed) and, if there is a recent listening record, the last 2 minutes of
words it heard, with a box to leave them out. Nothing is sent. Press **Save to Desktop**: your
e-mail program opens with a message that names the file, and the file's folder opens. Type your
tech lead's address, drag the file in, and send it.
