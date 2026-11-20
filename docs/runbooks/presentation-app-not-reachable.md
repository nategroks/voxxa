# Runbook: Voxxa can't reach the presentation app

**For** the volunteer at the slides computer (Windows). It takes about five minutes.

## Symptoms

The light is **red** and says one of these (`<app>` is ProPresenter, FreeShow, OpenLP, OpenSong
or "Your presentation app"):

- "`<app>` not found. Voxxa can't change slides." Voxxa is not connected to the app you chose,
  for example because the app was closed, or because it needs a password, which Voxxa never
  saves.
- "`<app>` didn't take the last slide change: `<reason>`." The app refused or did not answer.
  The reason "focus your presentation app: Voxxa's own window has the keyboard" means Voxxa is
  pressing the arrow keys for you and its own window was in front.

With **WorshipTools Presenter** the light can also be **amber** and say "Bring WorshipTools
Presenter to the front." or "Put "`<song>`" live in Presenter." See
[WorshipTools Presenter](#worshiptools-presenter) below.

## Steps

1. Make sure the presentation app is open and a presentation is loaded.
2. **If Voxxa presses keys for you** (no app was found in the set-up guide): click into the
   presentation app's window right after you choose **Songs**, and leave it in front. Keys sent
   while Voxxa's window is in front are refused on purpose.
3. **If Voxxa talks to the app over the network:** in the app, check that its remote control or
   network option is on. Voxxa looks on this computer at the usual ports: ProPresenter 1025,
   FreeShow 5506, OpenLP 4316, OpenSong 8082. If the app runs on another computer, both computers
   must be on the same network, and **Host** is that computer's address, not 127.0.0.1.
4. In Voxxa, press **Advanced**, then the **Settings** tab. Under **Presentation App**, press
   **Auto-detect**, or pick your app, check the host and port, type the password if it has one,
   and press **Connect**.
5. Or press **Advanced > Set-up guide**, go to step 2 ("Find your presentation app"), and pick
   your app from the list. If it finds nothing, open the app and press **Search again**.
6. Press **Simple mode** and check the light. It turns amber or green once Voxxa is connected,
   and after the next slide change that reaches the app.

ProPresenter's WebSocket connection and OpenLP with a login need the password again after every
start of Voxxa, and so does OpenSong with an API key: enter the key again under **Presentation
App > API key (optional)** and press **Connect**. Voxxa never saves passwords or keys. Expect this
red light after each start until you connect in step 4.

While you fix it, move the slides by hand in your presentation app.

## WorshipTools Presenter

WorshipTools Presenter has no remote control that Voxxa can use, so Voxxa presses its keys: Right
for the next slide and Left for the previous one. When Presenter is running, the set-up guide
finds it in step 2 and says so. After a restart, Voxxa uses Presenter's keys again by itself.

- **Keep Presenter in front.** Voxxa presses a key only while a Presenter window is in front.
  Otherwise nothing is pressed and the light is amber: "Bring WorshipTools Presenter to the
  front." Click into Presenter's window. The next slide change goes through.
  If Voxxa's own window is in front, the light is **red** instead and ends "focus your
  presentation app: Voxxa's own window has the keyboard." The fix is the same: click into
  Presenter's window.
- **Put each new song live yourself.** In Presenter, Right on a song's last slide moves on to the
  next service item, which may be a reading or an announcement. So Voxxa never presses a key from
  one song into the next. When the next song starts, the light is amber: "Put "`<song>`" live in
  Presenter." Click that song in Presenter's service list. Voxxa then follows it from its first
  slide. Voxxa can't see Presenter's screen: if the song is not put live, its next key still goes
  to whatever item Presenter shows.
  If the light goes green but Presenter shows the wrong item, Voxxa did not see you put the song
  live: put it live, then click the slide being sung in Presenter. Voxxa goes on from there.
  Don't use Voxxa's song button for that song: Voxxa thinks the song is already live, so the
  button presses Left back to its first slide, and Left on Presenter's first slide may leave the
  song.
- **Voxxa can't clear Presenter's screen yet.** F10 clears the lyrics, but how to bring them back
  has not been tested, so Voxxa doesn't press it. During a prayer the slide stays up and the light
  says so. Clear it in Presenter yourself if you need to. Voxxa never presses F11 or F12 (they
  clear the background media, or everything), Up or Down, or the number pad's + or − (they change
  the service item).
- **Don't run Presenter as administrator.** Windows silently drops keys that an ordinary program
  sends to a program run as administrator: nothing moves, the light stays amber or green, and
  Voxxa can't tell. If keys do nothing while Presenter is in front, close Presenter and start it
  normally. Check that its shortcut does not have **Run this program as an administrator** ticked
  (right-click the shortcut, **Properties**, **Compatibility**).

## When to Call for help

Call for help when the light is still red after step 6.

1. Press **Call for help** in Voxxa. It shows you every file it will save. Passwords are never
   in it. Nothing is sent.
2. Press **Save to Desktop**. Your e-mail program opens with a message that says which file to
   attach, and the folder with the file opens too.
3. Type your tech lead's address, drag the file into the message, and send it.

See also: [the quick-start](../quick-start.md).
