# Runbook: the voice model is missing

**For** the volunteer at the slides computer (Windows). It takes about five minutes, plus the
download (75 MB for the Tiny model).

Voxxa recognises singing with a voice model that is downloaded once and stays on this computer,
in `%APPDATA%\voxxa\models` (the Tiny model is `ggml-tiny.bin`). Nothing you sing is sent anywhere.

## Symptoms

- The light is **grey** and says: "Off. Download the voice model in Advanced first."
- Or you chose **Songs** and a message at the bottom of the window says
  `No Whisper model loaded. Download one from the Models tab first.`
- Or a message says `Failed to load Whisper model … moved it to … so it can be downloaded again`.
  The file was damaged; Voxxa set it aside so it can be downloaded again.

## Steps

1. Press **Advanced**, then the **Models** tab.
2. Find **Tiny (75MB)**:
   - **In use**: the model is ready. Go to step 4.
   - **Use**: it is on this computer. Press **Use** and wait a few seconds.
   - **Download**: press **Download** and wait for the bar to fill. This needs the internet
     once. Voxxa then uses it right away.
3. Or press **Advanced > Set-up guide** and go to step 3 ("Get the voice model"): it offers the
   same download, or **Use it** when the file is already there.
4. Press **Simple mode**. The light should now say "Off. Choose Songs to start following."
   Voxxa remembers the model and loads it again at the next start.

If the download stops or fails, check the internet connection and press **Download** again.

## When to Call for help

Call for help when the download fails twice, or when the light still asks for the model after
step 4.

1. Press **Call for help** in Voxxa. It shows you every file it will save. Nothing is sent.
2. Press **Save to Desktop**. Your e-mail program opens with a message that says which file to
   attach, and the folder with the file opens too.
3. Type your tech lead's address, drag the file into the message, and send it.

See also: [the quick-start](../quick-start.md).
