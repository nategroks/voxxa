# Runbook: the voice model is missing

**Use this when** one of these shows in Voxxa:

- the light is **grey**: "Off. Download the voice model in Advanced first.";
- the Set-up guide's step 3 says "Voxxa needs a voice model";
- you choose **Songs** and a message at the bottom of the window says "No Whisper model
  loaded";
- a message says Voxxa failed to load the model and moved it aside "so it can be
  downloaded again".

The voice model is the file Voxxa uses to recognise words. It downloads once (Tiny is
75 MB) and then stays on this computer. The download is the only time Voxxa needs the
internet for it.

## Steps

1. **Connect to the internet** for the download.
2. **Download it.** In Voxxa, click **Advanced**, then the **Models** tab. Next to
   **Tiny (75MB)**, click **Download** and wait for it to finish. (Or open **Set-up
   guide**, go to step 3, and click **Download**.)
3. **Already downloaded?** If the button says **Use**, click it. When it says
   **In use**, the model is ready.
4. **Back to Simple mode.** Click **Simple mode**. The light should say "Off. Choose
   Songs to start following." Voxxa loads the same model again at its next start.
5. **No internet at church?** Copy the file `ggml-tiny.bin` from another computer that
   runs Voxxa: it is in the folder `%APPDATA%\voxxa\models`. Paste it into the same
   folder on this computer (type `%APPDATA%\voxxa\models` in the File Explorer address
   bar), then do step 3.
6. **A model that failed to load** was renamed to `ggml-tiny.bin.unreadable` in that
   folder. Download it again (step 2). You may delete the `.unreadable` file.

## When to call for help

Click **Call for help** if the download fails twice, or the model will not load. Voxxa
shows you every file before it saves anything (see the
[quick-start](../quick-start.md)). Until it is fixed, change the slides by hand in your
presentation app.

Other runbooks: [microphone](no-audio-device.md),
[presentation app not reachable](presentation-app-not-reachable.md).
