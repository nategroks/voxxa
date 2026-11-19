# Runbook: Voxxa can't reach the presentation app

**Use this when** the light is **red** with one of these sentences:

- "ProPresenter 7.9+ not found. Voxxa can't change slides." (or FreeShow, OpenLP,
  OpenSong, or "Your presentation app not found");
- "ProPresenter 7.9+ didn't take the last slide change: ..." followed by a reason
  ("Keystroke didn't take ..." when Voxxa presses the arrow keys);
- "... didn't take the last slide change. Check that it is open."

Voxxa follows the song, but the slide change does not reach the app that shows the
slides. "Not found" goes away as soon as Voxxa is connected again. "Didn't take" stays
until a slide change gets through or you choose Songs again.

## Steps

1. **Is the presentation app open?** Start it, and open this week's presentation.
2. **Is its remote control on?** Voxxa talks to it over the network, even on the same
   computer:
   - ProPresenter 7.9 or later: **Settings → Network**; Voxxa expects port **1025**.
   - OpenLP: **Settings → Remote**; Voxxa expects port **4316**.
   - OpenSong: **Settings → General → System → Automation API**; port **8082**.
   - FreeShow: Voxxa looks for it on port **5506**.
3. **Connect again.** In Voxxa, click **Advanced**, then the **Settings** tab. Under
   **Presentation App**, click **Auto-detect** and pick your app, or choose it in the
   list, check **Host** and **Port**, and click **Connect**. Under the buttons it should
   say "Connected: " and your app.
4. **Passwords are not remembered.** ProPresenter 6 and 7.0 to 7.8 (the WebSocket
   choices) and OpenLP with a username need the password again after every start of
   Voxxa: enter it in step 3 and click **Connect**. Until then the light is red.
5. **No app found? Voxxa presses the arrow keys.** Then your presentation app must be
   the window in front. If the reason says "focus your presentation app", click once
   into the presentation app's window, and leave Voxxa in the background.
6. **Back to Simple mode.** Click **Simple mode**, then choose **Songs**. The light
   turns amber, then green when Voxxa follows a song.

If the presentation app runs on another computer, both computers must be on the same
network, and **Host** is that computer's address, not 127.0.0.1.

## When to call for help

Click **Call for help** if the light is still red after step 6. Voxxa shows you every
file before it saves anything (see the [quick-start](../quick-start.md)). Until it is
fixed, change the slides by hand in your presentation app.

Other runbooks: [microphone](no-audio-device.md), [voice model missing](model-missing.md).
