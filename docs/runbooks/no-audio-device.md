# Runbook: Voxxa can't hear the microphone

**For** the volunteer at the slides computer (Windows). It takes about five minutes.
Voxxa only listens while the switch is on **Songs**. It never records audio to a file.

## Symptoms

- The light is **red** and says: "Can't hear the mic. Check the cable, then choose Songs again."
  The switch has gone back to **Off**.
- Or you chose **Songs**, the switch went back to **Off**, and a message at the bottom of
  the window says `No default audio input device` or `Device '<name>' not found`.
- Or the light stays amber ("Listening. Waiting for a song from the list.") while the band
  plays, and the slides never move.

## Steps

1. Check the cable from the sound desk (or the USB microphone) into the computer. Unplug it and
   plug it in again. Wait ten seconds.
2. Open Windows **Settings > System > Sound**. Under **Input**, make sure the input is listed and
   its level bar moves when someone speaks or plays.
3. Open Windows **Settings > Privacy & security > Microphone**. Turn on **Microphone access** and
   **Let desktop apps access your microphone**.
4. In Voxxa, press **Advanced**, then the **Settings** tab. Under **Audio Device**, pick the input
   that carries the music (often the sound desk's USB or line input). "Default" uses whatever
   Windows chose.
5. Press **Simple mode** at the top, then choose **Songs** again. The light should turn amber:
   "Listening. Waiting for a song from the list."
6. Still not sure Voxxa hears the band? Press **Advanced > Set-up guide** and run the
   **20-second test** (step 4) while the band plays. "I couldn't hear anything" means no sound
   reaches Voxxa: go back to step 1. "I hear sound but no words" means the input is too quiet or
   too far from the speakers: turn it up at the desk.

While you fix it, move the slides by hand in your presentation app.

## When to Call for help

Call for help when the light is still red after step 5, or when the input does not appear in
step 2 or step 4.

1. Press **Call for help** in Voxxa. It shows you every file it will save. Nothing is sent.
2. Press **Save to Desktop**. Your e-mail program opens with a message that says which file to
   attach, and the folder with the file opens too.
3. Type your tech lead's address, drag the file into the message, and send it.

See also: [the quick-start](../quick-start.md).
