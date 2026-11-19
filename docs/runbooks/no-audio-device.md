# Runbook: Voxxa can't hear the microphone

**Use this when** one of these shows in Voxxa:

- the light is **red**: "Can't hear the mic. Check the cable, then choose Songs again.";
- you choose **Songs**, the switch goes back to **Off**, and a message at the bottom of
  the window names the audio device (for example "No default audio input device");
- the 20-second test says "I couldn't hear anything" or "The microphone stopped".

Voxxa listens to the input that carries the music, usually the sound desk's USB or line
input. Audio never leaves the computer.

## Steps

1. **Check the cable.** Is the USB or line cable from the sound desk plugged in at both
   ends? Unplug it and plug it in again. A USB input that was unplugged during the
   service turns the light red until you choose Songs again.
2. **Check that the desk sends sound.** The desk channel or output that feeds the
   computer must be on and turned up.
3. **Check that Windows sees it.** Open Windows **Settings → System → Sound**. Under
   **Input**, find the device, choose it, and sing or play a little: its level bar
   should move.
4. **Check that Windows lets apps use it.** Open Windows **Settings → Privacy &
   security → Microphone**. **Microphone access** and **Let desktop apps access your
   microphone** must be on.
5. **Choose the device in Voxxa.** In Voxxa, click **Advanced**, then **Set-up guide**.
   Step 1, "Which microphone?", lists the inputs. Pick the one from step 3 and click
   **Next**. Click **Skip setup** to close the guide, then **Simple mode**.
6. **Start again.** Choose **Songs**. The light should turn amber: "Listening. Waiting
   for a song from the list."
7. **Test it (optional, before the service).** In the Set-up guide, go to step 4 and run
   the 20-second test with a song from this week's list.

## When to call for help

Click **Call for help** if the light is still red after step 6, or the switch keeps
going back to Off. Voxxa shows you every file before it saves anything (see the
[quick-start](../quick-start.md)). Until it is fixed, change the slides by hand in your
presentation app.

Other runbooks: [presentation app not reachable](presentation-app-not-reachable.md),
[voice model missing](model-missing.md).
